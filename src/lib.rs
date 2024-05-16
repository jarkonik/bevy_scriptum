//! bevy_scriptum is a a plugin for [Bevy](https://bevyengine.org/) that allows you to write some of your game logic in a scripting language.
//! Currently, only [Rhai](https://rhai.rs/) is supported, but more languages may be added in the future.
//!
//! It's main advantages include:
//! - low-boilerplate
//! - easy to use
//! - asynchronicity with a promise-based API
//! - flexibility
//! - hot-reloading
//!
//! Scripts are separate files that can be hot-reloaded at runtime. This allows you to quickly iterate on your game logic without having to recompile your game.
//!
//! All you need to do is register callbacks on your Bevy app like this:
//! ```rust
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ScriptingPlugin::default())
//!     .add_script_function(String::from("hello_bevy"), || {
//!       println!("hello bevy, called from script");
//!     });
//! ```
//! And you can call them in your scripts like this:
//! ```rhai
//! hello_bevy();
//! ```
//!
//! Every callback function that you expose to the scripting language is also a Bevy system, so you can easily query and mutate ECS components and resources just like you would in a regular Bevy system:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//!
//! #[derive(Component)]
//! struct Player;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ScriptingPlugin::default())
//!     .add_script_function(
//!         String::from("print_player_names"),
//!         |players: Query<&Name, With<Player>>| {
//!             for player in &players {
//!                 println!("player name: {}", player);
//!             }
//!         },
//!     );
//! ```
//!
//! You can also pass arguments to your callback functions, just like you would in a regular Bevy system - using `In` structs with tuples:
//! ```rust
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//! use rhai::ImmutableString;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ScriptingPlugin::default())
//!     .add_script_function(
//!         String::from("fun_with_string_param"),
//!         |In((x,)): In<(ImmutableString,)>| {
//!             println!("called with string: '{}'", x);
//!         },
//!     );
//! ```
//! which you can then call in your script like this:
//! ```rhai
//! fun_with_string_param("Hello world!");
//! ```
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bevy_scriptum = "0.2"
//! ```
//!
//! or execute `cargo add bevy_scriptum` from your project directory.
//!
//! Add the following to your `main.rs`:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ScriptingPlugin::default())
//!     .run();
//! ```
//!
//! You can now start exposing functions to the scripting language. For example, you can expose a function that prints a message to the console:
//!
//! ```rust
//! use rhai::ImmutableString;
//! use bevy::prelude::*;
//! use bevy_scriptum::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ScriptingPlugin::default())
//!     .add_script_function(
//!         String::from("my_print"),
//!         |In((x,)): In<(ImmutableString,)>| {
//!             println!("my_print: '{}'", x);
//!         },
//!     );
//! ```
//!
//! Then you can create a script file in `assets` directory called `script.rhai` that calls this function:
//!
//! ```rhai
//! my_print("Hello world!");
//! ```
//!
//! And spawn a `Script` component with a handle to a script source file`:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_scriptum::Script;
//!
//! App::new()
//!     .add_systems(Startup,|mut commands: Commands, asset_server: Res<AssetServer>| {
//!         commands.spawn(Script::new(asset_server.load("script.rhai")));
//!     });
//! ```
//!
//! ## Provided examples
//!
//! You can also try running provided examples by cloning this repository and running `cargo run --example <example_name>`.  For example:
//!
//! ```bash
//! cargo run --example hello_world
//! ```
//! The examples live in `examples` directory and their corresponding scripts live in `assets/examples` directory within the repository.
//!
//! ## Bevy compatibility
//!
//! | bevy version | bevy_scriptum version |
//! |--------------|----------------------|
//! | 0.13         | 0.4                  |
//! | 0.12         | 0.3                  |
//! | 0.11         | 0.2                  |
//! | 0.10         | 0.1                  |
//!
//! ## Promises - getting return values from scripts
//!
//! Every function called from script returns a promise that you can call `.then` with a callback function on. This callback function will be called when the promise is resolved, and will be passed the return value of the function called from script. For example:
//!
//! ```rhai
//! get_player_name().then(|name| {
//!     print(name);
//! });
//! ```
//!
//! ## Access entity from script
//!
//! A variable called `entity` is automatically available to all scripts - it represents bevy entity that the `Script` component is attached to.
//! It exposes `.index()` method that returns bevy entity index.
//! It is useful for accessing entity's components from scripts.
//! It can be used in the following way:
//! ```rhai
//! print("Current entity index: " + entity.index());
//! ```
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
    any::{Any, TypeId},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use callback::{Callback, CallbackFunction, CallbackSystem};
use rhai::{Engine, EvalAltResult, FuncArgs, ParseError};
use systems::{init_callbacks, log_errors, process_calls};
use thiserror::Error;

use self::{
    assets::ScriptLoader,
    systems::{process_new_scripts, reload_scripts},
};

const ENTITY_VAR_NAME: &str = "entity";

