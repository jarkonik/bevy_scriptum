use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|builder| {
            builder.add_function(
                String::from("get_player_name"),
                |player_names: Query<&Name, With<Player>>| player_names.single().to_string(),
            );
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Player, Name::new("John")));
    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("examples/lua/promises.lua"),
    ));
}
