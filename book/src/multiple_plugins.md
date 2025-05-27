# Multiple plugins

It is possible to split the definition of your callback functions up over multiple plugins. This enables you to split up your code by subject and keep the main initialization light and clean.
This can be accomplished by using `add_scripting_api`. Be careful though, `add_scripting` has to be called before adding plugins.
```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

struct MyPlugin;
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_scripting_api::<LuaRuntime>(|runtime| {
            runtime.add_function(String::from("hello_from_my_plugin"), || {
                info!("Hello from MyPlugin");
            });
        });
    }
}

// Main
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|_| {
            // nice and clean
        })
        .add_plugins(MyPlugin)
        .run();
}
```
