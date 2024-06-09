use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;
use mlua::{UserData, UserDataMethods};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime.add_function(String::from("hello_bevy"), || {
                println!("hello bevy, called from script");
            });
        })
        .add_systems(Startup, startup)
        .run();
}

#[derive(Clone)]
struct MyType {
    my_field: u32,
}

impl UserData for MyType {}

fn startup(
    mut commands: Commands,
    mut scripting_runtime: ResMut<LuaRuntime>,
    assets_server: Res<AssetServer>,
) {
    scripting_runtime.with_engine_mut(|engine| {
        engine
            .register_userdata_type::<MyType>(|typ| {
                // Register a method on MyType
                typ.add_method("my_method", |_, my_type_instance: &MyType, ()| {
                    Ok(my_type_instance.my_field)
                })
            })
            .unwrap();

        // Register a "constructor" for MyType
        let my_type_constructor = engine
            .create_function(|_, ()| Ok(MyType { my_field: 42 }))
            .unwrap();
        engine.globals().set("MyType", my_type_constructor).unwrap();
    });

    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("examples/lua/custom_type.lua"),
    ));
}
