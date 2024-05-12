use bevy::prelude::*;
use bevy_scriptum::{
    prelude::*,
    rhai_support::{RhaiRuntime, RhaiRuntimeBuilder, RhaiScript},
    AddScriptingRuntimeAppExt, Script,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin::<RhaiRuntime>::default())
        .add_scripting_runtime::<RhaiRuntimeBuilder>(|r| {
            r.add_script_function(String::from("hello_bevy"), || {
                println!("hello bevy, called from script");
            });
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/hello_world.rhai"),
    ));
}
