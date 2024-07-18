use std::sync::OnceLock;

use bevy::ecs::system::RunSystemOnce as _;
use bevy::prelude::*;
use bevy_scriptum::{prelude::*, FuncArgs, Runtime};
use mlua::Table;

static TRACING_SUBSCRIBER: OnceLock<()> = OnceLock::new();

fn build_test_app() -> App {
    let mut app = App::new();

    if std::env::var("RUST_TEST_LOG").is_ok() {
        TRACING_SUBSCRIBER.get_or_init(|| {
            tracing_subscriber::fmt().init();
        });
    }

    app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()));
    app
}

fn run_script<R: Runtime + Prepare, Out, Marker>(
    app: &mut App,
    path: String,
    system: impl IntoSystem<(), Out, Marker>,
) -> Entity {
    R::prepare(&mut app.world);

    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load::<R::ScriptAsset>(path);

    let entity_id = app.world.spawn(Script::new(asset)).id();
    app.update(); // let `ScriptData` resources be added to entities
    app.world.run_system_once(system);
    app.update(); // let callbacks be executed

    entity_id
}

fn call_script_on_update_from_rust<R: Runtime>(
    mut scripted_entities: Query<(Entity, &mut R::ScriptData)>,
    scripting_runtime: ResMut<R>,
) where
    (): for<'a> FuncArgs<'a, R::Value, R>,
{
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}

trait AssertStateKeyValue {
    type ScriptData;
    fn assert_state_key_value_integer(world: &World, entity_id: Entity, key: &str, value: i64);
    fn assert_state_key_value_string(world: &World, entity_id: Entity, key: &str, value: &str);
}

trait Prepare {
    fn prepare(_world: &mut World) {}
}

