use bevy::{
    prelude::*,
    utils::tracing::{self},
};
use std::{
    fmt::{Debug, Display},
};
use tracing::instrument;

use crate::{
    callback::FunctionCallEvent,
    components::ScriptData,
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
    A: Asset + CreateScriptData<E>,
    D: Send + Sync + 'static,
    E: Send + Sync + 'static + Default,
>(
    mut commands: Commands,
    mut added_scripted_entities: Query<(Entity, &mut Script<A>), Without<ScriptData<D>>>,
    mut scripting_runtime: ResMut<ScriptingRuntime<E>>,
    scripts: Res<Assets<A>>,
) -> Result<(), ScriptingError>
where
    A::ScriptData: Component + Debug,
    ScriptingRuntime<E>: GetEngine<E>,
{
    for (entity, script_component) in &mut added_scripted_entities {
        trace!("evaulating a new script");
        if let Some(script) = scripts.get(&script_component.script) {
            let engine = scripting_runtime.engine_mut();
            let script_data = script.create_script_data(entity, engine)?;

            commands.entity(entity).insert(script_data);
        }
    }
    Ok(())
}

/// Initializes callbacks. Registers them in the scripting engine.
#[instrument(skip(world))]
pub(crate) fn init_callbacks<E: Send + Sync + 'static + Default>(
    world: &mut World,
) -> Result<(), ScriptingError>
where
    ScriptingRuntime<E>: RegisterRawFn<rhai::NativeCallContextStore>,
{
    let mut callbacks_resource = world
        .get_resource_mut::<Callbacks<rhai::NativeCallContextStore>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let mut callbacks = callbacks_resource
        .uninitialized_callbacks
        .drain(..)
        .collect::<Vec<Callback<rhai::NativeCallContextStore>>>();

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
                    todo!();
                    // #[allow(deprecated)]
                    // let context_data = context.store_data();

                    // let promise = Promise {
                    //     inner: Arc::new(Mutex::new(PromiseInner {
                    //         callbacks: vec![],
                    //         context_data,
                    //     })),
                    // };
                    //
                    // let mut calls = callback.calls.lock().unwrap();
                    // calls.push(FunctionCallEvent {
                    //     promise: promise.clone(),
                    //     params: args.iter_mut().map(|arg| arg.clone()).collect(),
                    // });
                    // Ok(promise)
                },
            );
        }
    }

    let callbacks_resource = world
        .get_resource_mut::<Callbacks<rhai::NativeCallContextStore>>()
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
        .get_resource::<Callbacks<rhai::NativeCallContextStore>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let callbacks = callbacks_resource.callbacks.lock().unwrap().clone();

    for callback in callbacks.into_iter() {
        let calls = callback
            .calls
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<FunctionCallEvent<rhai::NativeCallContextStore>>>();
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
