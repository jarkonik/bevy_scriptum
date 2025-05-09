use std::sync::mpsc::{Receiver, Sender};
use std::{
    cell::{LazyCell, OnceCell},
    sync::{LazyLock, Mutex, OnceLock},
    thread::{self, JoinHandle},
};

use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, resource::Resource, schedule::ScheduleLabel},
    reflect::TypePath,
};
use magnus::{
    embed::{init, Cleanup},
    function,
    prelude::*,
};
use serde::Deserialize;

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    FuncArgs, Runtime,
};

#[derive(Resource)]
pub struct RubyRuntime {
    ruby_thread: Option<JoinHandle<()>>,
    ruby_thread_sender: Option<crossbeam_channel::Sender<()>>,
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
struct RubyEngine(Cleanup);

// TODO: Add SAFETY?
unsafe impl Send for RubyEngine {}

static RUBY_ENGINE: LazyLock<Mutex<RubyEngine>> =
    LazyLock::new(|| Mutex::new(RubyEngine(unsafe { magnus::embed::init() })));

impl Default for RubyRuntime {
    fn default() -> Self {
        let (ruby_thread_sender, ruby_thread_receiver) = crossbeam_channel::unbounded::<()>();
        let ruby_thread = thread::spawn(move || {
            let _cleanup = LazyLock::force(&RUBY_ENGINE);
            while let Ok(val) = ruby_thread_receiver.recv() {
                println!("received");
            }
        });
        Self {
            ruby_thread: Some(ruby_thread),
            ruby_thread_sender: Some(ruby_thread_sender),
        }
    }
}

impl Drop for RubyRuntime {
    fn drop(&mut self) {
        drop(self.ruby_thread_sender.take().unwrap());
        let ruby_thread = self.ruby_thread.take().unwrap();
        ruby_thread.join().unwrap();
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
        todo!()
    }

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let ruby = magnus::Ruby::get().unwrap();
        ruby.eval::<magnus::value::Qnil>(&script.0);
        Ok(RubyScriptData)
    }

    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<std::any::TypeId>,
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
        let ruby = magnus::Ruby::get().unwrap();

        static mut FUN: Vec<Box<dyn Fn()>> = Vec::new();
        unsafe {
            FUN.push(Box::new(move || {
                f((), vec![]).unwrap();
            }));
        }

        let sender = self.ruby_thread_sender.as_ref().clone();
        let x = 5;

        fn callback() -> magnus::Value {
            // sender.unwrap().send(());
            let ruby = magnus::Ruby::get().unwrap();
            unsafe {
                FUN.pop().unwrap()();
            }
            ruby.qnil().as_value()
        }

        ruby.define_global_function(&name, function!(callback, 0));
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl for<'a> crate::FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let ruby = magnus::Ruby::get().unwrap();
        let _: magnus::value::Value = ruby.class_object().funcall(name, ()).unwrap();
        Ok(RubyValue(()))
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        todo!()
    }
}

pub mod prelude {
    pub use super::RubyRuntime;
}

impl<T> FromRuntimeValueWithEngine<'_, RubyRuntime> for T {
    fn from_runtime_value_with_engine(value: RubyValue, engine: &magnus::Ruby) -> Self {
        todo!();
    }
}

impl<T> IntoRuntimeValueWithEngine<'_, T, RubyRuntime> for T {
    fn into_runtime_value_with_engine(value: T, engine: &magnus::Ruby) -> RubyValue {
        RubyValue(())
    }
}

impl FuncArgs<'_, RubyValue, RubyRuntime> for () {
    fn parse(self, _engine: &magnus::Ruby) -> Vec<RubyValue> {
        Vec::new()
    }
}

impl<T> FuncArgs<'_, RubyValue, RubyRuntime> for Vec<T> {
    fn parse(self, engine: &magnus::Ruby) -> Vec<RubyValue> {
        self.into_iter().map(|x| RubyValue(())).collect()
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<'a, $($t,)+> FuncArgs<'a, RubyValue, RubyRuntime>
            for ($($t,)+)
        {
            fn parse(self, engine: &'a magnus::Ruby) -> Vec<RubyValue> {
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
