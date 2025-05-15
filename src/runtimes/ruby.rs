// TODO: maybe make all runtime engines not send and spawn threads for them like Ruby
// TODO: make sure ruby is statically linked
// TODO: add tests for every runtime for return value
// TODO: unwinding https://doc.rust-lang.org/nomicon/ffi.html#variadic-functions
// TODO: maybe unify api and call non thread methods non_send
// TODO: use ruby arena to rewind state

use std::{
    collections::HashMap,
    ffi::{c_void, CString},
    sync::{Arc, Condvar, LazyLock, Mutex},
    thread::{self, JoinHandle},
};

use anyhow::anyhow;
use bevy::{
    asset::Asset,
    ecs::{
        component::Component, query::QueryParManyUniqueIter, resource::Resource,
        schedule::ScheduleLabel,
    },
    reflect::TypePath,
    tasks::futures_lite::io,
};
use magnus::{
    block::Proc, data_type_builder, function, method::ReturnValue, try_convert, value::Lazy,
    DataType, DataTypeFunctions, IntoValue, RClass, RObject, Ruby, TryConvert, TypedData,
};
use magnus::{prelude::*, rb_sys::FromRawValue};
use rb_sys::{rb_define_global_function, rb_scan_args, VALUE};
use serde::Deserialize;

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    promise::Promise,
    FuncArgs, Runtime, ScriptingError,
};

#[derive(Resource)]
pub struct RubyRuntime {
    ruby_thread: Option<RubyThread>,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct RubySchedule;

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct RubyScript(pub String);

#[derive(Component)]
pub struct RubyScriptData;

impl GetExtensions for RubyScript {
    fn extensions() -> &'static [&'static str] {
        &["rb"]
    }
}

impl From<String> for RubyScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

type RubyClosure = Box<dyn FnOnce(Ruby) + Send>;

struct RubyThread {
    sender: crossbeam_channel::Sender<RubyClosure>,
    handle: Option<JoinHandle<()>>,
}

static RUBY_THREAD: LazyLock<Arc<(Mutex<Option<RubyThread>>, Condvar)>> =
    LazyLock::new(|| Arc::new((Mutex::new(Some(RubyThread::spawn())), Condvar::new())));

impl RubyThread {
    fn spawn() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Box<dyn FnOnce(Ruby) + Send>>();

        let handle = thread::spawn(move || {
            let _cleanup = unsafe { magnus::embed::init() };
            while let Ok(f) = receiver.recv() {
                let ruby = Ruby::get().unwrap();
                f(ruby);
            }
        });

        RubyThread {
            sender,
            handle: Some(handle),
        }
    }

    fn execute<T: Send + 'static>(&self, f: Box<dyn FnOnce(Ruby) -> T + Send>) -> T {
        let (return_sender, return_receiver) = crossbeam_channel::bounded(0);
        self.sender
            .send(Box::new(move |ruby| {
                return_sender.send(f(ruby)).unwrap();
            }))
            .unwrap();
        return_receiver.recv().unwrap()
    }
}

impl Drop for RubyThread {
    fn drop(&mut self) {
        let handle = self.handle.take().unwrap();
        handle.join().unwrap();
    }
}

impl DataTypeFunctions for Promise<(), RubyValue> {}

unsafe impl TypedData for Promise<(), RubyValue> {
    fn class(ruby: &Ruby) -> magnus::RClass {
        static CLASS: Lazy<RClass> = Lazy::new(|ruby| {
            let class = ruby.define_class("Promise", ruby.class_object()).unwrap();
            class.undef_default_alloc_func();
            class
        });
        ruby.get_inner(&CLASS)
    }

    fn data_type() -> &'static magnus::DataType {
        static DATA_TYPE: DataType = data_type_builder!(Promise<(), RubyValue>, "promise").build();
        &DATA_TYPE
    }
}

fn then(r_self: magnus::Value, callback: magnus::Value) -> magnus::Value {
    let promise: &Promise<(), RubyValue> = TryConvert::try_convert(r_self).unwrap();
    promise.clone().then(RubyValue::new(callback)).into_value()
}

