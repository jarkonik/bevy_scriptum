# Calling Rust from Lua

To call a rust function from Lua first you need to register a function
within Rust using builder pattern.

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            // `runtime` is a builder that you can use to register functions
        })
        .run();
}
```

For example to register a function called `my_rust_func` you can do the following:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("my_rust_func"), || {
               println!("my_rust_func has been called");
             });
        })
        .run();
}
```

After you do that the function will be available to Lua code in your spawned scripts.

```lua
my_rust_func()
```

Registered functions can also take parameters. A parameter can be any type
that implements `FromLua`.

Since a registered callback function is a Bevy system, the parameters are passed
to it as `In` struct with tuple, which has to be the first parameter of the closure.

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("func_with_params"), |args: In<(String, i64)>| {
               println!("my_rust_func has been called with string {} and i64 {}", args.0.0, args.0.1);
             });
        })
        .run();
}
```

To make it look nicer you can destructure the `In` struct.

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("func_with_params"), |In((a, b)): In<(String, i64)>| {
               println!("my_rust_func has been called with string {} and i64 {}", a, b);
             });
        })
        .run();
}
```

The above function can be called from Lua

```lua
func_with_params("abc", 123)
```

## Return value via promise

Any registered rust function that returns a value will retrurn a promise when
called within a script. By calling `:and_then` on the promise you can register
a callback that will receive the value returned from Rust function.

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("returns_value"), || {
                123
             });
        })
        .run();
}
```

```lua
returns_value():and_then(function (value)
    print(value) -- 123
end)
```
