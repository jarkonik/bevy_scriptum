use bevy::prelude::*;
use bevy_scriptum::{prelude::*, Script, ScriptingRuntime};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin::default())
        .add_script_function(String::from("hello_bevy"), || {
            println!("hello bevy, called from script");
        })
        .add_systems(Startup, startup)
        .run();
}

#[derive(Clone)]
struct MyType {
    my_field: u32,
}

fn startup(
    mut commands: Commands,
    mut scripting_runtime: ResMut<ScriptingRuntime>,
    assets_server: Res<AssetServer>,
) {
    let engine = scripting_runtime.engine_mut();

    engine
        .register_type_with_name::<MyType>("MyType")
        // Register a method on MyType
        .register_fn("my_method", |my_type_instance: &mut MyType| {
            my_type_instance.my_field
        })
        // Register a "constructor" for MyType
        .register_fn("new_my_type", || MyType { my_field: 42 });

    commands.spawn(Script::new(assets_server.load("examples/custom_type.rhai")));
}
