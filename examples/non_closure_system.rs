use bevy::prelude::*;
use bevy_scriptum::{
    runtimes::rhai::{RhaiScript, RhaiScriptingRuntime},
    Script, ScriptingPluginBuilder,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPluginBuilder::<RhaiScriptingRuntime>::new().build())
        // .add_script_function(String::from("hello_bevy"), hello_bevy_callback_system)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/hello_world.rhai"),
    ));
}

fn hello_bevy_callback_system() {
    println!("hello bevy, called from script");
}
