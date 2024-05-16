use bevy::{prelude::*, utils::tracing};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{
    callback::FunctionCallEvent,
    promise::{Promise, PromiseInner},
    runtimes::rhai::RhaiScriptData,
    Callback, Callbacks, Runtime, ScriptingError,
};

use super::components::Script;

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
        Without<R::ScriptData>,
    >,
    scripting_runtime: ResMut<R>,
    scripts: Res<Assets<R::ScriptAsset>>,
    asset_server: Res<AssetServer>,
) -> Result<(), ScriptingError> {
    for (entity, script_component) in &mut added_scripted_entities {
        tracing::trace!("evaulating a new script");
        if let Some(script) = scripts.get(&script_component.script) {
            match scripting_runtime.create_script_data(script, entity) {
                Ok(script_data) => {
                    commands.entity(entity).insert(script_data);
                }
                Err(e) => {
                    let path = asset_server
                        .get_path(&script_component.script)
                        .unwrap_or_default();
                    tracing::error!("error running script {} {:?}", path, e);
                }
            }
        }
    }
    Ok(())
}

/// Initializes callbacks. Registers them in the scripting engine.
pub(crate) fn init_callbacks<R: Runtime>(world: &mut World) -> Result<(), ScriptingError> {
    let mut callbacks_resource = world
        .get_resource_mut::<Callbacks<R::CallContext, R::Value>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let mut callbacks = callbacks_resource
        .uninitialized_callbacks
        .drain(..)
        .collect::<Vec<Callback<R::CallContext, R::Value>>>();

    for callback in callbacks.iter_mut() {
        if let Ok(mut system) = callback.system.lock() {
            system.system.initialize(world);

            let mut scripting_runtime = world
                .get_resource_mut::<R>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            tracing::trace!("init_callbacks: registering callback: '{}'", callback.name);

            let callback = callback.clone();

            let result = scripting_runtime.register_fn(
                callback.name,
                system.arg_types.clone(),
                move |context, params| {
                    let promise = Promise {
                        inner: Arc::new(Mutex::new(PromiseInner {
                            callbacks: vec![],
                            context,
                        })),
                    };

                    let mut calls = callback.calls.lock().unwrap();

                    calls.push(FunctionCallEvent {
                        promise: promise.clone(),
                        params,
                    });
                    Ok(promise)
                },
            );
            if let Err(e) = result {
                tracing::error!("error registering function: {:?}", e);
            }
        }
    }

    let callbacks_resource = world
        .get_resource_mut::<Callbacks<R::CallContext, R::Value>>()
        .ok_or(ScriptingError::NoSettingsResource)?;
    callbacks_resource
        .callbacks
        .lock()
        .unwrap()
        .append(&mut callbacks.clone());

    Ok(())
}

/// Processes calls. Calls the user-defined callback systems
pub(crate) fn process_calls<R: Runtime>(world: &mut World) -> Result<(), ScriptingError> {
    let callbacks_resource = world
        .get_resource::<Callbacks<R::CallContext, R::Value>>()
        .ok_or(ScriptingError::NoSettingsResource)?;

    let callbacks = callbacks_resource.callbacks.lock().unwrap().clone();

    for callback in callbacks.into_iter() {
        let calls = callback
            .calls
            .lock()
            .unwrap()
            .drain(..)
            .collect::<Vec<FunctionCallEvent<R::CallContext, R::Value>>>();
        for mut call in calls {
            tracing::trace!("process_calls: calling '{}'", callback.name);
            let mut system = callback.system.lock().unwrap();
            let val = system.call(&call, world);
            let mut runtime = world
                .get_resource_mut::<R>()
                .ok_or(ScriptingError::NoRuntimeResource)?;

            let result = call.promise.resolve(runtime.as_mut(), val);
            match result {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("error resolving call: {} {:?}", callback.name, e);
                }
            }
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