/// An error that can occur when internal [ScriptingPlugin] systems are being executed
#[derive(Error, Debug)]
pub enum ScriptingError {
    #[error("script runtime error: {0}")]
    RuntimeError(#[from] Box<EvalAltResult>),
    #[error("script compilation error: {0}")]
    CompileError(#[from] ParseError),
    #[error("no runtime resource present")]
    NoRuntimeResource,
    #[error("no settings resource present")]
    NoSettingsResource,
}

pub trait EngineMut {
    type Engine;

    fn engine_mut(&mut self) -> &mut Engine;
}

pub trait Runtime: Resource + Default + EngineMut {
    type Schedule: ScheduleLabel + Debug + Clone + Eq + Hash + Default;
    type ScriptAsset: Asset + From<String> + GetExtensions;
    type ScriptData: Component;
    type CallContext: Send + Clone;
    type Value: Send + Clone;

    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: Entity,
    ) -> Result<Self::ScriptData, ScriptingError>;

    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<TypeId>,
        f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<Promise<Self::CallContext>, ScriptingError>
            + Send
            + Sync
            + 'static,
    ) -> Result<(), ScriptingError>;

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: Entity,
        args: impl FuncArgs,
    ) -> Result<(), ScriptingError>;
}

#[derive(Default)]
pub struct ScriptingPlugin<R: Runtime> {
    _phantom_data: PhantomData<R>,
}

#[derive(Default)]
pub struct ScriptingPluginBuilder<R: Runtime> {
    _phantom_data: PhantomData<R>,
    systems: Vec<(
        String,
        Box<dyn System<In = Vec<Box<dyn Any>>, Out = Box<dyn Any>>>,
    )>,
}

/// An extension trait for [App] that allows to setup a scripting runtime `R`.
pub trait SetupScriptingRuntime {
    /// Returns a "builder" type than can be used to setup scripting runtime(
    /// add scripting functions etc.).
    fn setup_scripting_runtime<R: Runtime>(&mut self) -> ScriptingRuntimeSetupBuilder<R>;
}

pub struct ScriptingRuntimeSetupBuilder<'a, R: Runtime> {
    _phantom_data: PhantomData<R>,
    world: &'a mut World,
}

impl<'a, R: Runtime> ScriptingRuntimeSetupBuilder<'a, R> {
    fn new(world: &'a mut World) -> Self {
        Self {
            _phantom_data: PhantomData,
            world,
        }
    }

    pub fn add_script_function<In, Out, Marker>(
        self,
        name: String,
        fun: impl CallbackFunction<R::Value, In, Out, Marker>,
    ) -> Self {
        let system = fun.into_callback_system(self.world);

        let mut callbacks_resource = self
            .world
            .resource_mut::<Callbacks<R::CallContext, R::Value>>();

        callbacks_resource.uninitialized_callbacks.push(Callback {
            name,
            system: Arc::new(Mutex::new(system)),
            calls: Arc::new(Mutex::new(vec![])),
        });

        self
    }
}

impl SetupScriptingRuntime for App {
    #[must_use]
    fn setup_scripting_runtime<R: Runtime>(&mut self) -> ScriptingRuntimeSetupBuilder<R> {
        ScriptingRuntimeSetupBuilder::new(&mut self.world)
    }
}

impl<R: Runtime> ScriptingPluginBuilder<R> {
    pub fn new() -> Self {
        Self {
            _phantom_data: Default::default(),
            systems: Vec::new(),
        }
    }

    pub fn add_script_function<In, Out, Marker>(
        mut self,
        name: String,
        fun: impl CallbackFunction<R::Value, In, Out, Marker>,
    ) -> Self {
        // let system = IntoSystem::into_system(fun);
        // self.systems.push((name, Box::new(system)));

        self
    }

    pub fn build(self) -> ScriptingPlugin<R> {
        // let system: CallbackSystem<R::Value> = system.into_callback_system(&mut self.world);
        // let mut callbacks_resource = self
        //     .world
        //     .resource_mut::<Callbacks<R::CallContext, R::Value>>();
        // callbacks_resource.uninitialized_callbacks.push(Callback {
        //     name,
        //     system: Arc::new(Mutex::new(system)),
        //     calls: Arc::new(Mutex::new(vec![])),
        // });

        ScriptingPlugin::default()
    }
}

impl<R: Runtime> Plugin for ScriptingPlugin<R> {
    fn build(&self, app: &mut App) {
        app.world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Update, R::Schedule::default());

        app.register_asset_loader(ScriptLoader::<R::ScriptAsset>::default())
            .init_schedule(R::Schedule::default())
            .init_asset::<R::ScriptAsset>()
            .init_resource::<Callbacks<R::CallContext, R::Value>>()
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
    }
}

/// A resource that stores all the callbacks that were registered using [AddScriptFunctionAppExt::add_script_function].
#[derive(Resource)]
struct Callbacks<C: Send, V> {
    uninitialized_callbacks: Vec<Callback<C, V>>,
    callbacks: Mutex<Vec<Callback<C, V>>>,
}

impl<C: Send, V> Default for Callbacks<C, V> {
    fn default() -> Self {
        Self {
            uninitialized_callbacks: Default::default(),
            callbacks: Default::default(),
        }
    }
}

pub mod prelude {
    pub use crate::Runtime;
}
