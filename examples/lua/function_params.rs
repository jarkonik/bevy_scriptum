use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime
                .add_function(String::from("fun_without_params"), || {
                    println!("called without params");
                })
                .add_function(
                    String::from("fun_with_string_param"),
                    |In((x,)): In<(String,)>| {
                        println!("called with string: '{}'", x);
                    },
                )
                .add_function(
                    String::from("fun_with_i64_param"),
                    |In((x,)): In<(i64,)>| {
                        println!("called with i64: {}", x);
                    },
                )
                .add_function(
                    String::from("fun_with_multiple_params"),
                    |In((x, y)): In<(i64, String)>| {
                        println!("called with i64: {} and string: '{}'", x, y);
                    },
                )
                .add_function(
                    String::from("fun_with_i64_and_array_param"),
                    |In((x, y)): In<(i64, mlua::RegistryKey)>, runtime: Res<LuaRuntime>| {
                        runtime.with_engine(|engine| {
                            println!(
                                "called with i64: {} and dynamically typed array: [{:?}]",
                                x,
                                engine
                                    .registry_value::<mlua::Table>(&y)
                                    .unwrap()
                                    .pairs::<usize, mlua::Value>()
                                    .map(|pair| pair.unwrap())
                                    .map(|(_, v)| format!("{:?}", v))
                                    .collect::<Vec<String>>()
                                    .join(",")
                            );
                        });
                    },
                );
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("examples/lua/function_params.lua"),
    ));
}
