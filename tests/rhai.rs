use bevy::{ecs::system::RunSystemOnce as _, prelude::*};
use bevy_scriptum::runtimes::rhai::prelude::*;
use bevy_scriptum::{prelude::*, BuildScriptingRuntime};

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()));
    app
}

#[test]
fn test_rhai_function_gets_called_from_rust() {
    let mut app = build_test_app();

    app.add_scripting::<RhaiRuntime>(|_| {});

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset =
        asset_server.load::<RhaiScript>("tests/rhai/rhai_function_gets_called_from_rust.rhai");

    let entity_id = app.world.spawn(Script::new(asset)).id();
    app.update(); // let `ScriptData` resources be added to entities
    app.world.run_system_once(call_rhai_on_update_from_rust);

    let script_data = app.world.get::<RhaiScriptData>(entity_id).unwrap();
    let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
    assert_eq!(state["times_called"].clone_cast::<i64>(), 1);
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RhaiScriptData)>,
    scripting_runtime: ResMut<RhaiRuntime>,
) {
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}

#[test]
fn test_rust_function_gets_called_from_rhai() {
    let mut app = build_test_app();

    #[derive(Default, Resource)]
    struct TimesCalled {
        times_called: u8,
    }

    app.world.init_resource::<TimesCalled>();

    app.add_scripting::<RhaiRuntime>(|runtime| {
        runtime.add_function(String::from("rust_func"), |mut res: ResMut<TimesCalled>| {
            res.times_called += 1;
        });
    });

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset =
        asset_server.load::<RhaiScript>("tests/rhai/rust_function_gets_called_from_rhai.rhai");

    app.world.spawn(Script::new(asset));
    app.update(); // let `ScriptData` resources be added to entities
    app.world.run_system_once(call_rhai_on_update_from_rust);
    app.update(); // let callbacks be executed

    assert_eq!(
        app.world
            .get_resource::<TimesCalled>()
            .unwrap()
            .times_called,
        1
    );
}

#[test]
fn test_rust_function_with_int_param_gets_called_from_rhai() {
    let mut app = build_test_app();

    #[derive(Default, Resource)]
    struct IntResource {
        my_int: i64,
    }

    app.world.init_resource::<IntResource>();

    app.add_scripting::<RhaiRuntime>(|runtime| {
        runtime.add_function(
            String::from("rust_func"),
            |In((x,)): In<(i64,)>, mut res: ResMut<IntResource>| {
                res.my_int = x;
            },
        );
    });

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server
        .load::<RhaiScript>("tests/rhai/rust_function_gets_called_from_rhai_with_param.rhai");

    app.world.spawn(Script::new(asset));
    app.update(); // let `ScriptData` resources be added to entities
    app.world.run_system_once(call_rhai_on_update_from_rust);
    app.update(); // let callbacks be executed

    assert_eq!(app.world.get_resource::<IntResource>().unwrap().my_int, 5);
}