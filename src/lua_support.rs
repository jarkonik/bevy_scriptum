use bevy::{asset::Asset, ecs::component::Component, reflect::TypePath};
use serde::Deserialize;

use crate::{assets::FileExtension, systems::CreateScriptData};

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

#[derive(Component)]
pub struct LuaScriptData {}

impl CreateScriptData for LuaScript {
    type ScriptData = LuaScriptData;
    type Engine = rhai::Engine;

    fn create_script_data(
        &self,
        entity: bevy::prelude::Entity,
        engine: &mut rhai::Engine,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        todo!()
    }
}
