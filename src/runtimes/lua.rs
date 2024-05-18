use bevy::{
    asset::Asset,
    ecs::{component::Component, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use mlua::{Function, IntoLua, Lua, UserData};
use serde::Deserialize;
use std::{
    any::{Any, TypeId},
    mem::{transmute_copy},
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
pub enum LuaValue {
    Nil,
    Integer(i64),
    Number(f64),
    String(String),
}

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
        _entity: bevy::prelude::Entity,
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
                    let args: Vec<LuaValue> = args
                        .into_iter()
                        .map(|arg| match arg {
                            mlua::Value::Nil => LuaValue::Nil,
                            mlua::Value::Boolean(_) => todo!(),
                            mlua::Value::LightUserData(_) => todo!(),
                            mlua::Value::Integer(n) => LuaValue::Integer(n),
                            mlua::Value::Number(n) => LuaValue::Number(n),
                            mlua::Value::String(s) => {
                                LuaValue::String(s.to_string_lossy().to_string())
                            }
                            mlua::Value::Table(_) => todo!(),
                            mlua::Value::Function(_) => todo!(),
                            mlua::Value::Thread(_) => todo!(),
                            mlua::Value::UserData(_) => todo!(),
                            mlua::Value::Error(_) => todo!(),
                        })
                        .collect();
                    let promise = f((), args).unwrap();

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
        _script_data: &mut Self::ScriptData,
        _entity: bevy::prelude::Entity,
        args: impl FuncArgs<Self::Value>,
    ) -> Result<(), crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        let func = engine.globals().get::<_, Function>(name).unwrap();
        let args: Vec<mlua::Value> = args.parse().into_iter().map(|_a| mlua::Value::Nil).collect();
        let _ = func.call::<_, ()>(args);
        Ok(())
    }

    fn call_fn_from_value(
        &self,
        _value: &Self::Value,
        _context: &Self::CallContext,
        _args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        todo!()
    }
}

impl<T: Any + Clone + Send + Sync> IntoValue<LuaValue> for T {
    fn into_value(self) -> LuaValue {
        LuaValue::Nil
    }
}

impl From<()> for LuaValue {
    fn from(_value: ()) -> Self {
        LuaValue::Nil
    }
}

impl FuncArgs<LuaValue> for () {
    fn parse(self) -> Vec<LuaValue> {
        Vec::new()
    }
}

impl<T: IntoLua<'static>> FuncArgs<LuaValue> for Vec<T> {
    fn parse(self) -> Vec<LuaValue> {
        self.into_iter().map(|_| LuaValue::Nil).collect()
    }
}

impl CloneCast for LuaValue {
    fn clone_cast<T: Any + Clone + 'static>(&self) -> T {
        match self {
            LuaValue::Nil if TypeId::of::<T>() == TypeId::of::<()>() => unsafe {
                transmute_copy(&())
            },
            LuaValue::Integer(n) if TypeId::of::<T>() == TypeId::of::<i64>() => unsafe {
                transmute_copy(n)
            },
            LuaValue::Number(n) if TypeId::of::<T>() == TypeId::of::<i64>() => unsafe {
                transmute_copy(n)
            },
            LuaValue::String(s) if TypeId::of::<T>() == TypeId::of::<String>() => unsafe {
                transmute_copy(s)
            },
            _ => panic!(
                "Failed conversion of {:?} into {:?}",
                self,
                std::any::type_name::<T>()
            ),
        }
    }
}

pub mod prelude {
    pub use super::{LuaRuntime, LuaScript, LuaScriptData};
}
