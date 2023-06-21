use bevy::{prelude::*};
use bevy_scriptum::{prelude::*, Script};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScriptingPlugin::default())
        .add_script_function(
            String::from("get_name"),
            |In((entity,)): In<(Entity,)>, names: Query<&Name>| {
                names.get(entity).unwrap().to_string()
            },
        )
        .add_startup_system(startup)
        .run();
}

fn startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn((
        Name::from("MyEntityName"),
        Script::new(assets_server.load("examples/current_entity.rhai")),
    ));
}
