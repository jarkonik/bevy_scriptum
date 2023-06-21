use bevy::{prelude::*};
use bevy_scriptum::{prelude::*, Script};
use rhai::ImmutableString;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScriptingPlugin::default())
        .add_script_function(String::from("fun_without_params"), || {
            println!("called without params");
        })
        .add_script_function(
            String::from("fun_with_string_param"),
            |In((x,)): In<(ImmutableString,)>| {
                println!("called with string: '{}'", x);
            },
        )
        .add_script_function(
            String::from("fun_with_i64_param"),
            |In((x,)): In<(i64,)>| {
                println!("called with i64: {}", x);
            },
        )
        .add_script_function(
            String::from("fun_with_multiple_params"),
            |In((x, y)): In<(i64, ImmutableString)>| {
                println!("called with i64: {} and string: '{}'", x, y);
            },
        )
        .add_script_function(
            String::from("fun_with_i64_and_array_param"),
            |In((x, y)): In<(i64, rhai::Array)>| {
                println!(
                    "called with i64: {} and dynamically typed array: '{:?}'",
                    x, y
                );
            },
        )
        .add_startup_system(startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::new(
        assets_server.load("examples/function_params.rhai"),
    ));
}
