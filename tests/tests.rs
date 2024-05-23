use std::sync::OnceLock;

use bevy::ecs::system::RunSystemOnce as _;
use bevy::prelude::*;
use bevy_scriptum::{prelude::*, EngineRef as _, FuncArgs, Runtime};
use mlua::Table;

static TRACING_SUBSCRIBER: OnceLock<()> = OnceLock::new();

fn build_test_app() -> App {
    let mut app = App::new();

    TRACING_SUBSCRIBER.get_or_init(|| {
        tracing_subscriber::fmt().init();
    });

    app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()));
    app
}

fn run_callback<R: Runtime, Out, Marker>(
    app: &mut App,
    path: String,
    system: impl IntoSystem<(), Out, Marker>,
) -> Entity {
    let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
    let asset = asset_server.load::<R::ScriptAsset>(path);

    let entity_id = app.world.spawn(Script::new(asset)).id();
    app.update(); // let `ScriptData` resources be added to entities
    app.world.run_system_once(system);
    app.update(); // let callbacks be executed

    entity_id
}

fn run_script_from_string<R: Runtime>(app: &mut App, script: String) -> Entity {
    let mut scripts = app
        .world
        .get_resource_mut::<Assets<R::ScriptAsset>>()
        .unwrap();
    let asset = scripts.add(R::ScriptAsset::from(script));

    let entity_id = app.world.spawn(Script::new(asset)).id();
    app.update(); // let `ScriptData` resources be added to entities

    entity_id
}

fn call_script_on_update_from_rust<R: Runtime>(
    mut scripted_entities: Query<(Entity, &mut R::ScriptData)>,
    scripting_runtime: ResMut<R>,
) where
    (): FuncArgs<R::Value>,
{
    let (entity, mut script_data) = scripted_entities.single_mut();
    scripting_runtime
        .call_fn("test_func", &mut script_data, entity, ())
        .unwrap();
}

trait AssertStateKeyValue {
    type ScriptData;
    fn assert_state_key_value_i64(world: &World, entity_id: Entity, key: &str, value: i64);
}

macro_rules! scripting_tests {
    ($runtime:ty, $script:literal, $extension:literal) => {
        use super::*;

        #[test]
        fn call_script_function_with_params() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            run_callback::<$runtime, _, _>(
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

            run_callback::<$runtime, _, _>(
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
        fn test_script_function_gets_called_from_rust() {
            let mut app = build_test_app();

            app.add_scripting::<$runtime>(|_| {});

            let entity_id = run_callback::<$runtime, _, _>(
                &mut app,
                format!(
                    "tests/{}/script_function_gets_called_from_rust.{}",
                    $script, $extension
                )
                .to_string(),
                call_script_on_update_from_rust::<$runtime>,
            );

            <$runtime>::assert_state_key_value_i64(&app.world, entity_id, "times_called", 1i64);
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

            run_callback::<$runtime, _, _>(
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

macro_rules! type_conversion {
    ($runtime:ty, $mod: ident, $type:ty, $expected:expr, $value: literal) => {
        mod $mod {
            use super::{build_test_app, run_script_from_string};
            use bevy::prelude::*;
            use bevy_scriptum::prelude::*;
            use bevy_scriptum::runtimes::lua::prelude::*;

            #[test]
            fn type_conversions() {
                let mut app = build_test_app();
                app.add_scripting::<$runtime>(|runtime| {
                    runtime.add_function(String::from("rust_func"), |In((x,)): In<($type,)>| {
                        assert_eq!(x, $expected);
                    });
                });
                let script = format!("rust_func({})", $value);
                run_script_from_string::<LuaRuntime>(&mut app, script);
            }
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
    }

    scripting_tests!(RhaiRuntime, "rhai", "rhai");
}

#[cfg(feature = "luajit")]
mod lua_tests {
    use bevy::prelude::*;
    use bevy_scriptum::runtimes::lua::prelude::*;

    impl AssertStateKeyValue for LuaRuntime {
        type ScriptData = LuaScriptData;

        fn assert_state_key_value_i64(world: &World, _entity_id: Entity, key: &str, value: i64) {
            let runtime = world.get_resource::<LuaRuntime>().unwrap();
            let engine = &runtime.engine_ref().0;
            let engine = engine.lock().unwrap();
            let state = engine.globals().get::<_, Table>("State").unwrap();
            assert_eq!(state.get::<_, i64>(key).unwrap(), value);
        }
    }

    type_conversion!(LuaRuntime, nil, (), (), "nil");
    type_conversion!(LuaRuntime, boolean, bool, true, "true");
    // mlua::Value::LightUserData(_) => todo!(),
    type_conversion!(LuaRuntime, integer, i64, 5, "5");
    type_conversion!(LuaRuntime, number, f64, 5.1, "5.1");
    type_conversion!(LuaRuntime, string, String, "test", "\"test\"");
    // mlua::Value::Function(_) => todo!(),
    // mlua::Value::Thread(_) => todo!(),
    // mlua::Value::UserData(_) => todo!(),
    // mlua::Value::Error(_) => todo!(),

    // type_conversion!(LuaRuntime, table, std::collections::HashMap<String, LuaValue>, std::collections::HashMap::new(), "\"test\"");

    scripting_tests!(LuaRuntime, "lua", "lua");
}
