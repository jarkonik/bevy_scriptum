use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity},
    reflect::TypePath,
};
use rhai::Scope;
use serde::Deserialize;

use crate::{assets::FileExtension, systems::CreateScriptData, ScriptingError, ENTITY_VAR_NAME};

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

#[derive(Component)]
pub struct RhaiScriptData {
    pub scope: rhai::Scope<'static>,
    pub(crate) ast: rhai::AST,
}

impl CreateScriptData for RhaiScript {
    type ScriptData = RhaiScriptData;
    type Engine = rhai::Engine;

    fn create_script_data(
        &self,
        entity: Entity,
        engine: &mut Self::Engine,
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