impl Default for RubyRuntime {
    fn default() -> Self {
        let (lock, cvar) = &*Arc::clone(&RUBY_THREAD);
        let mut ruby_thread = lock.lock().unwrap();

        while ruby_thread.is_none() {
            ruby_thread = cvar.wait(ruby_thread).unwrap();
        }
        let ruby_thread = ruby_thread.take().unwrap();
        cvar.notify_all();

        ruby_thread.execute(Box::new(|ruby| {
            // TODO: maybe put promise in a module , maybe do so for other runtimes too
            let promise = ruby.define_class("Promise", ruby.class_object()).unwrap();
            promise
                .define_method("and_then", magnus::method!(then, 1))
                .unwrap();
        }));
        // TODO: add test for those types for every runtime
        // engine
        //     .register_userdata_type::<BevyEntity>(|typ| {
        //         typ.add_field_method_get("index", |_, entity| Ok(entity.0.index()));
        //     })
        //     .expect("Failed to register BevyEntity userdata type");
        //
        // engine
        //     .register_userdata_type::<Promise<(), LuaValue>>(|typ| {
        //         typ.add_method_mut("and_then", |engine, promise, callback: Function| {
        //             Ok(Promise::then(promise, LuaValue::new(engine, callback)))
        //         });
        //     })
        //     .expect("Failed to register Promise userdata type");
        //
        // engine
        //     .register_userdata_type::<BevyVec3>(|typ| {
        //         typ.add_field_method_get("x", |_engine, vec| Ok(vec.0.x));
        //         typ.add_field_method_get("y", |_engine, vec| Ok(vec.0.y));
        //         typ.add_field_method_get("z", |_engine, vec| Ok(vec.0.z));
        //     })
        //     .expect("Failed to register BevyVec3 userdata type");
        // let vec3_constructor = engine
        //     .create_function(|_, (x, y, z)| Ok(BevyVec3(Vec3::new(x, y, z))))
        //     .expect("Failed to create Vec3 constructor");
        // engine
        //     .globals()
        //     .set("Vec3", vec3_constructor)
        //     .expect("Failed to set Vec3 global");
        //
        Self {
            ruby_thread: Some(ruby_thread),
        }
    }
}

impl Drop for RubyRuntime {
    fn drop(&mut self) {
        let (lock, cvar) = &*Arc::clone(&RUBY_THREAD);
        let mut ruby_thread = lock.lock().unwrap();
        *ruby_thread = self.ruby_thread.take();
        cvar.notify_all();
    }
}

#[derive(Clone)]
pub struct RubyValue(magnus::value::Opaque<magnus::Value>);

impl RubyValue {
    fn nil(ruby: &Ruby) -> Self {
        Self::new(ruby.qnil().as_value())
    }

    fn new(value: magnus::Value) -> Self {
        Self(magnus::value::Opaque::from(value))
    }
}

impl Runtime for RubyRuntime {
    type Schedule = RubySchedule;

    type ScriptAsset = RubyScript;

    type ScriptData = RubyScriptData;

    type CallContext = ();

    type Value = RubyValue;

    type RawEngine = magnus::Ruby;

