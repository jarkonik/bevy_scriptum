use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use mlua::Lua;
use serde::Deserialize;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{
    assets::GetExtensions,
    callback::{CloneCast, IntoValue},
    EngineMut, Runtime,
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

impl Runtime for LuaRuntime {
    type Schedule = LuaSchedule;

    type ScriptAsset = LuaScript;

    type ScriptData = LuaScriptData;

    type CallContext = ();

    type Value = LuaValue;

    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        todo!()
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
            .create_function(|_, ()| {
                println!("ble");
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
        args: impl rhai::FuncArgs, // TODO: Remove rhai
    ) -> Result<(), crate::ScriptingError> {
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

impl<T: Any + Clone + Send + Sync> IntoValue<LuaValue> for T {
    fn into_value(self) -> LuaValue {
        LuaValue(())
    }
}

impl From<()> for LuaValue {
    fn from(value: ()) -> Self {
        LuaValue(())
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
