use std::any::TypeId;

use bevy::{
    asset::{Asset, Handle},
    ecs::{component::Component, entity::Entity},
    math::Vec3,
    reflect::TypePath,
};
use rhai::{CallFnOptions, Dynamic, FuncArgs, Scope};
use serde::Deserialize;

use crate::{
    assets::FileExtension, promise::Promise, systems::CreateScriptData, CallFunction, GetEngine,
    RegisterRawFn, Script, ScriptData, ScriptingError, ScriptingRuntime, ENTITY_VAR_NAME,
};

/// A rhai language script that can be loaded by the [crate::ScriptingPlugin].
#[derive(Asset, Debug, Deserialize, TypePath, Default)]
pub struct RhaiScript(pub String);

impl From<String> for RhaiScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl FileExtension for RhaiScript {
    fn extension() -> &'static [&'static str] {
        &["rhai"]
    }
}

#[derive(Debug)]
pub struct RhaiScriptData {
    pub scope: rhai::Scope<'static>,
    pub(crate) ast: rhai::AST,
}

impl GetEngine<rhai::Engine> for ScriptingRuntime<rhai::Engine> {
    fn engine_mut(&mut self) -> &mut rhai::Engine {
        &mut self.engine
    }
}

impl CreateScriptData<rhai::Engine> for RhaiScript {
    type ScriptData = RhaiScriptData;

    fn create_script_data(
        &self,
        entity: Entity,
        engine: &mut rhai::Engine,
    ) -> Result<Self::ScriptData, ScriptingError> {
        let mut scope = Scope::new();

        scope.push(ENTITY_VAR_NAME, entity);

        let ast = engine
            .compile_with_scope(&scope, &self.0)
            .map_err(ScriptingError::CompileError)?;

        engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(ScriptingError::RuntimeError)?;

        scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();

        Ok(Self::ScriptData { ast, scope })
    }
}

impl<D: Send + Clone + 'static, C: Clone + 'static> RegisterRawFn<D, C>
    for ScriptingRuntime<rhai::Engine>
{
    fn register_raw_fn(
        &mut self,
        name: &str,
        arg_types: Vec<TypeId>,
        f: impl Fn() -> Promise<D, C> + Sync + Send + 'static,
    ) {
        self.engine
            .register_raw_fn(name, arg_types, move |_context, _args| {
                let result = f();
                Ok(result)
            });
    }
}

impl CallFunction<RhaiScriptData> for ScriptingRuntime<rhai::Engine> {
    /// Get a  mutable reference to the internal [rhai::Engine].

    /// Call a function that is available in the scope of the script.
    fn call_fn(
        &mut self,
        function_name: &str,
        script_data: &mut ScriptData<RhaiScriptData>,
        entity: Entity,
        args: (), // args: impl FuncArgs,
    ) -> Result<(), ScriptingError> {
        // let script_data = &mut script_data.data;
        //
        // let ast = script_data.ast.clone();
        // let scope = &mut script_data.scope;
        // scope.push(ENTITY_VAR_NAME, entity);
        // let options = CallFnOptions::new().eval_ast(false);
        // let result =
        //     self.engine
        //         .call_fn_with_options::<Dynamic>(options, scope, &ast, function_name, args);
        // scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
        // if let Err(err) = result {
        //     match *err {
        //         rhai::EvalAltResult::ErrorFunctionNotFound(name, _) if name == function_name => {}
        //         e => Err(Box::new(e))?,
        //     }
        // }
        Ok(())
    }
}

impl Default for ScriptingRuntime<rhai::Engine> {
    fn default() -> Self {
        let mut engine = rhai::Engine::default();

        engine
            .register_type_with_name::<Entity>("Entity")
            .register_fn("index", |entity: &mut Entity| entity.index());
        engine
            .register_type_with_name::<Promise<(), RhaiCallback>>("Promise")
            .register_fn("then", Promise::<(), RhaiCallback>::then);
        engine
            .register_type_with_name::<Vec3>("Vec3")
            .register_fn("new_vec3", |x: f64, y: f64, z: f64| {
                Vec3::new(x as f32, y as f32, z as f32)
            })
            .register_get("x", |vec: &mut Vec3| vec.x as f64)
            .register_get("y", |vec: &mut Vec3| vec.y as f64)
            .register_get("z", |vec: &mut Vec3| vec.z as f64);
        #[allow(deprecated)]
        engine.on_def_var(|_, info, _| Ok(info.name != "entity"));

        Self { engine }
    }
}

impl Script<RhaiScript> {
    /// Create a new script component from a handle to a [RhaiScript] obtained using [AssetServer].
    pub fn new(script: Handle<RhaiScript>) -> Self {
        Self { script }
    }
}

#[derive(Debug, Clone)]
pub struct RhaiCallback;
