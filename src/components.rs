use bevy::prelude::*;

/// A component that represents a script.
#[derive(Component, Debug)]
pub struct Script<T: Asset> {
    pub script: Handle<T>,
}

/// A component that represents the data of a script. It stores the [rhai::Scope](basically the state of the script, any declared variable etc.)
/// and [rhai::AST] which is a cached AST representation of the script.
#[derive(Component)]
pub struct ScriptData<T> {
    pub data: T,
}