macro_rules! scripting_tests {
    ($runtime:ty, $script:literal, $extension:literal) => {
        use super::*;

        #[test]
        fn call_script_function_with_params() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/call_script_function_with_params.{}",
                    $script, $extension
                )
                .to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut();
                    scripting_runtime
                        .call_fn("test_func", &mut script_data, entity, vec![1])
                        .unwrap();
                },
            );
        }

        #[test]
        fn rust_function_gets_called_from_script_with_param() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct IntResource {
                my_int: i64,
            }

            app.world.init_resource::<IntResource>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(
                    String::from("rust_func"),
                    |In((x,)): In<(i64,)>, mut res: ResMut<IntResource>| {
                        res.my_int = x;
                    },
                );
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/rust_function_gets_called_from_script_with_param.{}",
                    $script, $extension
                )
                .to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert_eq!(app.world.get_resource::<IntResource>().unwrap().my_int, 5);
        }

        #[test]
        fn rust_function_gets_called_from_script_with_multiple_params() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct TestResource {
                a: i64,
                b: String,
            }

            app.world.init_resource::<TestResource>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(
                    String::from("rust_func"),
                    |In((a, b)): In<(i64, String)>, mut res: ResMut<TestResource>| {
                        res.a = a;
                        res.b = b;
                    },
                );
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/rust_function_gets_called_from_script_with_multiple_params.{}",
                    $script, $extension
                )
                .to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert_eq!(app.world.get_resource::<TestResource>().unwrap().a, 5);
            assert_eq!(
                app.world.get_resource::<TestResource>().unwrap().b,
                String::from("test")
            );
        }

        #[test]
        fn test_script_function_gets_called_from_rust_with_single_param() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            let entity_id = run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/script_function_gets_called_from_rust_with_single_param.{}",
                    $script, $extension
                )
                .to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut();
                    scripting_runtime
                        .call_fn("test_func", &mut script_data, entity, vec![1])
                        .unwrap();
                },
            );

            <$runtime>::assert_state_key_value_integer(&app.world, entity_id, "a_value", 1i64);
        }

        #[test]
        fn test_script_function_gets_called_from_rust_with_heterogenous_params() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            let entity_id = run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/script_function_gets_called_from_rust_with_multiple_params.{}",
                    $script, $extension
                )
                .to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut();
                    scripting_runtime
                        .call_fn(
                            "test_func",
                            &mut script_data,
                            entity,
                            (1, String::from("abc")),
                        )
                        .unwrap();
                },
            );

            <$runtime>::assert_state_key_value_integer(&app.world, entity_id, "a_value", 1i64);
            <$runtime>::assert_state_key_value_string(
                &app.world,
                entity_id,
                "b_value",
                &String::from("abc"),
            );
        }

        #[test]
        fn test_script_function_gets_called_from_rust_with_multiple_params() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            let entity_id = run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/script_function_gets_called_from_rust_with_multiple_params.{}",
                    $script, $extension
                )
                .to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut();
                    scripting_runtime
                        .call_fn("test_func", &mut script_data, entity, vec![1, 2])
                        .unwrap();
                },
            );

            <$runtime>::assert_state_key_value_integer(&app.world, entity_id, "a_value", 1);
            <$runtime>::assert_state_key_value_integer(&app.world, entity_id, "b_value", 2);
        }

        #[test]
        fn test_call_script_function_that_casues_runtime_error() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/call_script_function_that_causes_runtime_error.{}",
                    $script, $extension
                )
                .to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut();
                    let result =
                        scripting_runtime.call_fn("test_func", &mut script_data, entity, ());
                    assert!(result.is_err());
                },
            );
        }

        #[test]
        fn test_call_script_function_that_does_not_exist() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/call_script_function_that_causes_runtime_error.{}",
                    $script, $extension
                )
                .to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut();
                    let result =
                        scripting_runtime.call_fn("does_not_exist", &mut script_data, entity, ());
                    assert!(result.is_err());
                },
            );
        }

        #[test]
        fn test_script_function_gets_called_from_rust() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            let entity_id = run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/script_function_gets_called_from_rust.{}",
                    $script, $extension
                )
                .to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            <$runtime>::assert_state_key_value_integer(&app.world, entity_id, "times_called", 1);
        }

        #[test]
        fn test_promise() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(String::from("rust_func"), || 123);
            });

            let entity_id = run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/return_via_promise.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            <$runtime>::assert_state_key_value_integer(&app.world, entity_id, "x", 123);
        }

        #[test]
        fn test_promise_runtime_error_does_not_panic() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(String::from("rust_func"), || 123);
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/promise_runtime_error.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );
        }

        #[test]
        fn test_side_effects() {
            let mut app = build_test_app();

            #[derive(Component)]
            struct MyTag;

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(String::from("spawn_entity"), |mut commands: Commands| {
                    commands.spawn(MyTag);
                });
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/side_effects.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            app.world.run_system_once(|tagged: Query<&MyTag>| {
                tagged.single();
            });
        }

        #[test]
        fn test_rust_function_gets_called_from_script() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct TimesCalled {
                times_called: u8,
            }

            app.world.init_resource::<TimesCalled>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(String::from("rust_func"), |mut res: ResMut<TimesCalled>| {
                    res.times_called += 1;
                });
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/rust_function_gets_called_from_script.{}",
                    $script, $extension
                )
                .to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert_eq!(
                app.world
                    .get_resource::<TimesCalled>()
                    .unwrap()
                    .times_called,
                1
            );
        }
    };
}

#[cfg(feature = "rhai")]
mod rhai_tests {
    use bevy::prelude::*;
    use bevy_scriptum::runtimes::rhai::prelude::*;

    impl Prepare for RhaiRuntime {}

    impl AssertStateKeyValue for RhaiRuntime {
        type ScriptData = RhaiScriptData;

        fn assert_state_key_value_integer(world: &World, entity_id: Entity, key: &str, value: i64) {
            let script_data = world.get::<Self::ScriptData>(entity_id).unwrap();
            let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
            assert_eq!(
                state[key]
                    .clone()
                    .try_cast::<i64>()
                    .or_else(|| state[key].clone().try_cast::<i32>().map(|x| x as i64))
                    .unwrap(),
                value
            );
        }

