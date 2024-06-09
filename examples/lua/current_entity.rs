use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime.add_function(
                String::from("get_name"),
                |In((BevyEntity(entity),)): In<(BevyEntity,)>, names: Query<&Name>| {
                    names.get(entity).unwrap().to_string()
                },
            );
        })
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((
        Name::from("MyEntityName"),
        Script::<LuaScript>::new(assets_server.load("examples/lua/current_entity.lua")),
    ));
}
