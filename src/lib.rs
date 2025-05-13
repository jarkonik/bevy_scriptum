//! ![demo](demo.gif)
//!
//! bevy_scriptum is a a plugin for [Bevy](https://bevyengine.org/) that allows you to write some of your game logic in a scripting language.
//! Currently [Rhai](https://rhai.rs/) and [Lua](https://lua.org/) are supported, but more languages may be added in the future.
//!
//! Everything you need to know to get started with using this library is contained in the
//! [bevy_scriptum book](https://jarkonik.github.io/bevy_scriptum/)
//!
//! API docs are available in [docs.rs](https://docs.rs/bevy_scriptum/latest/bevy_scriptum/)
//!
//! bevy_scriptum's main advantages include:
//! - low-boilerplate
//! - easy to use
//! - asynchronicity with a promise-based API
//! - flexibility
//! - hot-reloading
//!
//! Scripts are separate files that can be hot-reloaded at runtime. This allows you to quickly iterate on your game logic without having to recompile your game.
//!
//! All you need to do is register callbacks on your Bevy app like this:
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_scripting::<LuaRuntime>(|runtime| {
//!          runtime.add_function(String::from("hello_bevy"), || {
//!            println!("hello bevy, called from script");
//!          });
//!     })
//!     .run();
//! ```
//! And you can call them in your scripts like this:
//! ```lua
//! hello_bevy()
//! ```
//!
//! Every callback function that you expose to the scripting language is also a Bevy system, so you can easily query and mutate ECS components and resources just like you would in a regular Bevy system:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! #[derive(Component)]
//! struct Player;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_scripting::<LuaRuntime>(|runtime| {
//!         runtime.add_function(
//!             String::from("print_player_names"),
//!             |players: Query<&Name, With<Player>>| {
//!                 for player in &players {
//!                     println!("player name: {}", player);
//!                 }
//!             },
//!         );
//!     })
//!     .run();
//! ```
//!
//! You can also pass arguments to your callback functions, just like you would in a regular Bevy system - using `In` structs with tuples:
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_scripting::<LuaRuntime>(|runtime| {
//!         runtime.add_function(
//!             String::from("fun_with_string_param"),
//!             |In((x,)): In<(String,)>| {
//!                 println!("called with string: '{}'", x);
//!             },
//!         );
//!     })
//!     .run();
//! ```
//! which you can then call in your script like this:
//! ```lua
//! fun_with_string_param("Hello world!")
//! ```
//! It is also possible to split the definition of your callback functions up over multiple plugins. This enables you to split up your code by subject and keep the main initialization light and clean.
//! This can be accomplished by using `add_scripting_api`. Be careful though, `add_scripting` has to be called before adding plugins.
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! struct MyPlugin;
//! impl Plugin for MyPlugin {
//!     fn build(&self, app: &mut App) {
//!         app.add_scripting_api::<LuaRuntime>(|runtime| {
//!             runtime.add_function(String::from("hello_from_my_plugin"), || {
//!                 info!("Hello from MyPlugin");
//!             });
//!         });
//!     }
//! }
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_scripting::<LuaRuntime>(|_| {
//!         // nice and clean
//!     })
//!     .add_plugins(MyPlugin)
//!     .run();
//! ```
//!
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bevy_scriptum = { version = "0.8", features = ["lua"] }
//! ```
//!
//! or execute `cargo add bevy_scriptum --features lua` from your project directory.
//!
//! You can now start exposing functions to the scripting language. For example, you can expose a function that prints a message to the console:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_scripting::<LuaRuntime>(|runtime| {
//!        runtime.add_function(
//!            String::from("my_print"),
//!            |In((x,)): In<(String,)>| {
//!                println!("my_print: '{}'", x);
//!            },
//!        );
//!     })
//!     .run();
//! ```
//!
//! Then you can create a script file in `assets` directory called `script.lua` that calls this function:
//!
//! ```lua
//! my_print("Hello world!")
//! ```
//!
//! And spawn an entity with attached `Script` component with a handle to a script source file:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_scripting::<LuaRuntime>(|runtime| {
//!        runtime.add_function(
//!            String::from("my_print"),
//!            |In((x,)): In<(String,)>| {
//!                println!("my_print: '{}'", x);
//!            },
//!        );
//!     })
//!     .add_systems(Startup,|mut commands: Commands, asset_server: Res<AssetServer>| {
//!         commands.spawn(Script::<LuaScript>::new(asset_server.load("script.lua")));
//!     })
//!     .run();
//! ```
//!
//! You should then see `my_print: 'Hello world!'` printed in your console.
//!
//! ## Provided examples
//!
//! You can also try running provided examples by cloning this repository and running `cargo run --example <example_name>_<language_name>`.  For example:
//!
//! ```bash
//! cargo run --example hello_world_lua
//! ```
//! The examples live in `examples` directory and their corresponding scripts live in `assets/examples` directory within the repository.
//!
//! ## Bevy compatibility
//!
//! | bevy version | bevy_scriptum version |
//! |--------------|-----------------------|
//! | 0.16         | 0.8                   |
//! | 0.15         | 0.7                   |
//! | 0.14         | 0.6                   |
//! | 0.13         | 0.4-0.5               |
//! | 0.12         | 0.3                   |
//! | 0.11         | 0.2                   |
//! | 0.10         | 0.1                   |
//!
//! ## Promises - getting return values from scripts
//!
//! Every function called from script returns a promise that you can call `:and_then` with a callback function on. This callback function will be called when the promise is resolved, and will be passed the return value of the function called from script. For example:
//!
//! ```lua
//! get_player_name():and_then(function(name)
//!     print(name)
//! end)
//! ```
//! which will print out `John` when used with following exposed function:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use bevy_scriptum::runtimes::lua::prelude::*;
//!
//! App::new()
//!    .add_plugins(DefaultPlugins)
//!    .add_scripting::<LuaRuntime>(|runtime| {
//!            runtime.add_function(String::from("get_player_name"), || String::from("John"));
//!    });
//! ````
//!
//! ## Access entity from script
//!
//! A variable called `entity` is automatically available to all scripts - it represents bevy entity that the `Script` component is attached to.
//! It exposes `index` property that returns bevy entity index.
//! It is useful for accessing entity's components from scripts.
//! It can be used in the following way:
//! ```lua
//! print("Current entity index: " .. entity.index)
//! ```
//!
//! `entity` variable is currently not available within promise callbacks.
//!
//! ## Contributing
//!
//! Contributions are welcome! Feel free to open an issue or submit a pull request.
//!
//! ## License
//!
//! bevy_scriptum is licensed under either of the following, at your option:
//! Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0) or MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

mod assets;
mod callback;
mod components;
mod promise;
mod systems;

pub mod runtimes;

pub use crate::components::Script;
use assets::GetExtensions;
use promise::Promise;

use std::{
    any::TypeId,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{
    app::MainScheduleOrder,
    ecs::{component::Mutable, schedule::ScheduleLabel},
    prelude::*,
};
use callback::{Callback, IntoCallbackSystem};
use systems::{init_callbacks, log_errors, process_calls};
use thiserror::Error;

use self::{
    assets::ScriptLoader,
    systems::{process_new_scripts, reload_scripts},
};

#[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
const ENTITY_VAR_NAME: &str = "entity";

/// An error that can occur when internal [ScriptingPlugin] systems are being executed
#[derive(Error, Debug)]
pub enum ScriptingError {
    #[error("script runtime error: {0}")]
    RuntimeError(Box<dyn std::error::Error>),
    #[error("script compilation error: {0}")]
    CompileError(Box<dyn std::error::Error>),
    #[error("no runtime resource present")]
    NoRuntimeResource,
    #[error("no settings resource present")]
    NoSettingsResource,
}

/// Trait that represents a scripting runtime/engine. In practice it is
/// implemented for a scripint language interpreter and the implementor provides
/// function implementations for calling and registering functions within the interpreter.
pub trait Runtime: Resource + Default {
    type Schedule: ScheduleLabel + Debug + Clone + Eq + Hash + Default;
    type ScriptAsset: Asset + From<String> + GetExtensions;
    type ScriptData: Component<Mutability = Mutable>;
    type CallContext: Send + Clone;
    type Value: Send + Clone;
    type RawEngine;

    fn is_current_thread() -> bool;

    /// Provides mutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine_thread_mut<T: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Self::RawEngine) -> T + Send + 'static,
    ) -> T;

    /// Provides immutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine_thread<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Self::RawEngine) -> T + Send + 'static,
    ) -> T;

    /// Provides mutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T;

    /// Provides immutable reference to raw scripting engine instance.
    /// Can be used to directly interact with an interpreter to use interfaces
    /// that bevy_scriptum does not provided adapters for.
    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T;

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: Entity,
    ) -> Result<Self::ScriptData, ScriptingError>;

    /// Registers a new function within the scripting engine. Provided callback
    /// function will be called when the function with provided name gets called
    /// in script.
    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<TypeId>,
        f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<Promise<Self::CallContext, Self::Value>, ScriptingError>
            + Send
            + Sync
            + 'static,
    ) -> Result<(), ScriptingError>;

    /// Calls a function by name defined within the runtime in the context of the
    /// entity that haas been paassed. Can return a dynamically typed value
    /// that got returned from the function within a script.
    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: Entity,
        args: impl for<'a> FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, ScriptingError>;

    /// Calls a function by value defined within the runtime in the context of the
    /// entity that haas been paassed. Can return a dynamically typed value
    /// that got returned from the function within a script.
    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, ScriptingError>;
}

pub trait FuncArgs<'a, V, R: Runtime> {
    fn parse(self, engine: &'a R::RawEngine) -> Vec<V>;
}

/// An extension trait for [App] that allows to setup a scripting runtime `R`.
pub trait BuildScriptingRuntime {
    /// Returns a "runtime" type than can be used to setup scripting runtime(
    /// add scripting functions etc.).
    fn add_scripting<R: Runtime>(&mut self, f: impl Fn(ScriptingRuntimeBuilder<R>)) -> &mut Self;

    /// Returns a "runtime" type that can be used to add additional scripting functions from plugins etc.
    fn add_scripting_api<R: Runtime>(
        &mut self,
        f: impl Fn(ScriptingRuntimeBuilder<R>),
    ) -> &mut Self;
}

pub struct ScriptingRuntimeBuilder<'a, R: Runtime> {
    _phantom_data: PhantomData<R>,
    world: &'a mut World,
}

impl<'a, R: Runtime> ScriptingRuntimeBuilder<'a, R> {
    fn new(world: &'a mut World) -> Self {
        Self {
            _phantom_data: PhantomData,
            world,
        }
    }

    /// Registers a function for calling from within a script.
    /// Provided function needs to be a valid bevy system and its
    /// arguments and return value need to be convertible to runtime
    /// value types.
    pub fn add_function<In, Out, Marker>(
        self,
        name: String,
        fun: impl IntoCallbackSystem<R, In, Out, Marker>,
    ) -> Self
    where
        In: SystemInput,
    {
        let system = fun.into_callback_system(self.world);

        let mut callbacks_resource = self.world.resource_mut::<Callbacks<R>>();

        callbacks_resource.uninitialized_callbacks.push(Callback {
            name,
            system: Arc::new(Mutex::new(system)),
            calls: Arc::new(Mutex::new(vec![])),
        });

        self
    }
}

impl BuildScriptingRuntime for App {
    /// Adds a scripting runtime. Registers required bevy systems that take
    /// care of processing and running the scripts.
    fn add_scripting<R: Runtime>(&mut self, f: impl Fn(ScriptingRuntimeBuilder<R>)) -> &mut Self {
        self.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Update, R::Schedule::default());

        self.register_asset_loader(ScriptLoader::<R::ScriptAsset>::default())
            .init_schedule(R::Schedule::default())
            .init_asset::<R::ScriptAsset>()
            .init_resource::<Callbacks<R>>()
            .insert_resource(R::default())
            .add_systems(
                R::Schedule::default(),
                (
                    reload_scripts::<R>,
                    process_calls::<R>
                        .pipe(log_errors)
                        .after(process_new_scripts::<R>),
                    init_callbacks::<R>.pipe(log_errors),
                    process_new_scripts::<R>
                        .pipe(log_errors)
                        .after(init_callbacks::<R>),
                ),
            );

        let runtime = ScriptingRuntimeBuilder::<R>::new(self.world_mut());

        f(runtime);

        self
    }

    /// Adds a way to add additional accesspoints to the scripting runtime. For example from plugins to add
    /// for example additional lua functions to the runtime.
    ///
    /// Be careful with calling this though, make sure that the `add_scripting` call is already called before calling this function.
    fn add_scripting_api<R: Runtime>(
        &mut self,
        f: impl Fn(ScriptingRuntimeBuilder<R>),
    ) -> &mut Self {
        let runtime = ScriptingRuntimeBuilder::<R>::new(self.world_mut());

        f(runtime);

        self
    }
}

/// A resource that stores all the callbacks that were registered using [AddScriptFunctionAppExt::add_function].
#[derive(Resource)]
struct Callbacks<R: Runtime> {
    uninitialized_callbacks: Vec<Callback<R>>,
    callbacks: Mutex<Vec<Callback<R>>>,
}

impl<R: Runtime> Default for Callbacks<R> {
    fn default() -> Self {
        Self {
            uninitialized_callbacks: Default::default(),
            callbacks: Default::default(),
        }
    }
}

pub mod prelude {
    pub use crate::{BuildScriptingRuntime as _, Runtime as _, Script};
}
