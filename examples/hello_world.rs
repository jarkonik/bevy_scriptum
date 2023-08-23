use bevy::prelude::*;
use bevy_scriptum::{prelude::*, Script};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin::default())
        .add_script_runtime(bevy_scriptum::runtimes::rhai::Runtime::new())
        .add_script_function(String::from("hello_bevy"), || {
            println!("hello bevy, called from script");
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::new(assets_server.load("examples/hello_world.rhai")));
}
