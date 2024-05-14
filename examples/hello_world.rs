use bevy::prelude::*;
use bevy_scriptum::{runtimes::rhai::RhaiScriptingRuntime, Script, ScriptingPluginBuilder};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPluginBuilder::<RhaiScriptingRuntime>::new().build())
        // .add_script_function(String::from("hello_bevy"), || {
        //     println!("hello bevy, called from script");
        // })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::new(assets_server.load("examples/hello_world.rhai")));
}
