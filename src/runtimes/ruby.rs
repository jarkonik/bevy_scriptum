use std::{
    sync::LazyLock,
    thread::{self, JoinHandle},
};

use bevy::{
    asset::Asset,
    ecs::{component::Component, resource::Resource, schedule::ScheduleLabel},
    reflect::TypePath,
};
use magnus::Ruby;
use magnus::{embed::Cleanup, function, prelude::*};
use serde::Deserialize;

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    FuncArgs, Runtime,
};

#[derive(Resource)]
pub struct RubyRuntime {}

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
struct RubyEngine(Cleanup);

struct RubyThread {
    sender: Option<crossbeam_channel::Sender<Box<dyn FnOnce(Ruby) + Send>>>,
    handle: Option<JoinHandle<()>>,
}

static RUBY_THREAD: LazyLock<RubyThread> = LazyLock::new(|| RubyThread::spawn());

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

    fn execute_in<T: Send + 'static>(&self, f: Box<dyn FnOnce(Ruby) -> T + Send>) -> T {
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
        Self {}
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

    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        f(&mut magnus::Ruby::get().unwrap())
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        RUBY_THREAD.execute_in(Box::new(move |ruby| f(&ruby)))
    }

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        _entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let script = script.0.clone();
        RUBY_THREAD.execute_in(Box::new(move |ruby| {
            ruby.eval::<magnus::value::Value>(&script).unwrap();
            RubyValue(())
        }));
        Ok(RubyScriptData)
    }

    fn register_fn(
        &mut self,
        name: String,
        _arg_types: Vec<std::any::TypeId>,
        _f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<
                crate::promise::Promise<Self::CallContext, Self::Value>,
                crate::ScriptingError,
            > + Send
            + Sync
            + 'static,
    ) -> Result<(), crate::ScriptingError> {
        fn callback(_val: magnus::Value) -> magnus::Value {
            let ruby = magnus::Ruby::get().unwrap();
            ruby.qnil().as_value()
        }

        RUBY_THREAD.execute_in(Box::new(move |ruby| {
            ruby.define_global_function(&name, function!(callback, 1));
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
        RUBY_THREAD.execute_in(Box::new(move |ruby| {
            let _: Result<magnus::Value, _> = ruby.class_object().funcall(name, ());
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
