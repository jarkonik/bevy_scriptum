use bevy::{app::AppExit, prelude::*};
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, call_rhai_on_update_from_rust)
        .add_scripting::<RhaiRuntime>(|runtime| {
            runtime.add_function(String::from("quit"), |mut exit: EventWriter<AppExit>| {
                exit.write(AppExit::Success);
            });
        })
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/rhai/function_return_value.rhai"),
    ));
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RhaiScriptData)>,
    scripting_runtime: ResMut<RhaiRuntime>,
    mut exit: EventWriter<AppExit>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        let val = scripting_runtime
            .call_fn("get_value", &mut script_data, entity, ())
            .unwrap()
            .0;
        println!("script returned: {}", val);
        exit.write(AppExit::Success);
    }
}
