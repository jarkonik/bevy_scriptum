# bevy_scriptum 📜

bevy_scriptum is a a plugin for [Bevy](https://bevyengine.org/) that allows you to write some of your game logic in a scripting language.
Currently [Rhai](https://rhai.rs/) and [Lua](https://lua.org/) are supported, but more languages may be added in the future.

API docs are available in [docs.rs](https://docs.rs/bevy_scriptum/latest/bevy_scriptum/)

bevy_scriptum's main advantages include:
- low-boilerplate
- easy to use
- asynchronicity with a promise-based API
- flexibility
- hot-reloading

Scripts are separate files that can be hot-reloaded at runtime. This allows you to quickly iterate on your game logic without having to recompile your game.

All you need to do is register callbacks on your Bevy app like this:
```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("hello_bevy"), || {
               println!("hello bevy, called from script");
             });
        })
        .run();
}
```
And you can call them in your scripts like this:
```lua
hello_bevy()
```

Every callback function that you expose to the scripting language is also a Bevy system, so you can easily query and mutate ECS components and resources just like you would in a regular Bevy system:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime.add_function(
                String::from("print_player_names"),
                |players: Query<&Name, With<Player>>| {
                    for player in &players {
                        println!("player name: {}", player);
                    }
                },
            );
        })
        .run();
}
```

You can also pass arguments to your callback functions, just like you would in a regular Bevy system - using `In` structs with tuples:
```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime.add_function(
                String::from("fun_with_string_param"),
                |In((x,)): In<(String,)>| {
                    println!("called with string: '{}'", x);
                },
            );
        })
        .run();
}
```
which you can then call in your script like this:
```lua
fun_with_string_param("Hello world!")
```

### Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
bevy_scriptum = { version = "0.6", features = ["lua"] }
```

or execute `cargo add bevy_scriptum --features lua` from your project directory.

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

### Provided examples

You can also try running provided examples by cloning this repository and running `cargo run --example <example_name>_<language_name>`.  For example:

```bash
cargo run --example hello_world_lua
```
The examples live in `examples` directory and their corresponding scripts live in `assets/examples` directory within the repository.

### Promises - getting return values from scripts

Every function called from script returns a promise that you can call `:and_then` with a callback function on. This callback function will be called when the promise is resolved, and will be passed the return value of the function called from script. For example:

```lua
get_player_name():and_then(function(name)
    print(name)
end)
```
which will print out `John` when used with following exposed function:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
       .add_plugins(DefaultPlugins)
       .add_scripting::<LuaRuntime>(|runtime| {
               runtime.add_function(String::from("get_player_name"), || String::from("John"));
       });
}
````

## Access entity from script

A variable called `entity` is automatically available to all scripts - it represents bevy entity that the `Script` component is attached to.
It exposes `index` property that returns bevy entity index.
It is useful for accessing entity's components from scripts.
It can be used in the following way:
```lua
print("Current entity index: " .. entity.index)
```

`entity` variable is currently not available within promise callbacks.

### Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

### License

bevy_scriptum is licensed under either of the following, at your option:
Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0) or MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
