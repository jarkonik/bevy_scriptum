# Calling Lua from Rust

To call a function defined in Lua

```lua
function on_update()
end
```

We need to acquire `LuaRuntime` resource within a bevy system.
Then we will be able to call `call_fn` on it, providing the name
of the function to call, `LuaScriptData` that has been automatically
attached to entity after an entity with script attached has been spawned
and its script evaluated, the entity and optionally some arguments.

```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn call_lua_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
    scripting_runtime: ResMut<LuaRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        // calling function named `on_update` defined in lua script
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, ())
            .unwrap();
    }
}

fn main() {}
```

We can also pass some arguments by providing a tuple or `Vec` as the last
`call_fn` argument.

```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn call_lua_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
    scripting_runtime: ResMut<LuaRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, (123, String::from("hello")))
            .unwrap();
    }
}

fn main() {}
```

They will be passed to `on_update` Lua function
```lua
function on_update(a, b)
    print(a) -- 123
    print(b) -- hello
end
```

Any type that implements `IntoLua` can be passed as an argument withing the
tuple in `call_fn`.
