use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    utils::tracing::{self},
};
use rhai::Engine;
use std::fmt::{Debug, Display};
use tracing::instrument;

use crate::{
    callback::FunctionCallEvent,
    components::ScriptData,
    promise::{Promise, PromiseInner},
    rhai_support::RhaiScript,
    Callback, Callbacks, GetEngine, RegisterRawFn, Runtime, RuntimeConfig, ScriptingError,
};

use super::{components::Script, ScriptingRuntime};

/// Reloads scripts when they are modified.
pub(crate) fn reload_scripts<C: RuntimeConfig>(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<C::ScriptAsset>>,
    mut scripts: Query<(Entity, &mut Script<C::ScriptAsset>)>,
) {
    for ev in ev_asset.read() {
        if let AssetEvent::Modified { id } = ev {
            for (entity, script) in &mut scripts {
                if script.script.id() == *id {
                    commands
                        .entity(entity)
                        .remove::<ScriptData<C::ScriptAsset>>();
                }
            }
        }
    }
}

pub trait CreateScriptData<A, D> {
    fn create_script_data(
        &self,
        entity: Entity,
        script: &A,
    ) -> Result<ScriptData<D>, ScriptingError>;
}

/// Processes new scripts.
#[instrument(skip(commands, added_scripted_entities, scripting_runtime, scripts))]
pub(crate) fn process_new_scripts<C: RuntimeConfig>(
    mut commands: Commands,
    mut added_scripted_entities: Query<
        (Entity, &mut Script<C::ScriptAsset>),
        Without<ScriptData<C::ScriptData>>,
    >,
    mut scripting_runtime: ResMut<C::Runtime>,
    scripts: Res<Assets<C::ScriptAsset>>,
) -> Result<(), ScriptingError> {
    for (entity, script_component) in &mut added_scripted_entities {
        tracing::trace!(script = ?script_component, "adding script");
        if let Some(script) = scripts.get(&script_component.script) {
            let script_data = scripting_runtime
                .create_script_data(entity, script)
                .unwrap();

            commands.entity(entity).insert(script_data);
        }
    }
    Ok(())
}

/// Initializes callbacks. Registers them in the scripting engine.
#[instrument(skip(world))]
pub(crate) fn init_callbacks<C: RuntimeConfig>(world: &mut World) -> Result<(), ScriptingError> {
    let mut callbacks_resource = world
        .get_resource_mut::<Callbacks<(), ()>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let mut callbacks = callbacks_resource
        .uninitialized_callbacks
        .drain(..)
        .collect::<Vec<Callback<(), ()>>>();

    for callback in callbacks.iter_mut() {
        if let Ok(mut system) = callback.system.lock() {
            system.system.initialize(world);

            let mut scripting_runtime = world
                .get_resource_mut::<C::Runtime>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            tracing::trace!("registering callback: '{}'", callback.name);
            let callback = callback.clone();

            scripting_runtime.register_raw_fn(
                &callback.name,
                system.arg_types.clone(),
                // move |context, args| {
                move || {
                    // #[allow(deprecated)]
                    // let context_data = context.store_data();

                    let promise = Promise {
                        inner: Arc::new(Mutex::new(PromiseInner {
                            callbacks: vec![],
                            context_data: (),
                        })),
                    };

                    let mut calls = callback.calls.lock().unwrap();
                    calls.push(FunctionCallEvent {
                        promise: promise.clone(),
                        // params: args.iter_mut().map(|arg| arg.clone()).collect(),
                        params: vec![],
                    });

                    promise
                },
            );
        }
    }

    let callbacks_resource = world
        .get_resource_mut::<Callbacks<(), ()>>()
        .ok_or(ScriptingError::NoSettingsResource)?;
    callbacks_resource
        .callbacks
        .lock()
        .unwrap()
        .append(&mut callbacks.clone());

    Ok(())
}

/// Processes calls. Calls the user-defined callback systems
#[instrument]
pub(crate) fn process_calls(world: &mut World) -> Result<(), ScriptingError> {
    let callbacks_resource = world
        .get_resource::<Callbacks<(), ()>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let callbacks = callbacks_resource.callbacks.lock().unwrap().clone();

    for callback in callbacks.into_iter() {
        let calls = callback
            .calls
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<FunctionCallEvent<(), ()>>>();
        for call in calls {
            tracing::trace!(?callback.name, "calling");
            let mut system = callback.system.lock().unwrap();
            let _val = system.call(&call, world);
            let _runtime = world
                .get_resource_mut::<ScriptingRuntime<rhai::Engine>>()
                .ok_or(ScriptingError::NoRuntimeResource)?;
            // call.promise.resolve(&mut runtime.engine, val)?;
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
