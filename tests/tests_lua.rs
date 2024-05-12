use bevy::prelude::*;
use bevy_scriptum::{
    lua_support::{LuaEngine, LuaRuntimeBuilder, LuaScript, LuaScriptData},
    prelude::*,
    AddScriptingRuntimeAppExt, CallFunction as _, Script, ScriptData, ScriptingRuntime,
};
use tracing_test::traced_test;

use crate::utils::{build_test_app, run_scripting_with, TimesCalled};

mod utils;

#[test]
#[traced_test]
fn test_lua_function_gets_called_from_rust() {
    let mut app = build_test_app();

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load("tests/lua/lua_function_gets_called_from_rust.lua");
    let entity_id = app.world.spawn(Script::<LuaScript>::new(asset)).id();

    run_scripting_with(&mut app, |app| {
        app.add_systems(Update, call_lua_on_update_from_rust);
    });

    let _script_data = app
        .world
        .get::<ScriptData<LuaScriptData>>(entity_id)
        .unwrap();
    // let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
    // assert_eq!(state["times_called"].clone_cast::<i64>(), 1);
}

#[test]
#[traced_test]
fn test_rust_function_gets_called_from_lua() {
    let mut app = build_test_app();

    app.world.init_resource::<TimesCalled>();

    app.add_scripting_runtime::<LuaRuntimeBuilder>(|r| {
        r.add_script_function(String::from("rust_func"), |mut res: ResMut<TimesCalled>| {
            res.times_called += 1;
        });
    });

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load("tests/lua/rust_function_gets_called_from_lua.lua");
    app.world.spawn(Script::<LuaScript>::new(asset));

    run_scripting_with(&mut app, |app| {
        app.add_systems(Update, call_lua_on_update_from_rust);
    });

    assert_eq!(
        app.world
            .get_resource::<TimesCalled>()
            .unwrap()
            .times_called,
        1
    );
}

fn call_lua_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut ScriptData<LuaScriptData>)>,
    mut scripting_runtime: ResMut<ScriptingRuntime<LuaEngine>>,
) {
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}