        fn assert_state_key_value_string(world: &World, entity_id: Entity, key: &str, value: &str) {
            let script_data = world.get::<Self::ScriptData>(entity_id).unwrap();
            let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
            assert_eq!(state[key].clone_cast::<String>(), value);
        }
    }

    scripting_tests!(RhaiRuntime, "rhai", "rhai");
}

#[cfg(feature = "lua")]
mod lua_tests {
    use bevy::prelude::*;
    use bevy_scriptum::runtimes::lua::prelude::*;

    impl Prepare for LuaRuntime {}

    impl AssertStateKeyValue for LuaRuntime {
        type ScriptData = LuaScriptData;

        fn assert_state_key_value_integer(
            world: &World,
            _entity_id: Entity,
            key: &str,
            value: i64,
        ) {
            let runtime = world.get_resource::<LuaRuntime>().unwrap();
            runtime.with_engine(|engine| {
                let state = engine.globals().get::<_, Table>("State").unwrap();
                assert_eq!(
                    state
                        .get::<_, i64>(key)
                        .or_else(|_| state.get::<_, i32>(key).map(|x| x as i64))
                        .unwrap(),
                    value
                );
            });
        }

        fn assert_state_key_value_string(
            world: &World,
            _entity_id: Entity,
            key: &str,
            value: &str,
        ) {
            let runtime = world.get_resource::<LuaRuntime>().unwrap();
            runtime.with_engine(|engine| {
                let state = engine.globals().get::<_, Table>("State").unwrap();
                assert_eq!(state.get::<_, String>(key).unwrap(), value);
            });
        }
    }

    scripting_tests!(LuaRuntime, "lua", "lua");
}

#[cfg(feature = "rune")]
mod rune_tests {

    use std::{any::Any, collections::HashMap, sync::Mutex};

    use bevy::prelude::*;
    use bevy_scriptum::runtimes::rune::{prelude::*, RuneScriptData};
    use rune::Module;

    #[derive(Resource)]
    struct State {
        state: std::sync::Arc<Mutex<HashMap<String, Box<dyn Any + Send>>>>,
    }

    impl Prepare for RuneRuntime {
        fn prepare(world: &mut World) {
            let mut state = HashMap::new();
            state.insert(
                String::from("times_called"),
                Box::new(0_i64) as Box<dyn Any + Send>,
            );
            let state = std::sync::Arc::new(Mutex::new(state));
            world.insert_resource(State {
                state: state.clone(),
            });
            let mut runtime = world.get_resource_mut::<RuneRuntime>().unwrap();
            let mut m = Module::new();

            let state_closure = state.clone();
            m.function("set_state_value", move |key: String, value: i64| {
                let mut state = state.lock().unwrap();
                state.insert(key, Box::new(value));
            })
            .build()
            .unwrap();

            m.function("increment_times_called", move || {
                let mut state = state_closure.lock().unwrap();
                let times_called: i64 = *(state["times_called"].downcast_ref::<i64>().unwrap());
                state.insert("times_called".into(), Box::new(times_called + 1));
            })
            .build()
            .unwrap();

            runtime.context.install(m).unwrap();
            runtime.engine = std::sync::Arc::new(runtime.context.runtime().unwrap());
        }
    }

    impl AssertStateKeyValue for RuneRuntime {
        type ScriptData = RuneScriptData;

        fn assert_state_key_value_integer(world: &World, _entity: Entity, key: &str, value: i64) {
            let state = world.get_resource::<State>().unwrap();
            let state = state.state.lock().unwrap();
            let x: i64 = *(state
                .get(key)
                .unwrap_or_else(|| panic!("no key {}", key))
                .downcast_ref()
                .unwrap());
            assert_eq!(x, value);
        }

        fn assert_state_key_value_string(
            _world: &World,
            _entity_id: Entity,
            _key: &str,
            _value: &str,
        ) {
            todo!();
        }
    }

    scripting_tests!(RuneRuntime, "rune", "rn");
}
