use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use bevy::{asset::Asset, ecs::component::Component, reflect::TypePath};
use serde::Deserialize;

use crate::{
    assets::FileExtension, promise::Promise, systems::CreateScriptData, GetEngine, RegisterRawFn,
    ScriptingError, ScriptingRuntime,
};

/// A lua language script that can be loaded by the [crate::ScriptingPlugin].
#[derive(Asset, Debug, Deserialize, TypePath, Default)]
pub struct LuaScript(pub String);

impl From<String> for LuaScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl FileExtension for LuaScript {
    fn extension() -> &'static [&'static str] {
        &["lua"]
    }
}

impl RegisterRawFn<rhai::NativeCallContextStore> for ScriptingRuntime<LuaEngine> {
    fn register_raw_fn<'name, 'types>(
        &mut self,
        name: &'name str,
        arg_types: Vec<TypeId>,
        f: impl Fn() -> Promise<rhai::NativeCallContextStore>,
    ) {
        todo!()
    }
}

#[derive(Component, Debug)]
pub struct LuaScriptData {}

impl GetEngine<LuaEngine> for ScriptingRuntime<LuaEngine> {
    fn engine_mut(&mut self) -> &mut LuaEngine {
        &mut self.engine
    }
}

impl CreateScriptData<LuaEngine> for LuaScript {
    type ScriptData = LuaScriptData;

    fn create_script_data(
        &self,
        entity: bevy::prelude::Entity,
        engine: &mut LuaEngine,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        todo!()
    }
}

pub type LuaEngine = Arc<Mutex<mlua::Lua>>;

impl Default for ScriptingRuntime<LuaEngine> {
    fn default() -> Self {
        Self {
            engine: Arc::new(Mutex::new(mlua::Lua::new())),
        }
    }
}
