use bevy::{app::AppExit, prelude::*};
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, call_lua_on_update_from_rust)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime.add_function(String::from("quit"), |mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            });
        })
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("examples/lua/function_return_value.lua"),
    ));
}

fn call_lua_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
    scripting_runtime: ResMut<LuaRuntime>,
    mut exit: MessageWriter<AppExit>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        let val = scripting_runtime
            .call_fn("get_value", &mut script_data, entity, ())
            .unwrap()
            .0;
        scripting_runtime.with_engine(|engine| {
            println!(
                "script returned: {}",
                engine.registry_value::<mlua::Integer>(&val).unwrap()
            );
        });
        exit.write(AppExit::Success);
    }
}
