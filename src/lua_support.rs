use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use bevy::{
    asset::{Asset, Handle},
    ecs::{entity::Entity},
    reflect::TypePath,
};
use serde::Deserialize;

use crate::{
    assets::FileExtension, promise::Promise, systems::CreateScriptData, CallFunction, GetEngine,
    RegisterRawFn, Script, ScriptData, ScriptingError, ScriptingRuntime,
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

impl<D, C> RegisterRawFn<D, C> for ScriptingRuntime<LuaEngine> {
    fn register_raw_fn(
        &mut self,
        name: &str,
        _arg_types: Vec<TypeId>,
        f: impl Fn() -> Promise<D, C> + Sync + Send + 'static,
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
        engine: &mut LuaEngine,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
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
        let engine = engine.lock().expect("Could not lock engine");
        engine.load(&self.0).exec().expect("Error runnning script");

        Ok(Self::ScriptData {})
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

impl CallFunction<LuaScriptData> for ScriptingRuntime<LuaEngine> {
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
        // engine
        //     .globals()
        //     .get::<>("function_name")
        //     .expect("Function not found");

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LuaCallback;
