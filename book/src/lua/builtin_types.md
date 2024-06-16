# Builtin types

bevy_scriptum provides following types that can be used in Lua:

- ```Vec3```
- ```BevyEntity```

## Vec3

### Constructor

`Vec3(x: number, y: number, z: number)`

### Properties

- `x: number`
- `y: number`
- `z: number`


### Example Lua usage

```
my_vec = Vec3(1, 2, 3)
set_translation(entity, my_vec)
```

### Example Rust usage

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("set_translation"), set_translation);
        })
        .run();
}

fn set_translation(
    In((entity, translation)): In<(BevyEntity, BevyVec3)>,
    mut entities: Query<&mut Transform>,
) {
    let mut transform = entities.get_mut(entity.0).unwrap();
    transform.translation = translation.0;
}
```

## BevyEntity

### Constructor

None - instances can only be acquired by using built-in `entity` global variable.

### Properties

- `index: integer`

### Example Lua usage

```lua
print(entity.index)
pass_to_rust(entity)
```

### Example Rust usage

```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::lua::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<LuaRuntime>(|runtime| {
             runtime.add_function(String::from("pass_to_rust"), |In((entity,)): In<(BevyEntity,)>| {
               println!("pass_to_rust called with entity: {:?}", entity);
             });
        })
        .run();
}
```
