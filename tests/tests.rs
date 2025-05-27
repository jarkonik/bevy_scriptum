#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
use std::sync::OnceLock;

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
use bevy::ecs::system::RunSystemOnce as _;
#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
use bevy::prelude::*;
#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
use bevy_scriptum::{FuncArgs, Runtime, prelude::*};

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
static TRACING_SUBSCRIBER: OnceLock<()> = OnceLock::new();

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
#[derive(Default, Resource)]
struct TimesCalled {
    times_called: u8,
}

macro_rules! assert_n_times_called {
    ($app: expr, $count: expr) => {
        assert_eq!(
            $app.world()
                .get_resource::<TimesCalled>()
                .unwrap()
                .times_called,
            $count
        );
    };
}

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
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

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
fn run_script<R: Runtime, Out, Marker>(
    app: &mut App,
    path: String,
    system: impl IntoSystem<(), Out, Marker>,
) -> Entity {
    let asset_server = app.world_mut().get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load::<R::ScriptAsset>(path);

    let entity_id = app.world_mut().spawn(Script::new(asset)).id();
    app.update(); // let `ScriptData` resources be added to entities
    app.world_mut().run_system_once(system).unwrap();
    app.update(); // let callbacks be executed

    entity_id
}

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
fn call_script_on_update_from_rust<R: Runtime>(
    mut scripted_entities: Query<(Entity, &mut R::ScriptData)>,
    scripting_runtime: ResMut<R>,
) where
    (): for<'a> FuncArgs<'a, R::Value, R>,
{
    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
trait AssertStateKeyValue {
    type ScriptData;
    fn assert_state_key_value_i64(world: &World, entity_id: Entity, key: &str, value: i64);
    fn assert_state_key_value_i32(world: &World, entity_id: Entity, key: &str, value: i32);
    fn assert_state_key_value_string(world: &World, entity_id: Entity, key: &str, value: &str);
}

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
macro_rules! scripting_tests {
    ($runtime:ty, $script:literal, $extension:literal, $entity_type: ty, $vec_type: ty) => {
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
                    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
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

            app.world_mut().init_resource::<IntResource>();

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

            assert_eq!(app.world().get_resource::<IntResource>().unwrap().my_int, 5);
        }

        #[test]
        fn rust_function_gets_called_from_script_with_multiple_params() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct TestResource {
                a: i64,
                b: String,
            }

            app.world_mut().init_resource::<TestResource>();

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

            assert_eq!(app.world().get_resource::<TestResource>().unwrap().a, 5);
            assert_eq!(
                app.world().get_resource::<TestResource>().unwrap().b,
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
                    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
                    scripting_runtime
                        .call_fn("test_func", &mut script_data, entity, vec![1])
                        .unwrap();
                },
            );

            <$runtime>::assert_state_key_value_i32(&app.world(), entity_id, "a_value", 1i32);
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
                    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
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

            <$runtime>::assert_state_key_value_i32(&app.world(), entity_id, "a_value", 1i32);
            <$runtime>::assert_state_key_value_string(
                &app.world(),
                entity_id,
                "b_value",
                &String::from("abc"),
            );
        }

        #[test]
        fn eval_that_casues_runtime_error_doesnt_panic() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|r| {
                r.add_function(
                    String::from("mark_called"),
                    |mut times_called: ResMut<TimesCalled>| {
                        times_called.times_called += 1;
                    },
                );
            })
            .init_resource::<TimesCalled>();

            run_script::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/eval_that_causes_runtime_error.{}",
                    $script, $extension
                )
                .to_string(),
                || {},
            );

            assert_n_times_called!(app, 2);
        }

        #[test]
        fn call_script_function_that_casues_runtime_error() {
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
                    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
                    let result =
                        scripting_runtime.call_fn("test_func", &mut script_data, entity, ());
                    assert!(result.is_err());
                },
            );
        }

        #[test]
        fn call_script_function_that_does_not_exist() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/side_effects.{}", $script, $extension).to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
                    let result =
                        scripting_runtime.call_fn("does_not_exist", &mut script_data, entity, ());
                    assert!(result.is_err());
                },
            );
        }

        #[test]
        fn script_function_gets_called_from_rust() {
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

            <$runtime>::assert_state_key_value_i64(&app.world(), entity_id, "times_called", 1i64);
        }

        #[test]
        fn return_via_promise() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(String::from("rust_func"), || 123);
            });

            let entity_id = run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/return_via_promise.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            <$runtime>::assert_state_key_value_i32(&app.world(), entity_id, "x", 123i32);
        }

        #[test]
        fn promise_runtime_error() {
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
        fn side_effects() {
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

            app.world_mut()
                .run_system_once(|tagged: Query<&MyTag>| {
                    tagged.single().unwrap();
                })
                .unwrap();
        }

        #[test]
        fn rust_function_gets_called_from_script() {
            let mut app = build_test_app();

            app.world_mut().init_resource::<TimesCalled>();

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

            assert_n_times_called!(app, 1);
        }

        #[test]
        fn entity_variable_index_is_available_in_callback() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct State {
                index: u32,
            }

            app.world_mut().init_resource::<State>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(
                    String::from("rust_func"),
                    |In((index,)): In<(u32,)>, mut res: ResMut<State>| {
                        res.index = index;
                    },
                );
            });

            let entity = run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/entity_variable.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert_eq!(
                app.world().get_resource::<State>().unwrap().index,
                entity.index()
            );
        }

        #[test]
        fn entity_variable_index_is_available_in_eval() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct State {
                index: Option<u32>,
            }

            app.world_mut().init_resource::<State>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(
                    String::from("rust_func"),
                    |In((index,)): In<(u32,)>, mut res: ResMut<State>| {
                        res.index = Some(index);
                    },
                );
            });

            let entity = run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/entity_variable_eval.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert_eq!(
                app.world().get_resource::<State>().unwrap().index,
                Some(entity.index())
            );
        }

        #[test]
        fn pass_entity_from_script() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct State {
                index: Option<u32>,
            }

            app.world_mut().init_resource::<State>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(
                    String::from("rust_func"),
                    |In((entity,)): In<($entity_type,)>, mut res: ResMut<State>| {
                        res.index = Some(entity.index());
                    },
                );
            });

            let entity = run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/pass_entity_from_script.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert_eq!(
                app.world().get_resource::<State>().unwrap().index,
                Some(entity.index())
            );
        }

        #[test]
        fn pass_vec3_from_script() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct State {
                success: bool,
            }

            app.world_mut().init_resource::<State>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(
                    String::from("rust_func"),
                    |In((v,)): In<($vec_type,)>, mut res: ResMut<State>| {
                        assert_eq!(v.x(), 1.5);
                        assert_eq!(v.y(), 2.5);
                        assert_eq!(v.z(), -3.5);
                        res.success = true
                    },
                );
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/pass_vec3_from_script.{}", $script, $extension).to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            assert!(app.world().get_resource::<State>().unwrap().success);
        }

        #[test]
        fn pass_vec3_to_script() {
            let mut app = build_test_app();

            #[derive(Default, Resource)]
            struct State {
                success: bool,
            }

            app.world_mut().init_resource::<State>();

            app.add_scripting::<$runtime>(|runtime| {
                runtime.add_function(String::from("mark_success"), |mut res: ResMut<State>| {
                    res.success = true
                });
            });

            run_script::<$runtime, _, _>(
                &mut app,
                format!("tests/{}/pass_vec3_to_script.{}", $script, $extension).to_string(),
                |mut scripted_entities: Query<(Entity, &mut <$runtime as Runtime>::ScriptData)>,
                 scripting_runtime: ResMut<$runtime>| {
                    let (entity, mut script_data) = scripted_entities.single_mut().unwrap();
                    scripting_runtime
                        .call_fn(
                            "test_func",
                            &mut script_data,
                            entity,
                            (<$vec_type>::new(1.5, 2.5, -3.5),),
                        )
                        .unwrap();
                },
            );

            assert!(app.world().get_resource::<State>().unwrap().success);
        }
    };
}

