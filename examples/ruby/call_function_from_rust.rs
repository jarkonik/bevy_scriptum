use bevy::{app::AppExit, prelude::*};
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, call_ruby_on_update_from_rust)
        .add_scripting::<RubyRuntime>(|runtime| {
            runtime.add_function(String::from("quit"), |mut exit: EventWriter<AppExit>| {
                exit.write(AppExit::Success);
            });
        })
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/ruby/call_function_from_rust.rb"),
    ));
}

fn call_ruby_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RubyScriptData)>,
    scripting_runtime: ResMut<RubyRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, ())
            .unwrap();
    }
}