    fn with_engine_thread_mut<T: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Self::RawEngine) -> T + Send + 'static,
    ) -> T {
        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |mut ruby| f(&mut ruby)))
    }

    fn with_engine_thread<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Self::RawEngine) -> T + Send + 'static,
    ) -> T {
        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| f(&ruby)))
    }

    fn with_engine_mut<T>(&mut self, _f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        unimplemented!();
    }

    fn with_engine<T>(&self, _f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        unimplemented!();
    }

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        _entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let script = script.0.clone();
        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| {
                let value = ruby.eval::<magnus::value::Value>(&script).unwrap();
                RubyValue::new(value)
            }));
        Ok(RubyScriptData)
    }

    fn register_fn(
        &mut self,
        name: String,
        _arg_types: Vec<std::any::TypeId>,
        f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<
                crate::promise::Promise<Self::CallContext, Self::Value>,
                crate::ScriptingError,
            > + Send
            + Sync
            + 'static,
    ) -> Result<(), crate::ScriptingError> {
        type CallbackClosure = Box<
            dyn Fn(
                    (),
                    Vec<RubyValue>,
                )
                    -> Result<crate::promise::Promise<(), RubyValue>, crate::ScriptingError>
                + Send
                + Sync
                + 'static,
        >;
        static RUBY_CALLBACKS: LazyLock<Mutex<HashMap<String, CallbackClosure>>> =
            LazyLock::new(|| Mutex::new(HashMap::new()));
        let mut callbacks = RUBY_CALLBACKS.lock().unwrap();
        callbacks.insert(name.clone(), Box::new(f));

        fn callback(args: &[magnus::Value]) -> magnus::Value {
            let ruby = magnus::Ruby::get().unwrap();
            let method_name: magnus::value::StaticSymbol =
                ruby.class_object().funcall("__method__", ()).unwrap();
            let method_name = method_name.to_string();
            let callbacks = RUBY_CALLBACKS.lock().unwrap();
            let f = callbacks.get(&method_name).unwrap();
            let result = f(
                (),
                args.iter()
                    .map(|arg| RubyValue::new(arg.into_value()))
                    .collect(),
            )
            .expect("failed to call callback");
            result.into_value()
        }

        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| {
                ruby.define_global_function(&name, function!(callback, -1));
                RubyValue::nil(&ruby)
            }));

        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        _script_data: &mut Self::ScriptData,
        _entity: bevy::prelude::Entity,
        args: impl for<'a> crate::FuncArgs<'a, Self::Value, Self> + Send + 'static,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let name = name.to_string();
        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| {
                let args: Vec<_> = args
                    .parse(&ruby)
                    .into_iter()
                    .map(|a| ruby.get_inner(a.0))
                    .collect();
                let return_value: magnus::Value = ruby
                    .class_object()
                    .funcall(name, args.as_slice())
                    .map_err(|e| {
                        ScriptingError::RuntimeError(Box::new(io::Error::other(e.to_string())))
                    })?;
                Ok(RubyValue::new(return_value))
            }))
    }

    // TODO: add test
    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        _context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let value = value.clone(); // TODO: maybe just clone/wrap where we added Send + static
                                   // currently?>
        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| {
                let f: Proc = TryConvert::try_convert(ruby.get_inner(value.0)).unwrap();

                let args: Vec<_> = args
                    .into_iter()
                    .map(|x| ruby.get_inner(x.0).as_value())
                    .collect();
                let result: magnus::Value = f.funcall("call", args.as_slice()).map_err(|e| {
                    ScriptingError::RuntimeError(Box::new(io::Error::other(e.to_string())))
                })?;
                Ok(RubyValue::new(result))
            }))
    }

    fn is_current_thread() -> bool {
        false
    }
}

pub mod prelude {
    pub use super::RubyRuntime;
}

impl<T: TryConvert> FromRuntimeValueWithEngine<'_, RubyRuntime> for T {
    fn from_runtime_value_with_engine(value: RubyValue, engine: &magnus::Ruby) -> Self {
        let inner = engine.get_inner(value.0);
        T::try_convert(inner).unwrap()
    }
}

impl<T: IntoValue> IntoRuntimeValueWithEngine<'_, T, RubyRuntime> for T {
    fn into_runtime_value_with_engine(value: T, _engine: &magnus::Ruby) -> RubyValue {
        RubyValue::new(value.into_value())
    }
}

impl FuncArgs<'_, RubyValue, RubyRuntime> for () {
    fn parse(self, _engine: &magnus::Ruby) -> Vec<RubyValue> {
        Vec::new()
    }
}

impl<T: IntoValue> FuncArgs<'_, RubyValue, RubyRuntime> for Vec<T> {
    fn parse(self, engine: &magnus::Ruby) -> Vec<RubyValue> {
        self.into_iter()
            .map(|x| RubyValue::new(x.into_value()))
            .collect()
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<'a, $($t: IntoValue,)+> FuncArgs<'a, RubyValue, RubyRuntime>
            for ($($t,)+)
        {
            fn parse(self, _engine: &'a magnus::Ruby) -> Vec<RubyValue> {
                vec![
                    $(RubyValue::new(self.$idx.into_value()), )+
                ]
            }
        }
    };
}

impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W, 23 X, 24 Y, 25 Z);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W, 23 X, 24 Y);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W, 23 X);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_tuple!(0 A, 1 B, 2 C, 3 D);
impl_tuple!(0 A, 1 B, 2 C);
impl_tuple!(0 A, 1 B);
impl_tuple!(0 A);
