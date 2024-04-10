use bevy::{prelude::*, utils::tracing};
use rhai::Scope;
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

use super::{assets::RhaiScript, components::Script, ScriptingRuntime};

/// Initialize the scripting engine. Adds built-in types and functions.
pub(crate) fn init_engine(world: &mut World) -> Result<(), ScriptingError> {
    let mut scripting_runtime = world
        .get_resource_mut::<ScriptingRuntime>()
        .ok_or(ScriptingError::NoRuntimeResource)?;

    let engine = &mut scripting_runtime.engine;

    engine
        .register_type_with_name::<Entity>("Entity")
        .register_fn("index", |entity: &mut Entity| entity.index());
    engine
        .register_type_with_name::<Promise>("Promise")
        .register_fn("then", Promise::then);
    engine
        .register_type_with_name::<Vec3>("Vec3")
        .register_fn("new_vec3", |x: f64, y: f64, z: f64| {
            Vec3::new(x as f32, y as f32, z as f32)
        })
        .register_get("x", |vec: &mut Vec3| vec.x as f64)
        .register_get("y", |vec: &mut Vec3| vec.y as f64)
        .register_get("z", |vec: &mut Vec3| vec.z as f64);
    #[allow(deprecated)]
    engine.on_def_var(|_, info, _| Ok(info.name != "entity"));

    Ok(())
}

/// Reloads scripts when they are modified.
pub(crate) fn reload_scripts(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<RhaiScript>>,
    mut scripts: Query<(Entity, &mut Script)>,
) {
    for ev in ev_asset.read() {
        if let AssetEvent::Modified { id } = ev {
            for (entity, script) in &mut scripts {
                if script.script.id() == *id {
                    commands.entity(entity).remove::<ScriptData>();
                }
            }
        }
    }
}

/// Processes new scripts. Evaluates them and stores the [rhai::Scope] and cached [rhai::AST] in [ScriptData].
pub(crate) fn process_new_scripts(
    mut commands: Commands,
    mut added_scripted_entities: Query<(Entity, &mut Script), Without<ScriptData>>,
    scripting_runtime: ResMut<ScriptingRuntime>,
    scripts: Res<Assets<RhaiScript>>,
) -> Result<(), ScriptingError> {
    for (entity, script_component) in &mut added_scripted_entities {
        trace!("process_new_scripts: evaulating a new script");
        if let Some(script) = scripts.get(&script_component.script) {
            let mut scope = Scope::new();

            scope.push(ENTITY_VAR_NAME, entity);

            let engine = &scripting_runtime.engine;

            let ast = engine
                .compile_with_scope(&scope, script.0.as_str())
                .map_err(ScriptingError::CompileError)?;

            engine
                .run_ast_with_scope(&mut scope, &ast)
                .map_err(ScriptingError::RuntimeError)?;

            scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();

            commands.entity(entity).insert(ScriptData { ast, scope });
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
