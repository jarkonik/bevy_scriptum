use bevy::{asset::Asset, reflect::TypePath};
use serde::Deserialize;

use crate::assets::FileExtension;

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
