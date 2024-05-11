use bevy::prelude::*;

use super::rhai_support::RhaiScript;

/// A component that represents a script.
#[derive(Component)]
pub struct Script<T: Asset> {
    pub script: Handle<T>,
}

/// A component that represents the data of a script. It stores the [rhai::Scope](basically the state of the script, any declared variable etc.)
/// and [rhai::AST] which is a cached AST representation of the script.
#[derive(Component)]
pub struct ScriptData<T> {
    pub data: T,
}

impl Script<RhaiScript> {
    /// Create a new script component from a handle to a [RhaiScript] obtained using [AssetServer].
    pub fn new(script: Handle<RhaiScript>) -> Self {
        Self { script }
    }
}
