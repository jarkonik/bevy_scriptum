use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RhaiRuntime>(|runtime| {
            runtime.add_function(String::from("hello_bevy"), hello_bevy_callback_system);
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/rhai/hello_world.rhai"),
    ));
}

fn hello_bevy_callback_system() {
    println!("hello bevy, called from script");
}