#[cfg(feature = "rhai")]
mod rhai_tests {
    use bevy::prelude::*;
    use bevy_scriptum::runtimes::rhai::prelude::*;

    impl AssertStateKeyValue for RhaiRuntime {
        type ScriptData = RhaiScriptData;

        fn assert_state_key_value_i64(world: &World, entity_id: Entity, key: &str, value: i64) {
            let script_data = world.get::<Self::ScriptData>(entity_id).unwrap();
            let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
            assert_eq!(state[key].clone_cast::<i64>(), value);
        }

        fn assert_state_key_value_i32(world: &World, entity_id: Entity, key: &str, value: i32) {
            let script_data = world.get::<Self::ScriptData>(entity_id).unwrap();
            let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
            assert_eq!(state[key].clone_cast::<i32>(), value);
        }

        fn assert_state_key_value_string(world: &World, entity_id: Entity, key: &str, value: &str) {
            let script_data = world.get::<Self::ScriptData>(entity_id).unwrap();
            let state = script_data.scope.get_value::<rhai::Map>("state").unwrap();
            assert_eq!(state[key].clone_cast::<String>(), value);
        }
    }

    scripting_tests!(RhaiRuntime, "rhai", "rhai", BevyEntity, BevyVec3);
}

#[cfg(feature = "lua")]
mod lua_tests {
    use bevy::prelude::*;
    use bevy_scriptum::runtimes::lua::prelude::*;
    use mlua::Table;

