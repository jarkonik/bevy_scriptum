use bevy::{asset::Asset, reflect::TypePath};
use serde::Deserialize;

use crate::assets::FileExtension;

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
