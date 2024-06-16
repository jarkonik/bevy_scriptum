# Spawning scripts

To spawn a Lua script you will need to get a handle to a script asset using
bevy's `AssetServer`.

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn my_spawner(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("my_script.lua"),
    ));
}

fn main() {}
```

After they scripts have been evaled by bevy_scriptum, the entities that they've
been attached to will get the `Script::<LuaScript>` component stripped and instead
```LuaScriptData``` component will be attached.

So to query scipted entities you could do something like:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn my_system(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        // do something with scripted entities
    }
}

fn main() {}
```
