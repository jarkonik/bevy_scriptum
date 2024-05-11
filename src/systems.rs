use bevy::{
    prelude::*,
    utils::tracing::{self},
};
use rhai::{Engine, Scope};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    callback::FunctionCallEvent,
    components::ScriptData,
    promise::{Promise, PromiseInner},
    Callback, Callbacks, ScriptingError, ENTITY_VAR_NAME,
};

use super::{components::Script, ScriptingRuntime};

/// Reloads scripts when they are modified.
pub(crate) fn reload_scripts<T: Asset>(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<T>>,
    mut scripts: Query<(Entity, &mut Script<T>)>,
) {
    for ev in ev_asset.read() {
        if let AssetEvent::Modified { id } = ev {
            for (entity, script) in &mut scripts {
                if script.script.id() == *id {
                    commands.entity(entity).remove::<ScriptData<T>>();
                }
            }
        }
    }
}

pub trait CreateScriptData {
    type ScriptData;
    type Engine;

    fn create_script_data(
        &self,
        entity: Entity,
        engine: &mut Engine,
    ) -> Result<Self::ScriptData, ScriptingError>;
}

/// Processes new scripts.
pub(crate) fn process_new_scripts<A: Asset + CreateScriptData, D: Send + Sync + 'static>(
    mut commands: Commands,
    mut added_scripted_entities: Query<(Entity, &mut Script<A>), Without<ScriptData<D>>>,
    mut scripting_runtime: ResMut<ScriptingRuntime<Engine>>,
    scripts: Res<Assets<A>>,
) -> Result<(), ScriptingError>
where
    A::ScriptData: Component,
{
    for (entity, script_component) in &mut added_scripted_entities {
        trace!("evaulating a new script");
        if let Some(script) = scripts.get(&script_component.script) {
            let script_data = script.create_script_data(entity, scripting_runtime.engine_mut())?;
            commands.entity(entity).insert(script_data);
        }
    }
    Ok(())
}

/// Initializes callbacks. Registers them in the scripting engine.
pub(crate) fn init_callbacks(world: &mut World) -> Result<(), ScriptingError> {
    let mut callbacks_resource = world
        .get_resource_mut::<Callbacks>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let mut callbacks = callbacks_resource
        .uninitialized_callbacks
        .drain(..)
        .collect::<Vec<Callback>>();

    for callback in callbacks.iter_mut() {
        if let Ok(mut system) = callback.system.lock() {
            system.system.initialize(world);

            let mut scripting_runtime = world
                .get_resource_mut::<ScriptingRuntime<rhai::Engine>>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            trace!("init_callbacks: registering callback: '{}'", callback.name);
            let engine = &mut scripting_runtime.engine;
            let callback = callback.clone();
            engine.register_raw_fn(
                callback.name,
                system.arg_types.clone(),
                move |context, args| {
                    #[allow(deprecated)]
                    let context_data = context.store_data();
                    let promise = Promise {
                        inner: Arc::new(Mutex::new(PromiseInner {
                            callbacks: vec![],
                            context_data,
                        })),
                    };

                    let mut calls = callback.calls.lock().unwrap();
                    calls.push(FunctionCallEvent {
                        promise: promise.clone(),
                        params: args.iter_mut().map(|arg| arg.clone()).collect(),
                    });
                    Ok(promise)
                },
            );
        }
    }

    let callbacks_resource = world
        .get_resource_mut::<Callbacks>()
        .ok_or(ScriptingError::NoSettingsResource)?;
    callbacks_resource
        .callbacks
        .lock()
        .unwrap()
        .append(&mut callbacks.clone());

    Ok(())
}

/// Processes calls. Calls the user-defined callback systems
pub(crate) fn process_calls(world: &mut World) -> Result<(), ScriptingError> {
    let callbacks_resource = world
        .get_resource::<Callbacks>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let callbacks = callbacks_resource.callbacks.lock().unwrap().clone();

    for callback in callbacks.into_iter() {
        let calls = callback
            .calls
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<FunctionCallEvent>>();
        for mut call in calls {
            trace!("process_calls: calling '{}'", callback.name);
            let mut system = callback.system.lock().unwrap();
            let val = system.call(&call, world);
            let mut runtime = world
                .get_resource_mut::<ScriptingRuntime<rhai::Engine>>()
                .ok_or(ScriptingError::NoRuntimeResource)?;
            call.promise.resolve(&mut runtime.engine, val)?;
        }
    }
    Ok(())
}

/// Error logging system
pub fn log_errors<E: Display>(In(res): In<Result<(), E>>) {
    if let Err(error) = res {
        tracing::error!("{}", error);
    }
}
