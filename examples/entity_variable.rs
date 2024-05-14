use bevy::prelude::*;
use bevy_scriptum::{
    runtimes::rhai::{RhaiScript, RhaiScriptingRuntime},
    Script, ScriptingPluginBuilder,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPluginBuilder::<RhaiScriptingRuntime>::new().build())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/entity_variable.rhai"),
    ));
}
