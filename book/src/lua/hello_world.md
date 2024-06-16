# Hello World

After you are done installing the required crates, you can start developing
your first game or application using bevy_scriptum.

To start using the library you need to first import some structs and traits
with Rust `use` statements.

For convenience there is a main "prelude" module provided called
`bevy_scriptum::prelude` and a prelude for each runtime you have enabled as
a create feature.

You can now start exposing functions to the scripting language. For example, you can expose a function that prints a message to the console:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
           runtime.add_function(
               String::from("my_print"),
               |In((x,)): In<(String,)>| {
                   println!("my_print: '{}'", x);
               },
           );
        })
        .run();
}
```

Then you can create a script file in `assets` directory called `script.lua` that calls this function:

```lua
my_print("Hello world!")
```

And spawn an entity with attached `Script` component with a handle to a script source file:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
           runtime.add_function(
               String::from("my_print"),
               |In((x,)): In<(String,)>| {
                   println!("my_print: '{}'", x);
               },
           );
        })
        .add_systems(Startup,|mut commands: Commands, asset_server: Res<AssetServer>| {
            commands.spawn(Script::<LuaScript>::new(asset_server.load("script.lua")));
        })
        .run();
}
```

You should then see `my_print: 'Hello world!'` printed in your console.
