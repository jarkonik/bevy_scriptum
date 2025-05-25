use bevy::prelude::*;
use bevy_scriptum::ScriptingError;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::magnus;
use bevy_scriptum::runtimes::ruby::magnus::Module as _;
use bevy_scriptum::runtimes::ruby::magnus::Object as _;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|runtime| {
            runtime.add_function(String::from("hello_bevy"), || {
                println!("hello bevy, called from script");
            });
        })
        .add_systems(Startup, startup)
        .run();
}

#[magnus::wrap(class = "MyType")]
struct MyType {
    my_field: u32,
}

impl MyType {
    fn new() -> Self {
        Self { my_field: 42 }
    }

    fn my_method(&self) -> u32 {
        self.my_field
    }
}

fn startup(
    mut commands: Commands,
    scripting_runtime: ResMut<RubyRuntime>,
    assets_server: Res<AssetServer>,
) {
    scripting_runtime
        .with_engine_send(|ruby| {
            let my_type = ruby.define_class("MyType", ruby.class_object())?;
            my_type.define_singleton_method("new", magnus::function!(MyType::new, 0))?;
            my_type.define_method("my_method", magnus::method!(MyType::my_method, 0))?;

            Ok::<(), ScriptingError>(())
        })
        .unwrap();

    commands.spawn(Script::<RubyScript>::new(
        assets_server.load("examples/ruby/custom_type.rb"),
    ));
}
