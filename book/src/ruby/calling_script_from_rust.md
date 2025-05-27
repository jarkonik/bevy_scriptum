# Calling Ruby from Rust

To call a function defined in Ruby

```ruby
def on_update
end
```

We need to acquire `RubyRuntime` resource within a bevy system.
Then we will be able to call `call_fn` on it, providing the name
of the function to call, `RubyScriptData` that has been automatically
attached to entity after an entity with script attached has been spawned
and its script evaluated, the entity and optionally some arguments.

```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn call_ruby_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RubyScriptData)>,
    scripting_runtime: ResMut<RubyRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        // calling function named `on_update` defined in Ruby script
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, ())
            .unwrap();
    }
}
```

We can also pass some arguments by providing a tuple or `Vec` as the last
`call_fn` argument.

```rust,no_run
# extern crate bevy;
# extern crate bevy_scriptum;

use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::ruby::prelude::*;

fn call_ruby_on_update_from_rust(
    mut scripted_entities: Query<(Entity, &mut RubyScriptData)>,
    scripting_runtime: ResMut<RubyRuntime>,
) {
    for (entity, mut script_data) in &mut scripted_entities {
        scripting_runtime
            .call_fn("on_update", &mut script_data, entity, (123, String::from("hello")))
            .unwrap();
    }
}
```

They will be passed to `on_update` Ruby function
```ruby
def on_update(a, b)
    puts(a) # 123
    puts(b) # hello
end
```
