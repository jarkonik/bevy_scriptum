// TODO: maybe make all runtime engines not send and spawn threads for them like Ruby
// TODO: make sure ruby is statically linked
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, LazyLock, Mutex},
    thread::{self, JoinHandle},
};

use bevy::{
    asset::Asset,
    ecs::{component::Component, resource::Resource, schedule::ScheduleLabel},
    reflect::TypePath,
};
use magnus::Ruby;
use magnus::{function, prelude::*};
use serde::Deserialize;

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    FuncArgs, Runtime,
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
    sender: Option<crossbeam_channel::Sender<RubyClosure>>,
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
            sender: Some(sender),
            handle: Some(handle),
        }
    }

    fn execute<T: Send + 'static>(&self, f: Box<dyn FnOnce(Ruby) -> T + Send>) -> T {
        let (return_sender, return_receiver) = crossbeam_channel::bounded(0);
        self.sender
            .as_ref()
            .unwrap()
            .send(Box::new(move |ruby| {
                return_sender.send(f(ruby)).unwrap();
            }))
            .unwrap();
        return_receiver.recv().unwrap()
    }
}

impl Drop for RubyThread {
    fn drop(&mut self) {
        drop(self.sender.take().unwrap());
        let handle = self.handle.take().unwrap();
        handle.join().unwrap();
    }
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
pub struct RubyValue(());

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
                ruby.eval::<magnus::value::Value>(&script).unwrap();
                RubyValue(())
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

        fn callback() -> magnus::Value {
            let ruby = magnus::Ruby::get().unwrap();
            let method_name: magnus::value::StaticSymbol =
                ruby.class_object().funcall("__method__", ()).unwrap();
            let method_name = method_name.to_string();
            let callbacks = RUBY_CALLBACKS.lock().unwrap();
            let f = callbacks.get(&method_name).unwrap();
            f((), vec![]).unwrap();
            ruby.qnil().as_value()
        }

        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| {
                ruby.define_global_function(&name, function!(callback, 0));
                RubyValue(())
            }));

        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        _script_data: &mut Self::ScriptData,
        _entity: bevy::prelude::Entity,
        _args: impl for<'a> crate::FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let name = name.to_string();
        self.ruby_thread
            .as_ref()
            .unwrap()
            .execute(Box::new(move |ruby| {
                let _: magnus::Value = ruby.class_object().funcall(name, ()).unwrap();
                RubyValue(())
            }));

        Ok(RubyValue(()))
    }

    fn call_fn_from_value(
        &self,
        _value: &Self::Value,
        _context: &Self::CallContext,
        _args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        todo!()
    }

    fn is_current_thread() -> bool {
        false
    }
}

pub mod prelude {
    pub use super::RubyRuntime;
}

impl<T> FromRuntimeValueWithEngine<'_, RubyRuntime> for T {
    fn from_runtime_value_with_engine(_value: RubyValue, _engine: &magnus::Ruby) -> Self {
        todo!();
    }
}

impl<T> IntoRuntimeValueWithEngine<'_, T, RubyRuntime> for T {
    fn into_runtime_value_with_engine(_value: T, _engine: &magnus::Ruby) -> RubyValue {
        RubyValue(())
    }
}

impl FuncArgs<'_, RubyValue, RubyRuntime> for () {
    fn parse(self, _engine: &magnus::Ruby) -> Vec<RubyValue> {
        Vec::new()
    }
}

impl<T> FuncArgs<'_, RubyValue, RubyRuntime> for Vec<T> {
    fn parse(self, _engine: &magnus::Ruby) -> Vec<RubyValue> {
        self.into_iter().map(|_x| RubyValue(())).collect()
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<'a, $($t,)+> FuncArgs<'a, RubyValue, RubyRuntime>
            for ($($t,)+)
        {
            fn parse(self, _engine: &'a magnus::Ruby) -> Vec<RubyValue> {
                todo!();
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
