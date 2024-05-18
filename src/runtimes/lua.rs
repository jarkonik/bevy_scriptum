use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use mlua::{FromLua, Function, IntoLua, Lua, UserData};
use serde::Deserialize;
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{
    assets::GetExtensions,
    callback::{CloneCast, IntoValue},
    promise::Promise,
    EngineMut, EngineRef, FuncArgs, Runtime, ScriptingError,
};

type LuaEngine = Arc<Mutex<Lua>>;

#[derive(Clone, Debug)]
pub struct LuaValue<'a>(Arc<Mutex<mlua::Value<'a>>>);

impl<'a> LuaValue<'a> {
    fn new(value: mlua::Value<'a>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

/// # Safety: This is safe because we ensure thread safety using Arc and Mutex
unsafe impl Send for LuaValue<'_> {}

/// # Safety: This is safe because we ensure thread safety using Arc and Mutex
unsafe impl Sync for LuaValue<'_> {}

#[derive(Default, Resource)]
pub struct LuaRuntime {
    engine: LuaEngine,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct LuaSchedule;

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct LuaScript(pub String);

impl GetExtensions for LuaScript {
    fn extensions() -> &'static [&'static str] {
        &["lua"]
    }
}

impl From<String> for LuaScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Component)]
pub struct LuaScriptData;

impl EngineMut for LuaRuntime {
    type Engine = LuaEngine;

    fn engine_mut(&mut self) -> &mut Self::Engine {
        &mut self.engine
    }
}

impl EngineRef for LuaRuntime {
    type Engine = LuaEngine;

    fn engine_ref(&self) -> &Self::Engine {
        &self.engine
    }
}

impl<C: Send, V: Send> UserData for Promise<C, V> {}

impl Runtime for LuaRuntime {
    type Schedule = LuaSchedule;

    type ScriptAsset = LuaScript;

    type ScriptData = LuaScriptData;

    type CallContext = ();

    type Value = LuaValue<'static>;

    // TODO: Should be renamed or even split as it also evals
    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        engine.load(&script.0).exec().unwrap();
        Ok(LuaScriptData)
    }

    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<std::any::TypeId>,
        f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<Promise<Self::CallContext, Self::Value>, ScriptingError>
            + Send
            + Sync
            + 'static,
    ) -> Result<(), crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();

        let func = if !arg_types.is_empty() {
            engine
                .create_function(move |_, args: mlua::Variadic<mlua::Value>| {
                    let args: Vec<LuaValue> =
                        args.into_iter().map(|arg| LuaValue::new(arg)).collect();
                    // let promise = f((), args).unwrap();
                    let promise = f((), vec![]).unwrap();

                    Ok(promise)
                })
                .unwrap()
        } else {
            engine
                .create_function::<(), crate::promise::Promise<Self::CallContext, Self::Value>, _>(
                    move |_, _| {
                        let promise = f((), vec![]).unwrap();
                        Ok(promise)
                    },
                )
                .unwrap()
        };
        engine.globals().set(name, func).unwrap();
        Ok(())
    }

    fn call_fn<'v>(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl FuncArgs<Self::Value>,
    ) -> Result<(), crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        let func = engine.globals().get::<_, Function>(name).unwrap();
        let args: Vec<mlua::Value> = args.parse().into_iter().map(|a| mlua::Value::Nil).collect();
        let _ = func.call::<_, ()>(args);
        Ok(())
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        todo!()
    }
}

impl<'a, T: Any + Clone + Send + Sync> IntoValue<LuaValue<'a>> for T {
    fn into_value(self) -> LuaValue<'a> {
        LuaValue::new(mlua::Value::Nil)
    }
}

impl From<()> for LuaValue<'_> {
    fn from(value: ()) -> Self {
        LuaValue::new(mlua::Value::Nil)
    }
}

impl<'a> FuncArgs<LuaValue<'a>> for () {
    fn parse(self) -> Vec<LuaValue<'a>> {
        Vec::new()
    }
}

impl<'a, T: IntoLua<'static>> FuncArgs<LuaValue<'a>> for Vec<T> {
    fn parse(self) -> Vec<LuaValue<'a>> {
        self.into_iter()
            .map(|_| LuaValue::new(mlua::Value::Nil))
            .collect()
    }
}

impl CloneCast for LuaValue<'_> {
    fn clone_cast<T: Clone + 'static>(&self) -> T {
        let val = self.0.lock().unwrap();

        if TypeId::of::<T>() == TypeId::of::<i64>() {
            if let mlua::Value::Integer(n) = *val {
                unsafe { std::mem::transmute_copy::<_, T>(&n) }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}

pub mod prelude {
    pub use super::{LuaRuntime, LuaScript, LuaScriptData};
}
