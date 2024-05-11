use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    utils::tracing::{self},
};
use std::fmt::{Debug, Display};
use tracing::instrument;

use crate::{
    callback::FunctionCallEvent,
    components::ScriptData,
    promise::{Promise, PromiseInner},
    Callback, Callbacks, GetEngine, RegisterRawFn, ScriptingError,
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

pub trait CreateScriptData<E> {
    type ScriptData;

    fn create_script_data(
        &self,
        entity: Entity,
        engine: &mut E,
    ) -> Result<Self::ScriptData, ScriptingError>;
}

/// Processes new scripts.
#[instrument(skip(commands, added_scripted_entities, scripting_runtime, scripts))]
pub(crate) fn process_new_scripts<
    A: Asset + CreateScriptData<E> + Debug,
    D: Send + Sync + 'static,
    E: Send + Sync + 'static + Default + Debug,
>(
    mut commands: Commands,
    mut added_scripted_entities: Query<(Entity, &mut Script<A>), Without<ScriptData<D>>>,
    mut scripting_runtime: ResMut<ScriptingRuntime<E>>,
    scripts: Res<Assets<A>>,
) -> Result<(), ScriptingError>
where
    ScriptData<<A as CreateScriptData<E>>::ScriptData>: Component,
    ScriptingRuntime<E>: GetEngine<E>,
{
    for (entity, script_component) in &mut added_scripted_entities {
        tracing::trace!(script = ?script_component, "adding script");
        if let Some(script) = scripts.get(&script_component.script) {
            let engine = scripting_runtime.engine_mut();
            let script_data = script.create_script_data(entity, engine)?; // TODO: Should we return here?

            commands
                .entity(entity)
                .insert(ScriptData { data: script_data });
        }
    }
    Ok(())
}

/// Initializes callbacks. Registers them in the scripting engine.
#[instrument(skip(world))]
pub(crate) fn init_callbacks<
    E: Send + Sync + 'static + Default + Debug,
    C: Clone + 'static,
    D: Clone + Send + Default + 'static,
>(
    world: &mut World,
) -> Result<(), ScriptingError>
where
    ScriptingRuntime<E>: RegisterRawFn<D, C>,
{
    let mut callbacks_resource = world
        .get_resource_mut::<Callbacks<D, C>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let mut callbacks = callbacks_resource
        .uninitialized_callbacks
        .drain(..)
        .collect::<Vec<Callback<D, C>>>();

    for callback in callbacks.iter_mut() {
        if let Ok(mut system) = callback.system.lock() {
            system.system.initialize(world);

            let mut scripting_runtime = world
                .get_resource_mut::<ScriptingRuntime<E>>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            trace!("registering callback: '{}'", callback.name);
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
                            context_data: D::default(),
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
        .get_resource_mut::<Callbacks<D, C>>()
        .ok_or(ScriptingError::NoSettingsResource)?;
    callbacks_resource
        .callbacks
        .lock()
        .unwrap()
        .append(&mut callbacks.clone());

    Ok(())
}

/// Processes calls. Calls the user-defined callback systems
pub(crate) fn process_calls<D: Send + Clone + 'static, C: Clone + 'static>(
    world: &mut World,
) -> Result<(), ScriptingError> {
    let callbacks_resource = world
        .get_resource::<Callbacks<D, C>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let callbacks = callbacks_resource.callbacks.lock().unwrap().clone();

    for callback in callbacks.into_iter() {
        let calls = callback
            .calls
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<FunctionCallEvent<D, C>>>();
        for call in calls {
            trace!("process_calls: calling '{}'", callback.name);
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
