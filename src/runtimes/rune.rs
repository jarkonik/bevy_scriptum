use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use rune::{
    alloc::clone::TryClone as _,
    runtime::{ConstValue, GuardedArgs, RuntimeContext, Shared, Stack, VmResult},
    termcolor::{ColorChoice, StandardStream},
    vm_try, Context, Diagnostics, FromValue, Module, Source, Sources, ToValue, Unit, Value, Vm,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    FuncArgs, Runtime,
};

#[derive(Resource)]
pub struct RuneRuntime {
    pub context: Context,
    pub engine: std::sync::Arc<RuntimeContext>,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct RuneSchedule;

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct RuneScript(pub String);

#[derive(Component)]
pub struct RuneScriptData {
    pub unit: Arc<Unit>,
}

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
pub struct RuneValue(Arc<Mutex<ConstValue>>);

impl Runtime for RuneRuntime {
    type Schedule = RuneSchedule;

    type ScriptAsset = RuneScript;

    type ScriptData = RuneScriptData;

    type CallContext = ();

    type Value = RuneValue;

    type RawEngine = Self;

    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        f(self)
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        f(self)
    }

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let mut sources = Sources::new();
        sources.insert(Source::memory(&script.0).unwrap()).unwrap();
        let mut diagnostics = Diagnostics::new();
        let result = rune::prepare(&mut sources)
            .with_context(&self.context)
            .with_diagnostics(&mut diagnostics)
            .build();
        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources).unwrap();
        }

        let unit = result.unwrap();
        Ok(RuneScriptData {
            unit: Arc::new(unit),
        })
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
        let mut module = Module::new();
        module
            .raw_function(name.as_str(), move |stack: &mut Stack, args_len: usize| {
                let mut args = Vec::new();
                for _ in 0..args_len {
                    let val = ConstValue::from_value(vm_try!(stack.pop())).unwrap();
                    let val = RuneValue(Arc::new(Mutex::new(val)));
                    args.push(val);
                }
                let result = f((), args).unwrap();
                // let args = { args.into_iter().map(|x| LuaValue::new(engine, x)).collect() };
                // let result = f((), args).unwrap();
                stack.push(Value::EmptyTuple).unwrap();
                VmResult::Ok(())
            })
            .build()
            .unwrap();
        self.context.install(module).unwrap();
        self.engine = Arc::new(self.context.runtime().unwrap());
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl for<'a> crate::FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let mut vm = Vm::new(self.engine.clone(), script_data.unit.clone());
        let args = RuneArgs(args.parse(self));
        let result = vm.call([name], args).unwrap();
        let result = ConstValue::from_value(result).unwrap();

        Ok(RuneValue(Arc::new(Mutex::new(result))))
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

struct RuneArgs(Vec<RuneValue>);

impl GuardedArgs for RuneArgs {
    type Guard = ();

    unsafe fn unsafe_into_stack(self, stack: &mut rune::runtime::Stack) -> VmResult<Self::Guard> {
        for val in self.0 {
            let val = val.0.lock().unwrap();
            let val = val.try_clone().unwrap().into_value().unwrap();
            stack.push(val).unwrap();
        }
        VmResult::Ok(())
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

impl rune::runtime::Args for RuneArgs {
    fn into_stack(self, stack: &mut rune::runtime::Stack) -> VmResult<()> {
        for val in self.0.into_iter() {
            let val = val.0.lock().unwrap();
            let val = val.try_clone().unwrap().into_value().unwrap();
            stack.push(val).unwrap();
        }
        VmResult::Ok(())
    }

    fn try_into_vec(self) -> VmResult<rune::alloc::Vec<rune::Value>> {
        let mut v = rune::alloc::Vec::new();

        for val in self.0 {
            let val = val.0.lock().unwrap();
            let val = val.try_clone().unwrap().into_value().unwrap();
            v.try_push(val).unwrap();
        }
        return VmResult::Ok(v);
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

impl<T> FromRuntimeValueWithEngine<'_, RuneRuntime> for T {
    fn from_runtime_value_with_engine(value: RuneValue, engine: &RuneRuntime) -> Self {
        todo!();
    }
}

impl<T> IntoRuntimeValueWithEngine<'_, T, RuneRuntime> for T {
    fn into_runtime_value_with_engine(value: T, engine: &RuneRuntime) -> RuneValue {
        RuneValue(Arc::new(Mutex::new(ConstValue::EmptyTuple)))
    }
}

impl FuncArgs<'_, RuneValue, RuneRuntime> for () {
    fn parse(self, _engine: &RuneRuntime) -> Vec<RuneValue> {
        Vec::new()
    }
}

impl<T: ToValue> FuncArgs<'_, RuneValue, RuneRuntime> for Vec<T> {
    fn parse(self, engine: &RuneRuntime) -> Vec<RuneValue> {
        self.into_iter()
            .map(|x| {
                RuneValue(Arc::new(Mutex::new(
                    ConstValue::from_value(x.to_value().unwrap()).unwrap(),
                )))
            })
            .collect()
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<'a, $($t,)+> FuncArgs<'a, RuneValue, RuneRuntime>
            for ($($t,)+)
        {
            fn parse(self, engine: &RuneRuntime) -> Vec<RuneValue> {
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
