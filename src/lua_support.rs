use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use bevy::{
    asset::{Asset, Handle},
    ecs::component::Component,
    reflect::TypePath,
};
use serde::Deserialize;

use crate::{
    assets::FileExtension, promise::Promise, systems::CreateScriptData, GetEngine, RegisterRawFn,
    Script, ScriptingRuntime,
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
    fn register_raw_fn(
        &mut self,
        _name: &str,
        _arg_types: Vec<TypeId>,
        f: impl Fn() -> Promise<rhai::NativeCallContextStore>,
    ) {
        let engine = self.engine.lock().expect("Could not lock engine mutex");
        engine.create_function(|context, args: ()| {
            // let result = f();
            Ok(())
        });
    }
}

#[derive(Debug)]
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
        _entity: bevy::prelude::Entity,
        _engine: &mut LuaEngine,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        Ok(LuaScriptData {})
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

impl Script<LuaScript> {
    /// Create a new script component from a handle to a [LuaScript] obtained using [AssetServer].
    pub fn new(script: Handle<LuaScript>) -> Self {
        Self { script }
    }
}
