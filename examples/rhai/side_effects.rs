use bevy::{app::AppExit, prelude::*};
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        // This is just needed for headless console app, not needed for a regular bevy game
        // that uses a winit window
        .set_runner(move |mut app: App| loop {
            app.update();
            if let Some(exit) = app.should_exit() {
                return exit;
            }
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, print_entity_names_and_quit)
        .add_scripting::<RhaiRuntime>(|runtime| {
            runtime.add_function(String::from("spawn_entity"), spawn_entity);
        })
        .run();
}

fn spawn_entity(mut commands: Commands) {
    commands.spawn(Name::new("SpawnedEntity"));
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((Script::<RhaiScript>::new(
        assets_server.load("examples/rhai/side_effects.rhai"),
    ),));
}

fn print_entity_names_and_quit(query: Query<&Name>, mut exit: EventWriter<AppExit>) {
    if !query.is_empty() {
        for e in &query {
            println!("{}", e);
        }
        exit.send(AppExit::Success);
    }
}
