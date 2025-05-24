use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

// Plugin A
struct PluginA;
impl Plugin for PluginA {
    fn build(&self, app: &mut App) {
        app.add_scripting_api::<RubyRuntime>(|runtime| {
            runtime.add_function(String::from("hello_from_plugin_a"), || {
                info!("Hello from Plugin A");
            });
        })
        .add_systems(Startup, plugin_a_startup);
    }
}

fn plugin_a_startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/lua/multiple_plugins_plugin_a.lua"),
    ));
}

// Plugin B
struct PluginB;
impl Plugin for PluginB {
    fn build(&self, app: &mut App) {
        app.add_scripting_api::<RubyRuntime>(|runtime| {
            runtime.add_function(
                String::from("hello_from_plugin_b_with_parameters"),
                hello_from_b,
            );
        })
        .add_systems(Startup, plugin_b_startup);
    }
}

fn plugin_b_startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/lua/multiple_plugins_plugin_b.lua"),
    ));
}

fn hello_from_b(In((text, x)): In<(String, i32)>) {
    info!("{} from Plugin B: {}", text, x);
}

// Main
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|runtime| {
            runtime.add_function(String::from("hello_bevy"), || {
                info!("hello bevy, called from script");
            });
        })
        .add_systems(Startup, main_startup)
        .add_plugins(PluginA)
        .add_plugins(PluginB)
        .run();
}

fn main_startup(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/ruby/hello_world.rb"),
    ));
}
