# Hello World

After you are done installing the required crates, you can start developing
your first game or application using bevy_scriptum.

To start using the library you need to first import some structs and traits
with Rust `use` statements.

For convenience there is a main "prelude" module provided called
`bevy_scriptum::prelude` and a prelude for each runtime you have enabled as
a create feature.

All you need to do is register callbacks on your Bevy app like this:
```rust
use bevy::prelude::*;
use bevy_scriptum::prelude::*;
use bevy_scriptum::runtimes::rhai::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_scripting::<RhaiRuntime>(|runtime| {
             runtime.add_function(String::from("hello_bevy"), || {
               println!("hello bevy, called from script");
             });
        });
}
```
And you can call them in your scripts like this:
```rhai
hello_bevy();
```
