use bevy::prelude::*;
use bevy_scriptum::{prelude::*, Script};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::new(
        assets_server.load("examples/entity_variable.rhai"),
    ));
}
