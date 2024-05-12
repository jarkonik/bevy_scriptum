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

pub mod lua_support;
pub mod rhai_support;

use std::{
    any::TypeId,
    fmt::Debug,
    marker::PhantomData,
    mem::take,
    sync::{Arc, Mutex},
};

pub use crate::components::{Script, ScriptData};

use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use callback::{Callback, RegisterCallbackFunction};
use lua_support::{LuaCallback, LuaEngine, LuaScript, LuaScriptData};
use promise::Promise;
use rhai::{EvalAltResult, ParseError};
use rhai_support::{RhaiCallback, RhaiScript, RhaiScriptData};
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

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Scripting;

#[derive(Default)]
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Update, Scripting);

        app.register_asset_loader(ScriptLoader::<RhaiScript>::default())
            .register_asset_loader(ScriptLoader::<LuaScript>::default())
            .init_schedule(Scripting)
            .init_asset::<RhaiScript>()
            .init_asset::<LuaScript>()
            .init_resource::<Callbacks<(), RhaiCallback>>()
            .init_resource::<Callbacks<(), LuaCallback>>()
            .add_systems(
                Scripting,
                (
                    reload_scripts::<RhaiScript>,
                    reload_scripts::<LuaScript>,
                    process_calls::<(), RhaiCallback>
                        .pipe(log_errors)
                        .after(process_new_scripts::<RhaiScript, RhaiScriptData, rhai::Engine>),
                    process_calls::<(), LuaCallback>
                        .pipe(log_errors)
                        .after(process_new_scripts::<LuaScript, LuaScriptData, LuaEngine>),
                    init_callbacks::<rhai::Engine, RhaiCallback, ()>.pipe(log_errors),
                    init_callbacks::<LuaEngine, LuaCallback, ()>.pipe(log_errors),
                    process_new_scripts::<RhaiScript, RhaiScriptData, rhai::Engine>
                        .pipe(log_errors)
                        .after(init_callbacks::<rhai::Engine, RhaiCallback, ()>),
                    process_new_scripts::<LuaScript, LuaScriptData, LuaEngine>
                        .pipe(log_errors)
                        .after(init_callbacks::<LuaEngine, LuaCallback, ()>),
                ),
            );
    }
}

#[derive(Resource, Debug)]
pub struct ScriptingRuntime<T: Default + Debug> {
    engine: T,
}

pub trait RegisterRawFn<D, C> {
    fn register_raw_fn(
        &mut self,
        name: &str,
        arg_types: Vec<TypeId>,
        f: impl Fn() -> Promise<D, C> + Send + Sync + 'static,
    );
}

pub trait GetEngine<T> {
    fn engine_mut(&mut self) -> &mut T;
}

/// An extension trait for [App] that allows to register a script function.
pub trait AddScriptFunction {
    fn add_script_function<
        Out,
        Marker,
        A: 'static,
        const N: usize,
        const X: bool,
        R: 'static,
        const F: bool,
        Args,
    >(
        &mut self,
        name: String,
        system: impl RegisterCallbackFunction<Out, Marker, A, N, X, R, F, Args>,
    ) -> &mut Self;
}

/// A resource that stores all the callbacks that were registered using [AddScriptFunctionAppExt::add_script_function].
#[derive(Resource)]
struct Callbacks<D, C> {
    uninitialized_callbacks: Vec<Callback<D, C>>,
    callbacks: Mutex<Vec<Callback<D, C>>>,
}

impl<D, C> Default for Callbacks<D, C> {
    fn default() -> Self {
        Self {
            callbacks: Mutex::new(Vec::new()),
            uninitialized_callbacks: Vec::new(),
        }
    }
}

pub trait BuildScriptingRuntime {
    type Runtime: Resource;
    type Callbacks;

    fn build(self) -> (World, Self::Runtime);
}

pub struct ScriptingRuntimeBuilder<R> {
    _phantom_data: PhantomData<R>,
    world: Option<World>,
}

impl<'a, R> AddScriptFunction for ScriptingRuntimeBuilder<R> {
    fn add_script_function<
        Out,
        Marker,
        A: 'static,
        const N: usize,
        const X: bool,
        Y: 'static,
        const F: bool,
        Args,
    >(
        &mut self,
        name: String,
        system: impl RegisterCallbackFunction<Out, Marker, A, N, X, Y, F, Args>,
    ) -> &mut Self {
        let mut world = self.world.take().expect("World has not been set");
        let system = system.into_callback_system(&mut world);
        let mut callbacks_resource = world.resource_mut::<Callbacks<(), ()>>();

        callbacks_resource.uninitialized_callbacks.push(Callback {
            name,
            system: Arc::new(Mutex::new(system)),
            calls: Arc::new(Mutex::new(vec![])),
        });
        self.world = Some(world);

        self
    }
}

impl<R> Default for ScriptingRuntimeBuilder<R> {
    fn default() -> Self {
        Self {
            _phantom_data: Default::default(),
            world: Default::default(),
        }
    }
}

pub trait AddScriptingRuntimeAppExt {
    fn add_scripting_runtime<B: AddScriptFunction + BuildScriptingRuntime + SetWorld + Default>(
        &mut self,
        f: impl Fn(&mut B),
    ) -> &mut App;
}

pub trait SetWorld {
    fn set_world(&mut self, world: World);
}

impl<R> SetWorld for ScriptingRuntimeBuilder<R> {
    fn set_world(&mut self, world: World) {
        self.world = Some(world);
    }
}

impl AddScriptingRuntimeAppExt for App {
    fn add_scripting_runtime<
        'a,
        B: AddScriptFunction + BuildScriptingRuntime + SetWorld + Default,
    >(
        &mut self,
        f: impl Fn(&mut B),
    ) -> &mut Self {
        let world = std::mem::take(&mut self.world);

        let mut builder = B::default();
        builder.set_world(world);

        f(&mut builder);

        let (world, runtime) = builder.build();

        self.insert_resource(runtime);

        self.world = world;

        self
    }
}

pub trait CallFunction<T> {
    fn call_fn(
        &mut self,
        function_name: &str,
        script_data: &mut ScriptData<T>,
        entity: Entity,
        args: (),
    ) -> Result<(), ScriptingError>;
}

pub mod prelude {
    pub use crate::{AddScriptFunction, ScriptingPlugin};
}
