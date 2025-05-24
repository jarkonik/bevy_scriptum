use bevy::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;
use bevy_scriptum::{prelude::*, BuildScriptingRuntime};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|_| {})
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/ruby/entity_variable.rb"),
    ));
}
