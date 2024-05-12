use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use bevy::{
    asset::{Asset, Handle},
    ecs::{
        component::Component, entity::Entity, schedule::ScheduleLabel, system::Resource,
        world::World,
    },
    reflect::TypePath,
};
use mlua::Lua;

use serde::Deserialize;

use crate::{
    assets::FileExtension, promise::Promise, systems::CreateScriptData, BuildScriptingRuntime,
    CallFunction, GetEngine, RegisterRawFn, RuntimeConfig, Script, ScriptData, ScriptingError,
    ScriptingRuntime, ScriptingRuntimeBuilder,
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

impl RegisterRawFn for LuaScriptingRuntime {
    fn register_raw_fn(
        &mut self,
        name: &str,
        _arg_types: Vec<TypeId>,
        f: impl Fn() -> Promise<(), ()> + Sync + Send + 'static,
    ) {
        let engine = self.engine.lock().expect("Could not lock engine mutex");
        let fun = engine
            .create_function(move |_, _args: ()| {
                let _result = f();
                Ok(())
            })
            .expect("Error creating function");

        engine
            .globals()
            .set(name, fun)
            .expect("Error setting function");
    }
}

#[derive(Component, Debug)]
pub struct LuaScriptData;

#[derive(Resource, Debug)]
pub struct LuaScriptingRuntime {
    engine: LuaEngine,
}

impl GetEngine for LuaScriptingRuntime {
    type Engine = LuaEngine;
    fn engine_mut(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}

impl CreateScriptData<LuaScript, LuaScriptData> for LuaScriptingRuntime {
    fn create_script_data(
        &self,
        _entity: bevy::prelude::Entity,
        script: &LuaScript,
    ) -> Result<ScriptData<LuaScriptData>, crate::ScriptingError> {
        //
        // let mut scope = Scope::new();
        //
        // scope.push(ENTITY_VAR_NAME, entity);
        //
        // let ast = engine
        //     .compile_with_scope(&scope, &self.0)
        //     .map_err(ScriptingError::CompileError)?;
        //
        // engine
        //     .run_ast_with_scope(&mut scope, &ast)
        //     .map_err(ScriptingError::RuntimeError)?;
        //
        // scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
        //
        // Ok(Self::ScriptData { ast, scope })
        //
        let engine = self.engine.lock().expect("Could not lock engine");
        engine
            .load(&script.0)
            .exec()
            .expect("Error runnning script");

        Ok(ScriptData {
            data: LuaScriptData,
        })
    }
}

pub type LuaEngine = Arc<Mutex<mlua::Lua>>;

impl Default for LuaScriptingRuntime {
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

impl CallFunction<LuaScriptData> for LuaScriptingRuntime {
    /// Get a  mutable reference to the internal [rhai::Engine].

    /// Call a function that is available in the scope of the script.
    fn call_fn(
        &mut self,
        function_name: &str,
        _script_data: &mut ScriptData<LuaScriptData>,
        _entity: Entity,
        _args: (), // args: impl FuncArgs,
    ) -> Result<(), ScriptingError> {
        let engine = self.engine.lock().expect("Could not lock engine");
        engine
            .load(format!("{function_name}()"))
            .exec()
            .expect("Error calling function");

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LuaCallback;

pub type LuaRuntimeBuilder = ScriptingRuntimeBuilder<ScriptingRuntime<LuaEngine>>;

impl BuildScriptingRuntime for ScriptingRuntimeBuilder<ScriptingRuntime<LuaEngine>> {
    type Callbacks = ();
    type Runtime = ScriptingRuntime<LuaEngine>;

    fn build(self) -> (World, Self::Runtime) {
        (
            self.world.expect("no world"),
            ScriptingRuntime {
                engine: Arc::new(Mutex::new(Lua::new())),
            },
        )
    }
}

#[derive(Default, ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaSchedule;

impl RuntimeConfig for LuaScriptingRuntime {
    type ScriptAsset = LuaScript;
    type Schedule = LuaSchedule;
    type Runtime = Self;
    type ScriptData = LuaScriptData;
}
