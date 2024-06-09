use bevy::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;
use bevy_scriptum::{prelude::*, BuildScriptingRuntime};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|_| {})
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("examples/lua/entity_variable.lua"),
    ));
}
