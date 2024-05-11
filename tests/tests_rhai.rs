use bevy::prelude::*;
use bevy_scriptum::{
    prelude::*,
    rhai_support::{RhaiScript, RhaiScriptData},
    Script, ScriptData, ScriptingRuntime,
};
use tracing_test::traced_test;

use crate::utils::{build_test_app, run_scripting_with, TimesCalled};

mod utils;

#[test]
#[traced_test]
fn test_rhai_function_gets_called_from_rust() {
    let mut app = build_test_app();

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load("tests/rhai/rhai_function_gets_called_from_rust.rhai");
    let entity_id = app.world.spawn(Script::<RhaiScript>::new(asset)).id();

    run_scripting_with(&mut app, |app| {
        app.add_systems(Update, call_rhai_on_update_from_rust);
    });

    let script_data = app
        .world
        .get::<ScriptData<RhaiScriptData>>(entity_id)
        .unwrap();
    let state = script_data
        .data
        .scope
        .get_value::<rhai::Map>("state")
        .unwrap();
    assert_eq!(state["times_called"].clone_cast::<i64>(), 1);
}

#[test]
#[traced_test]
fn test_rust_function_gets_called_from_rhai() {
    let mut app = build_test_app();

    app.world.init_resource::<TimesCalled>();

    app.add_script_function(String::from("rust_func"), |mut res: ResMut<TimesCalled>| {
        res.times_called += 1;
    });

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load("tests/rhai/rust_function_gets_called_from_rhai.rhai");
    app.world.spawn(Script::<RhaiScript>::new(asset));

    run_scripting_with(&mut app, |app| {
        app.add_systems(Update, call_rhai_on_update_from_rust);
    });

    assert_eq!(
        app.world
            .get_resource::<TimesCalled>()
            .unwrap()
            .times_called,
        1
    );
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut ScriptData<RhaiScriptData>)>,
    mut scripting_runtime: ResMut<ScriptingRuntime<rhai::Engine>>,
) {
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}
