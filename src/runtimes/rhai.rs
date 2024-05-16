use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, schedule::ScheduleLabel, system::Resource},
    math::Vec3,
    reflect::TypePath,
};
use rhai::{CallFnOptions, Dynamic, Engine, Scope, Variant};
use serde::Deserialize;

use crate::{
    assets::GetExtensions, promise::Promise, EngineMut, Runtime, ScriptingError, ENTITY_VAR_NAME,
};

/// A script that can be loaded by the [crate::ScriptingPlugin].
#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct RhaiScript(pub String);

impl GetExtensions for RhaiScript {
    fn extensions() -> &'static [&'static str] {
        &["rhai"]
    }
}

impl From<String> for RhaiScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Resource)]
pub struct RhaiScriptingRuntime {
    engine: rhai::Engine,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct RhaiSchedule;

/// A component that represents the data of a script. It stores the [rhai::Scope](basically the state of the script, any declared variable etc.)
/// and [rhai::AST] which is a cached AST representation of the script.
#[derive(Component)]
pub struct RhaiScriptData {
    pub scope: rhai::Scope<'static>,
    pub(crate) ast: rhai::AST,
}

impl EngineMut for RhaiScriptingRuntime {
    type Engine = rhai::Engine;

    fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }
}

#[derive(Clone)]
pub struct RhaiValue(rhai::Dynamic);

impl Runtime for RhaiScriptingRuntime {
    type Schedule = RhaiSchedule;
    type ScriptAsset = RhaiScript;
    type ScriptData = RhaiScriptData;
    type CallContext = rhai::NativeCallContextStore;
    type Value = RhaiValue;

    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: Entity,
    ) -> Result<Self::ScriptData, ScriptingError> {
        let mut scope = Scope::new();
        scope.push(ENTITY_VAR_NAME, entity);

        let engine = &self.engine;

        let ast = engine
            .compile_with_scope(&scope, script.0.as_str())
            .map_err(ScriptingError::CompileError)?;

        engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(ScriptingError::RuntimeError)?;

        scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();

        Ok(Self::ScriptData { ast, scope })
    }

    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<std::any::TypeId>,
        f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<Promise<Self::CallContext>, ScriptingError>
            + Send
            + Sync
            + 'static,
    ) -> Result<(), ScriptingError> {
        self.engine
            .register_raw_fn(name, arg_types, move |context, args| {
                let args = args.iter_mut().map(|arg| RhaiValue(arg.clone())).collect();
                let promise = f(context.store_data(), args).unwrap();
                Ok(promise)
            });
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: Entity,
        args: impl rhai::FuncArgs,
    ) -> Result<(), ScriptingError> {
        let ast = script_data.ast.clone();
        let scope = &mut script_data.scope;
        scope.push(ENTITY_VAR_NAME, entity);
        let options = CallFnOptions::new().eval_ast(false);
        let result = self
            .engine
            .call_fn_with_options::<Dynamic>(options, scope, &ast, name, args);
        scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
        if let Err(err) = result {
            match *err {
                rhai::EvalAltResult::ErrorFunctionNotFound(n, _) if n == name => {}
                e => Err(Box::new(e))?,
            }
        }
        Ok(())
    }
}

impl Default for RhaiScriptingRuntime {
    fn default() -> Self {
        let mut engine = Engine::new();

        engine
            .register_type_with_name::<Entity>("Entity")
            .register_fn("index", |entity: &mut Entity| entity.index());
        engine
            .register_type_with_name::<Promise<rhai::NativeCallContextStore>>("Promise")
            .register_fn("then", Promise::<rhai::NativeCallContextStore>::then);
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

        RhaiScriptingRuntime { engine }
    }
}

trait Sealed {}
impl<T: Variant + Sealed + Clone> From<T> for RhaiValue {
    fn from(value: T) -> Self {
        RhaiValue(Dynamic::from(value))
    }
}

impl From<()> for RhaiValue {
    fn from(value: ()) -> Self {
        RhaiValue(Dynamic::from(value))
    }
}
