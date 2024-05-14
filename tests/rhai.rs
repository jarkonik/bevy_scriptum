use bevy::prelude::*;
use bevy_scriptum::{
    runtimes::rhai::{RhaiScript, RhaiScriptData, RhaiScriptingRuntime},
    Script, ScriptingPluginBuilder, ScriptingRuntime,
};

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()))
        .add_plugins(ScriptingPluginBuilder::<RhaiScriptingRuntime>::new().build());
    app.update();
    app
}

#[test]
fn test_rhai_function_gets_called_from_rust() {
    let mut app = build_test_app();

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load::<RhaiScript>("tests/rhai_function_gets_called_from_rust.rhai");
    let entity_id = app.world.spawn(Script::new(asset)).id();

    run_scripting_with(&mut app, |app| {
        app.add_systems(Update, call_rhai_on_update_from_rust);
    });

    let script_data = app.world.get::<RhaiScriptData>(entity_id).unwrap();
    let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
    assert_eq!(state["times_called"].clone_cast::<i64>(), 1);
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RhaiScriptData)>,
    mut scripting_runtime: ResMut<ScriptingRuntime>,
) {
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}

fn run_scripting_with(app: &mut App, f: impl FnOnce(&mut App)) {
    app.update(); // Execute plugin internal systems
    f(app);
    app.update(); // Execute systems added by callback
}

#[test]
fn test_rust_function_gets_called_from_rhai() {
    let mut app = build_test_app();

    #[derive(Default, Resource)]
    struct TimesCalled {
        times_called: u8,
    }

    app.world.init_resource::<TimesCalled>();

    // app.add_script_function(String::from("rust_func"), |mut res: ResMut<TimesCalled>| {
    //     res.times_called += 1;
    // });

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load::<RhaiScript>("tests/rust_function_gets_called_from_rhai.rhai");
    app.world.spawn(Script::new(asset));

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
