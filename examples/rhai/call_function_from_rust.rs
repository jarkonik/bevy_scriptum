use bevy::{app::AppExit, prelude::*};
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, call_rhai_on_update_from_rust)
        .add_scripting::<RhaiRuntime>(|runtime| {
            runtime.add_function(String::from("quit"), |mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            });
        })
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/rhai/call_function_from_rust.rhai"),
    ));
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RhaiScriptData)>,
    scripting_runtime: ResMut<RhaiRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, ())
            .unwrap();
    }
}
