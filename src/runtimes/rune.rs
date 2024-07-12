use std::sync::Mutex;

use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use rune::{runtime::RuntimeContext, Context, Source, Sources, Vm};
use serde::Deserialize;

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    FuncArgs, Runtime,
};

#[derive(Resource)]
pub struct RuneRuntime {
    context: Context,
    engine: std::sync::Arc<RuntimeContext>,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct RuneSchedule;

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct RuneScript(pub String);

#[derive(Component)]
pub struct RuneScriptData;

impl GetExtensions for RuneScript {
    fn extensions() -> &'static [&'static str] {
        &["rn"]
    }
}

impl From<String> for RuneScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Default for RuneRuntime {
    fn default() -> Self {
        let context = Context::with_default_modules().unwrap();
        let runtime = std::sync::Arc::new(context.runtime().unwrap());
        Self {
            context: context,
            engine: runtime,
        }
    }
}

#[derive(Clone)]
pub struct RuneValue(());

impl Runtime for RuneRuntime {
    type Schedule = RuneSchedule;

    type ScriptAsset = RuneScript;

    type ScriptData = RuneScriptData;

    type CallContext = ();

    type Value = RuneValue;

    type RawEngine = ();

    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        todo!()
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        todo!()
    }

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let mut sources = Sources::new();
        sources
            .insert(Source::memory("pub fn add(a, b) { a + b }").unwrap())
            .unwrap();
        let result = rune::prepare(&mut sources)
            .with_context(&self.context)
            .build()
            .unwrap();

        Vm::new(self.engine.clone(), std::sync::Arc::new(result));
        Ok(RuneScriptData)
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
        todo!()
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl for<'a> crate::FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        todo!()
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
    pub use super::RuneRuntime;
}

impl<T> FromRuntimeValueWithEngine<'_, RuneRuntime> for T {
    fn from_runtime_value_with_engine(value: RuneValue, engine: &()) -> Self {
        todo!();
    }
}

impl<T> IntoRuntimeValueWithEngine<'_, T, RuneRuntime> for T {
    fn into_runtime_value_with_engine(value: T, engine: &()) -> RuneValue {
        todo!();
    }
}

impl FuncArgs<'_, RuneValue, RuneRuntime> for () {
    fn parse(self, _engine: &()) -> Vec<RuneValue> {
        Vec::new()
    }
}

impl<T> FuncArgs<'_, RuneValue, RuneRuntime> for Vec<T> {
    fn parse(self, engine: &()) -> Vec<RuneValue> {
        self.into_iter().map(|x| RuneValue(())).collect()
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<'a, $($t,)+> FuncArgs<'a, RuneValue, RuneRuntime>
            for ($($t,)+)
        {
            fn parse(self, engine: &'a ()) -> Vec<RuneValue> {
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