    impl AssertStateKeyValue for LuaRuntime {
        type ScriptData = LuaScriptData;

        fn assert_state_key_value_i64(world: &World, _entity_id: Entity, key: &str, value: i64) {
            let runtime = world.get_resource::<LuaRuntime>().unwrap();
            runtime.with_engine(|engine| {
                let state = engine.globals().get::<_, Table>("State").unwrap();
                assert_eq!(state.get::<_, i64>(key).unwrap(), value);
            });
        }

        fn assert_state_key_value_i32(world: &World, _entity_id: Entity, key: &str, value: i32) {
            let runtime = world.get_resource::<LuaRuntime>().unwrap();
            runtime.with_engine(|engine| {
                let state = engine.globals().get::<_, Table>("State").unwrap();
                assert_eq!(state.get::<_, i32>(key).unwrap(), value);
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

    scripting_tests!(LuaRuntime, "lua", "lua", BevyEntity, BevyVec3);
}

#[cfg(feature = "ruby")]
mod ruby_tests {
    use bevy::prelude::*;
    use bevy_scriptum::runtimes::ruby::{RubyScriptData, prelude::*};
    use magnus::value::ReprValue;

    impl AssertStateKeyValue for RubyRuntime {
        type ScriptData = RubyScriptData;

        fn assert_state_key_value_i64(world: &World, _entity_id: Entity, key: &str, value: i64) {
            let runtime = world.get_resource::<RubyRuntime>().unwrap();
            let key = key.to_string();
            runtime.with_engine_send(move |engine| {
                let state: magnus::value::Value = engine.eval("$state").unwrap();
                let res: i64 = state.funcall_public("[]", (key,)).unwrap();
                assert_eq!(res, value)
            })
        }

        fn assert_state_key_value_i32(world: &World, _entity_id: Entity, key: &str, value: i32) {
            let runtime = world.get_resource::<RubyRuntime>().unwrap();
            let key = key.to_string();
            runtime.with_engine_send(move |engine| {
                let state: magnus::value::Value = engine.eval("$state").unwrap();
                let res: i32 = state.funcall_public("[]", (key,)).unwrap();
                assert_eq!(res, value)
            })
        }

        fn assert_state_key_value_string(
            world: &World,
            _entity_id: Entity,
            key: &str,
            value: &str,
        ) {
            let runtime = world.get_resource::<RubyRuntime>().unwrap();
            let key = key.to_string();
            let value = value.to_string();
            runtime.with_engine_send(move |engine| {
                let state: magnus::value::Value = engine.eval("$state").unwrap();
                let res: String = state.funcall_public("[]", (key,)).unwrap();
                assert_eq!(res, value);
            });
        }
    }

    #[test]
    fn test_symbol_inspection() {
        let mut app = build_test_app();

        app.add_scripting::<RubyRuntime>(|_| {});
        let runtime = app.world().get_resource::<RubyRuntime>().unwrap();
        runtime.with_engine_send(|engine| {
            let symbol_string: String = engine.eval(":test_symbol.inspect").unwrap();
            assert_eq!(symbol_string, ":test_symbol")
        });
    }

    scripting_tests!(RubyRuntime, "ruby", "rb", BevyEntity, BevyVec3);
}
