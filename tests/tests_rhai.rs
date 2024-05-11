use bevy::prelude::*;
use bevy_scriptum::{prelude::*, Script, ScriptData, ScriptingRuntime};

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins).add_plugins(ScriptingPlugin);
    app.update();
    app
}

#[test]
fn test_rhai_function_gets_called_from_rust() {
    let mut app = build_test_app();

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load("tests/rhai_function_gets_called_from_rust.rhai");
    let entity_id = app.world.spawn(Script::new(asset)).id();

    app.update(); // Execute plugin internal systems

    app.add_systems(Update, call_rhai_on_update_from_rust);

    app.update(); // Execute test tystem

    let script_data = app.world.get::<ScriptData>(entity_id).unwrap();
    let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
    assert_eq!(state["times_called"].clone_cast::<i64>(), 1);
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut ScriptData)>,
    mut scripting_runtime: ResMut<ScriptingRuntime>,
) {
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}
