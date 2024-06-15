# Interacting with bevy in callbacks

Every registered function is also just a regular Bevy system.

That allows you to do anything you would do in a Bevy system.

You could for example create a callback system function that prints names
of all entities with `Player` component.

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

In script:

```lua
print_player_names();
```

You can use functions that interact with Bevy entities and resources and
take arguments at the same time. It could be used for example to mutate a
component.

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

#[derive(Component)]
struct Player {
    health: i32
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
            runtime.add_function(
                String::from("hurt_player"),
                |In((hit_value,)): In<(i32,)>, mut players: Query<&mut Player>| {
                    let mut player = players.single_mut();
                    player.health -= hit_value;
                },
            );
        })
        .run();
}
```
