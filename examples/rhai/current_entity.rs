use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RhaiRuntime>(|runtime| {
            runtime.add_function(
                String::from("get_name"),
                |In((entity,)): In<(Entity,)>, names: Query<&Name>| {
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
        Script::<RhaiScript>::new(assets_server.load("examples/rhai/current_entity.rhai")),
    ));
}
