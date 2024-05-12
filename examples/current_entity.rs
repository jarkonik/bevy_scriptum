use bevy::prelude::*;
use bevy_scriptum::{
    prelude::*,
    rhai_support::{RhaiRuntimeBuilder, RhaiScript},
    AddScriptingRuntimeAppExt, Script,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScriptingPlugin)
        .add_scripting_runtime::<RhaiRuntimeBuilder>(|r| {
            r.add_script_function(
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
        Script::<RhaiScript>::new(assets_server.load("examples/current_entity.rhai")),
    ));
}
