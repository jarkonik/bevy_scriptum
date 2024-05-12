use bevy::prelude::*;
use bevy_scriptum::{
    prelude::*,
    rhai_support::{RhaiScript, RhaiScriptingRuntime},
    AddScriptingRuntimeAppExt, Script,
};

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin::<RhaiScriptingRuntime>::default())
        // .add_scripting_runtime::<RhaiRuntimeBuilder>(|r| {
        //     r.add_script_function(
        //         String::from("print_player_names"),
        //         |players: Query<&Name, With<Player>>| {
        //             for player in &players {
        //                 println!("player name: {}", player);
        //             }
        //         },
        //     );
        // })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Player, Name::new("John")));
    commands.spawn((Player, Name::new("Mary")));
    commands.spawn((Player, Name::new("Alice")));

    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/ecs.rhai"),
    ));
}
