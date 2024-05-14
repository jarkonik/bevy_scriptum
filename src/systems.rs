use bevy::{prelude::*, utils::tracing};
use rhai::Scope;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    callback::FunctionCallEvent,
    promise::{Promise, PromiseInner},
    runtimes::rhai::RhaiScriptData,
    Callback, Callbacks, Runtime, ScriptingError, ENTITY_VAR_NAME,
};

use super::{components::Script, ScriptingRuntime};

/// Reloads scripts when they are modified.
pub(crate) fn reload_scripts<R: Runtime>(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<R::ScriptAsset>>,
    mut scripts: Query<(Entity, &mut Script<R::ScriptAsset>)>,
) {
    for ev in ev_asset.read() {
        if let AssetEvent::Modified { id } = ev {
            for (entity, script) in &mut scripts {
                if script.script.id() == *id {
                    commands.entity(entity).remove::<RhaiScriptData>();
                }
            }
        }
    }
}

/// Processes new scripts. Evaluates them and stores the [rhai::Scope] and cached [rhai::AST] in [ScriptData].
pub(crate) fn process_new_scripts<R: Runtime>(
    mut commands: Commands,
    mut added_scripted_entities: Query<
        (Entity, &mut Script<R::ScriptAsset>),
        Without<RhaiScriptData>,
    >,
    scripting_runtime: ResMut<R>,
    scripts: Res<Assets<R::ScriptAsset>>,
) -> Result<(), ScriptingError> {
    for (entity, script_component) in &mut added_scripted_entities {
        tracing::trace!("evaulating a new script");
        if let Some(script) = scripts.get(&script_component.script) {
            match scripting_runtime.create_script_data(script, entity) {
                Ok(script_data) => {
                    commands.entity(entity).insert(script_data);
                }
                Err(e) => tracing::error!(?e),
            }
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
                .get_resource_mut::<ScriptingRuntime>()
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
                .get_resource_mut::<ScriptingRuntime>()
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
