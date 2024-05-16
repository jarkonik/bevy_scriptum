use bevy::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;
use bevy_scriptum::{prelude::*, BuildScriptingRuntime};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RhaiRuntime>(|_| {})
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/rhai/entity_variable.rhai"),
    ));
}
