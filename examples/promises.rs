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
        .add_plugins(ScriptingPlugin::<RhaiScriptingRuntime>::build())
        // .add_scripting_runtime::<RhaiRuntimeBuilder>(|r| {
        //     r.add_script_function(
        //         String::from("get_player_name"),
        //         |player_names: Query<&Name, With<Player>>| player_names.single().to_string(),
        //     );
        // })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Player, Name::new("John")));
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/promises.rhai"),
    ));
}
