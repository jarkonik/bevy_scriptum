use bevy::prelude::*;

/// A component that represents a script.
#[derive(Component)]
pub struct Script<A: Asset> {
    pub script: Handle<A>,
}

impl<A: Asset> Script<A> {
    /// Create a new script component from a handle to a [RhaiScript] obtained using [AssetServer].
    pub fn new(script: Handle<A>) -> Self {
        Self { script }
    }
}
