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

pub use crate::components::{Script, ScriptData};
use assets::GetExtensions;

use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};
use callback::{Callback, RegisterCallbackFunction};
use rhai::{CallFnOptions, Dynamic, Engine, EvalAltResult, FuncArgs, ParseError};
use systems::{init_callbacks, init_engine, log_errors, process_calls};
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

pub trait Runtime: Resource {
    type Schedule: ScheduleLabel + Debug + Clone + Eq + Hash + Default;
    type ScriptAsset: Asset + From<String> + GetExtensions;
}

#[derive(Default)]
pub struct ScriptingPlugin<R: Runtime> {
    _phantom_data: PhantomData<R>,
}

pub struct ScriptingPluginBuilder<R> {
    _phantom_data: PhantomData<R>,
}

impl<R: Runtime> ScriptingPluginBuilder<R> {
    pub fn new() -> Self {
        Self {
            _phantom_data: Default::default(),
        }
    }

    pub fn build(self) -> ScriptingPlugin<R> {
        todo!()
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
            .init_resource::<Callbacks>()
            .insert_resource(ScriptingRuntime::default())
            .add_systems(Startup, init_engine.pipe(log_errors))
            .add_systems(
                Scripting,
                (
                    reload_scripts::<R>,
                    process_calls
                        .pipe(log_errors)
                        .after(process_new_scripts::<R>),
                    init_callbacks.pipe(log_errors),
                    process_new_scripts::<R>
                        .pipe(log_errors)
                        .after(init_callbacks),
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct ScriptingRuntime {
    engine: Engine,
}

impl ScriptingRuntime {
    /// Get a  mutable reference to the internal [rhai::Engine].
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// Call a function that is available in the scope of the script.
    pub fn call_fn(
        &mut self,
        function_name: &str,
        script_data: &mut ScriptData,
        entity: Entity,
        args: impl FuncArgs,
    ) -> Result<(), ScriptingError> {
        let ast = script_data.ast.clone();
        let scope = &mut script_data.scope;
        scope.push(ENTITY_VAR_NAME, entity);
        let options = CallFnOptions::new().eval_ast(false);
        let result =
            self.engine
                .call_fn_with_options::<Dynamic>(options, scope, &ast, function_name, args);
        scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
        if let Err(err) = result {
            match *err {
                rhai::EvalAltResult::ErrorFunctionNotFound(name, _) if name == function_name => {}
                e => Err(Box::new(e))?,
            }
        }
        Ok(())
    }
}

/// An extension trait for [App] that allows to register a script function.
pub trait AddScriptFunctionAppExt {
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
#[derive(Resource, Default)]
struct Callbacks {
    uninitialized_callbacks: Vec<Callback>,
    callbacks: Mutex<Vec<Callback>>,
}

impl AddScriptFunctionAppExt for App {
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
    ) -> &mut Self {
        let system = system.into_callback_system(&mut self.world);
        let mut callbacks_resource = self.world.resource_mut::<Callbacks>();

        callbacks_resource.uninitialized_callbacks.push(Callback {
            name,
            system: Arc::new(Mutex::new(system)),
            calls: Arc::new(Mutex::new(vec![])),
        });
        self
    }
}
