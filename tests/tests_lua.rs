use bevy::prelude::*;
use bevy_scriptum::{
    lua_support::{LuaEngine, LuaRuntimeBuilder, LuaScript, LuaScriptData, LuaScriptingRuntime},
    prelude::*,
    AddScriptingRuntimeAppExt, CallFunction as _, GetEngine, Script, ScriptData, ScriptingRuntime,
};
use tracing_test::traced_test;

use crate::utils::{run_scripting_with, TimesCalled};

mod utils;

pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()))
        .add_plugins(ScriptingPlugin::<LuaScriptingRuntime>::default());
    app.update();
    app
}

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
    let mut r = app.world.get_resource_mut::<LuaScriptingRuntime>().unwrap();
    let e = r.engine_mut(); // FIXME: Proivided non-mut api and use here
    let e = e.lock().unwrap();
    let state = e.globals().get::<_, mlua::Table>("state").unwrap();
    let times_called = state.get::<_, u8>("times_called").unwrap();
    assert_eq!(times_called, 1);
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
    mut scripting_runtime: ResMut<LuaScriptingRuntime>,
) {
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}
