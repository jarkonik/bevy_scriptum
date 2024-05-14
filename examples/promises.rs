use bevy::prelude::*;
use bevy_scriptum::{
    runtimes::rhai::{RhaiScript, RhaiScriptingRuntime},
    Script, ScriptingPluginBuilder,
};

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPluginBuilder::<RhaiScriptingRuntime>::new().build())
        // .add_script_function(
        //     String::from("get_player_name"),
        //     |player_names: Query<&Name, With<Player>>| player_names.single().to_string(),
        // )
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Player, Name::new("John")));
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/promises.rhai"),
    ));
}
