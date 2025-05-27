use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|runtime| {
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
        Script::<RubyScript>::new(assets_server.load("examples/ruby/current_entity.rb")),
    ));
}
