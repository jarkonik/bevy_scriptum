use bevy::prelude::*;

/// A component that represents a script.
#[derive(Component)]
pub struct Script<A: Asset> {
    pub script: Handle<A>,
}

/// A component that represents the data of a script. It stores the [rhai::Scope](basically the state of the script, any declared variable etc.)
/// and [rhai::AST] which is a cached AST representation of the script.
#[derive(Component)]
pub struct ScriptData {
    pub scope: rhai::Scope<'static>,
    pub(crate) ast: rhai::AST,
}

impl<A: Asset> Script<A> {
    /// Create a new script component from a handle to a [RhaiScript] obtained using [AssetServer].
    pub fn new(script: Handle<A>) -> Self {
        Self { script }
    }
}
