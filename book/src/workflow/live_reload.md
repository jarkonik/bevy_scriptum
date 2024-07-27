# Live-reload

## Bevy included support

To enable live reload it should be enough to enable `file-watcher` feature
within bevy dependency in `Cargo.toml`

```
bevy = { version = "0.13", features = ["file_watcher"] }
```

## Init-teardown pattern for game development

It is useful to structure your game in a way that would allow making changes to
the scripting code without restarting the game.

A useful pattern is to hava three functions "init", "update" and "teardown".

- "init" function will take care of starting the game(spawning the player, the level etc)

- "update" function will run the main game logic

- "teardown" function will despawn all the entities so game starts at fresh state.

This pattern is very easy to implement in bevy_scriptum. All you need is to define all needed functions
in script:

```lua
player = {
    entity = nil
}

-- spawning all needed entities
local function init()
	player.entity = spawn_player()
end

-- game logic here, should be called in a bevy system using call_fn
local function update()
    (...)
end

-- despawning entities and possible other cleanup logic needed
local function teardown()
	despawn(player.entity)
end

-- call init to start the game, this will be called on each file-watcher script
-- reload
init()
```

The function calls can be implemented on Rust side the following way:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;
use bevy_scriptum::runtimes::lua::BevyVec3;

fn init(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands.spawn(Script::<LuaScript>::new(
        assets_server.load("scripts/game.lua"),
    ));
}


fn update(
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
    scripting_runtime: ResMut<LuaRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        scripting_runtime
            .call_fn("update", &mut script_data, entity, ())
            .unwrap();
    }
}


fn teardown(
    mut ev_asset: EventReader<AssetEvent<LuaScript>>,
    scripting_runtime: ResMut<LuaRuntime>,
    mut scripted_entities: Query<(Entity, &mut LuaScriptData)>,
) {
    for event in ev_asset.read() {
        if let AssetEvent::Modified { .. } = event {
            for (entity, mut script_data) in &mut scripted_entities {
                scripting_runtime
                    .call_fn("teardown", &mut script_data, entity, ())
                    .unwrap();
            }
        }
    }
}

fn main() {}
```

And to tie this all together we do the following:

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|builder| {
            builder
                .add_function(String::from("spawn_player"), spawn_player)
                .add_function(String::from("despawn"), despawn);
        })
        .add_systems(Startup, init)
        .add_systems(Update, (update, teardown))
        .run();
}

fn init() {} // Implemented elsewhere
fn update() {} // Implemented elsewhere
fn despawn() {} // Implemented elsewhere
fn teardown() {} // Implemented elsewhere
fn spawn_player() {} // Implemented elsewhere
```

`despawn` can be implemented as:

```rust
use bevy::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn despawn(In((entity,)): In<(BevyEntity,)>, mut commands: Commands) {
    commands.entity(entity.0).despawn();
}

fn main() {} // Implemented elsewhere
```

Implementation of `spawn_player` has been left out as an exercise for the reader.
