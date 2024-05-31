use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use mlua::{Function, IntoLua, Lua};
use serde::Deserialize;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{
    assets::GetExtensions,
    callback::{CloneCast, IntoValue},
    EngineMut, EngineRef, FuncArgs, Runtime,
};

type LuaEngine = Arc<Mutex<Lua>>;

#[derive(Clone)]
pub struct LuaValue(());

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

impl EngineMut for LuaRuntime {
    type Engine = LuaEngine;

    fn engine_mut(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}

impl EngineRef for LuaRuntime {
    type Engine = LuaEngine;

    fn engine_ref(&self) -> &Self::Engine {
        &self.engine
    }
}

impl Runtime for LuaRuntime {
    type Schedule = LuaSchedule;

    type ScriptAsset = LuaScript;

    type ScriptData = LuaScriptData;

    type CallContext = ();

    type Value = LuaValue;

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
        let engine = self.engine.lock().unwrap();
        let func = engine
            .create_function(move |_, ()| {
                f((), vec![]).unwrap();
                Ok(())
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
        let args: Vec<mlua::Value> = args.parse().into_iter().map(|a| mlua::Value::Nil).collect();
        let _ = func.call::<_, ()>(args);
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
}

impl<T: Any + Clone + Send + Sync> IntoValue<LuaRuntime> for T {
    fn into_value(self, runtime: &mut LuaRuntime) -> LuaValue {
        LuaValue(())
    }
}

impl From<()> for LuaValue {
    fn from(value: ()) -> Self {
        LuaValue(())
    }
}

impl FuncArgs<LuaValue> for () {
    fn parse(self) -> Vec<LuaValue> {
        Vec::new()
    }
}

impl<T: IntoLua<'static>> FuncArgs<LuaValue> for Vec<T> {
    fn parse(self) -> Vec<LuaValue> {
        self.into_iter().map(|_| LuaValue(())).collect()
    }
}

impl CloneCast for LuaValue {
    fn clone_cast<T: Clone + 'static>(&self) -> T {
        todo!();
    }
}

pub mod prelude {
    pub use super::{LuaRuntime, LuaScript, LuaScriptData};
}
