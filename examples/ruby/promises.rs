use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|builder| {
            builder.add_function(
                String::from("get_player_name"),
                |player_names: Query<&Name, With<Player>>| {
                    player_names
                        .single()
                        .expect("Missing player_names")
                        .to_string()
                },
            );
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Player, Name::new("John")));
    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/ruby/promises.rb"),
    ));
}
