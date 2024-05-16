use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RhaiRuntime>(|runtime| {
            runtime.add_function(String::from("fun_without_params"), || {
                println!("called without params");
            });
            // .add_function(
            //     String::from("fun_with_string_param"),
            //     |In((x,)): In<(ImmutableString,)>| {
            //         println!("called with string: '{}'", x);
            //     },
            // )
            // .add_function(
            //     String::from("fun_with_i64_param"),
            //     |In((x,)): In<(i64,)>| {
            //         println!("called with i64: {}", x);
            //     },
            // )
            // .add_function(
            //     String::from("fun_with_multiple_params"),
            //     |In((x, y)): In<(i64, ImmutableString)>| {
            //         println!("called with i64: {} and string: '{}'", x, y);
            //     },
            // )
            // .add_function(
            //     String::from("fun_with_i64_and_array_param"),
            //     |In((x, y)): In<(i64, rhai::Array)>| {
            //         println!(
            //             "called with i64: {} and dynamically typed array: '{:?}'",
            //             x, y
            //         );
            //     },
            // );
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/rhai/function_params.rhai"),
    ));
}
