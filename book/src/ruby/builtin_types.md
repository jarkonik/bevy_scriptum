# Builtin types

bevy_scriptum provides following types that can be used in Ruby:

- `Bevy::Vec3`
- `Bevy::Entity`

## Bevy::Vec3

### Class Methods

- `new(x, y, z)`
- `current`

### Instance Methods

- `x`
- `y`
- `z`

### Example Ruby usage

```ruby
my_vec = Bevy::Vec3.new(1, 2, 3)
set_translation(entity, my_vec)
```

### Example Rust usage

```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|runtime| {
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

## Bevy::Entity

`Bevy::Entity.current` is currently not available within promise callbacks.

### Constructor

None - instances can only be acquired by using `Bevy::Entity.current`

### Class method

- `index`

### Example Ruby usage

```ruby
puts(Bevy::Entity.current.index)
pass_to_rust(Bevy::Entity.current)
```

### Example Rust usage

```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RubyRuntime>(|runtime| {
             runtime.add_function(String::from("pass_to_rust"), |In((entity,)): In<(BevyEntity,)>| {
               println!("pass_to_rust called with entity: {:?}", entity);
             });
        })
        .run();
}
```
