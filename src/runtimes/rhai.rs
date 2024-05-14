use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use rhai::Scope;
use serde::Deserialize;

use crate::{assets::GetExtensions, Runtime, ScriptingError, ENTITY_VAR_NAME};

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

impl Runtime for RhaiScriptingRuntime {
    type Schedule = RhaiSchedule;
    type ScriptAsset = RhaiScript;
    type ScriptData = RhaiScriptData;

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
}
