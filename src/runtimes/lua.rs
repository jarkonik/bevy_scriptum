use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use mlua::{FromLua, Function, IntoLua, IntoLuaMulti, Lua, UserData, Variadic};
use serde::Deserialize;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    promise::Promise,
    FuncArgs, Runtime,
};

type LuaEngine = Arc<Mutex<Lua>>;

#[derive(Clone)]
pub struct LuaValue<'a>(mlua::Value<'a>);

// FIXME: Need to be wrapper in mutex
// TODO: https://github.com/mlua-rs/mlua/issues/120
unsafe impl Send for LuaValue<'_> {}
unsafe impl Sync for LuaValue<'_> {}

#[derive(Default, Resource)]
pub struct LuaRuntime {
    engine: LuaEngine,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct LuaSchedule;

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct LuaScript(pub String);

impl GetExtensions for LuaScript {
    fn extensions() -> &'static [&'static str] {
        &["lua"]
    }
}

impl From<String> for LuaScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Component)]
pub struct LuaScriptData;

impl Runtime for LuaRuntime {
    type Schedule = LuaSchedule;

    type ScriptAsset = LuaScript;

    type ScriptData = LuaScriptData;

    type CallContext = ();

    type Value = LuaValue<'static>;

    type RawEngine = Lua;

    // TODO: Should be renamed or even split as it also evals
    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        engine.load(&script.0).exec().unwrap();
        Ok(LuaScriptData)
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
        let engine_closure = self.engine.clone();
        let engine = self.engine.lock().unwrap();
        let func = engine
            .create_function(move |engine, args: Variadic<mlua::Value>| {
                let args = {
                    args.into_iter()
                        .map(|x| LuaValue::into_runtime_value_with_engine(x, engine))
                        .collect()
                };
                Ok(f((), args).unwrap())
            })
            .unwrap();
        engine.globals().set(name, func).unwrap();
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl FuncArgs<Self::Value>,
    ) -> Result<(), crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        let func = engine.globals().get::<_, Function>(name).unwrap();
        let args: Vec<mlua::Value> = args.parse().into_iter().map(|a| a.0).collect();
        func.call::<_, ()>(args).unwrap();
        Ok(())
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        todo!()
    }

    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        let mut engine = self.engine.lock().unwrap();
        f(&mut engine)
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        let engine = self.engine.lock().unwrap();
        f(&engine)
    }
}

impl<'a> IntoRuntimeValueWithEngine<'a, (), LuaRuntime> for () {
    fn into_runtime_value_with_engine(
        _value: (),
        _runtime: &Lua,
    ) -> <LuaRuntime as Runtime>::Value {
        LuaValue(mlua::Value::Nil)
    }
}

impl<'a, T: IntoLua<'a>> IntoRuntimeValueWithEngine<'a, T, LuaRuntime> for LuaValue<'a> {
    fn into_runtime_value_with_engine(value: T, engine: &'a Lua) -> LuaValue<'static> {
        let e = value.into_lua(engine).unwrap();
        match e {
            mlua::Value::Number(n) => LuaValue(mlua::Value::Number(n)),
            mlua::Value::Integer(n) => LuaValue(mlua::Value::Integer(n)),
            _ => todo!(),
        }
    }
}

impl<'a, T: FromLua<'a>> FromRuntimeValueWithEngine<'a, LuaRuntime> for T {
    fn from_runtime_value_with_engine(value: LuaValue<'a>, engine: &'a Lua) -> Self {
        T::from_lua(value.0, engine).unwrap()
    }
}

impl From<()> for LuaValue<'_> {
    fn from(_value: ()) -> Self {
        LuaValue(mlua::Value::Nil)
    }
}

impl<'a> FuncArgs<LuaValue<'a>> for () {
    fn parse(self) -> Vec<LuaValue<'a>> {
        Vec::new()
    }
}

impl<'a, T: IntoLua<'static>> FuncArgs<LuaValue<'a>> for Vec<T> {
    fn parse(self) -> Vec<LuaValue<'a>> {
        self.into_iter()
            .map(|_| LuaValue(mlua::Value::Nil))
            .collect()
    }
}

impl UserData for Promise<(), LuaValue<'static>> {}

pub mod prelude {
    pub use super::{LuaRuntime, LuaScript, LuaScriptData};
}
