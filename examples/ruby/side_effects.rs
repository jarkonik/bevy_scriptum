use bevy::{app::AppExit, prelude::*};
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn main() {
    App::new()
        // This is just needed for headless console app, not needed for a regular bevy application
        // that uses a winit window
        .set_runner(move |mut app: App| {
            loop {
                app.update();
                if let Some(exit) = app.should_exit() {
                    return exit;
                }
            }
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, print_entity_names_and_quit)
        .add_scripting::<RubyRuntime>(|runtime| {
            runtime.add_function(String::from("spawn_entity"), spawn_entity);
        })
        .run();
}

fn spawn_entity(mut commands: Commands) {
    commands.spawn(Name::new("SpawnedEntity"));
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Script::<RubyScript>::new(
        assets_server.load("examples/ruby/side_effects.rb"),
    ),));
}

fn print_entity_names_and_quit(query: Query<&Name>, mut exit: MessageWriter<AppExit>) {
    if !query.is_empty() {
        for e in &query {
            println!("{}", e);
        }
        exit.write(AppExit::Success);
    }
}
