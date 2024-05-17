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
pub struct LuaValue(Arc<Mutex<mlua::Value<'static>>>);

impl LuaValue {
    fn new(value: mlua::Value<'static>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

/// # Safety: This is safe because we ensure thread safety using Arc and Mutex
unsafe impl Send for LuaValue {}

/// # Safety: This is safe because we ensure thread safety using Arc and Mutex
unsafe impl Sync for LuaValue {}

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

    type Value = LuaValue;

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
            .create_function::<(mlua::Value), crate::promise::Promise<Self::CallContext, Self::Value>, _>(move |_, mut args| {
                // let args = args.into_iter().map(|arg| LuaValue::new(mlua::Value::Number(5.0))).collect();
                let promise = f((), vec![LuaValue::new(mlua::Value::Number(5.0))]).unwrap();
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

impl<T: Any + Clone + Send + Sync> IntoValue<LuaValue> for T {
    fn into_value(self) -> LuaValue {
        LuaValue::new(mlua::Value::Nil)
    }
}

impl From<()> for LuaValue {
    fn from(value: ()) -> Self {
        LuaValue::new(mlua::Value::Nil)
    }
}

impl FuncArgs<LuaValue> for () {
    fn parse(self) -> Vec<LuaValue> {
        Vec::new()
    }
}

impl<T: IntoLua<'static>> FuncArgs<LuaValue> for Vec<T> {
    fn parse(self) -> Vec<LuaValue> {
        self.into_iter()
            .map(|_| LuaValue::new(mlua::Value::Nil))
            .collect()
    }
}

impl CloneCast for LuaValue {
    fn clone_cast<T: Clone + 'static>(&self) -> T {
        let val = self.0.lock().unwrap();

        match TypeId::of::<T>() {
            i64 => {
                if let mlua::Value::Number(n) = *val {
                    let i = val.as_number().unwrap() as i64;
                    unsafe { std::mem::transmute_copy::<_, T>(&i) }
                } else {
                    panic!();
                }
            }
            _ => todo!("{:?}", TypeId::of::<T>()),
        }
    }
}

pub mod prelude {
    pub use super::{LuaRuntime, LuaScript, LuaScriptData};
}
