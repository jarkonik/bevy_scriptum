use bevy::{app::AppExit, ecs::event::ManualEventReader, prelude::*};
use bevy_scriptum::{
    prelude::*,
    rhai_support::{RhaiScript, RhaiScriptData, RhaiScriptingRuntime},
    AddScriptingRuntimeAppExt, CallFunction as _, Script, ScriptData, ScriptingRuntime,
};

fn main() {
    App::new()
        // This is just needed for headless console app, not needed for a regular bevy game
        // that uses a winit window
        .set_runner(move |mut app: App| {
            let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
            loop {
                if let Some(app_exit_events) = app.world.get_resource_mut::<Events<AppExit>>() {
                    if app_exit_event_reader
                        .read(&app_exit_events)
                        .last()
                        .is_some()
                    {
                        break;
                    }
                }
                app.update();
            }
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin::<RhaiScriptingRuntime>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, call_rhai_on_update_from_rust)
        // .add_scripting_runtime::<RhaiRuntimeBuilder>(|r| {
        //     r.add_script_function(String::from("quit"), |mut exit: EventWriter<AppExit>| {
        //         exit.send(AppExit);
        //     });
        // })
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RhaiScript>::new(
        assets_server.load("examples/call_function_from_rust.rhai"),
    ));
}

fn call_rhai_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut ScriptData<RhaiScriptData>)>,
    mut scripting_runtime: ResMut<RhaiScriptingRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, ())
            .unwrap();
    }
}
