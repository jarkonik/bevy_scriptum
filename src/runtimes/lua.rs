use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use core::panic;
use mlua::{Function, IntoLua, Lua, OwnedFunction, UserData, UserDataFields, UserDataMethods};
use serde::Deserialize;
use std::{
    any::{Any, TypeId},
    fmt::Debug,
    mem::transmute_copy,
    sync::{Arc, Mutex},
};

use crate::{
    assets::GetExtensions,
    callback::{CloneCast, IntoValue},
    promise::Promise,
    EngineMut, EngineRef, FuncArgs, Runtime, ScriptingError, ENTITY_VAR_NAME,
};

pub struct LuaEngine(Arc<Mutex<Lua>>);
unsafe impl Send for LuaEngine {}
unsafe impl Sync for LuaEngine {}

#[derive(Clone, Debug)]
pub enum LuaValue {
    Nil,
    Integer(i64),
    Number(f64),
    String(String),
    Boolean(bool),
    Function(LuaFunction),
    Entity(BevyEntity),
}

#[derive(Clone, Debug)]
struct LuaFunction(Arc<Mutex<OwnedFunction>>);
unsafe impl Send for LuaFunction {}
unsafe impl Sync for LuaFunction {}

impl LuaFunction {
    fn new(f: OwnedFunction) -> Self {
        Self(Arc::new(Mutex::new(f)))
    }
}

#[derive(Resource)]
pub struct LuaRuntime {
    engine: LuaEngine,
}

impl Default for LuaRuntime {
    fn default() -> Self {
        let engine = Lua::new();

        engine
            .register_userdata_type::<Entity>(|t| {
                t.add_method("index", |_, entity, ()| {
                    let index = entity.index();

                    Ok(index)
                });
            })
            .unwrap();

        engine
            .register_userdata_type::<Promise<(), LuaValue>>(|t| {
                t.add_method_mut("and_then", |_, promise, (callback): mlua::Function| {
                    let promise = Promise::then(
                        promise,
                        LuaValue::Function(LuaFunction::new(callback.into_owned())),
                    );
                    Ok(promise)
                });
            })
            .unwrap();

        // #[allow(deprecated)]
        // engine
        //     .register_fn(
        //         "then",
        //         |promise: &mut Promise<rhai::NativeCallContextStore, RhaiValue>,
        //          callback: rhai::Dynamic| {
        //             Promise::then(promise, RhaiValue(callback));
        //         },
        //     );
        //
        // engine
        //     .register_type_with_name::<Vec3>("Vec3")
        //     .register_fn("new_vec3", |x: f64, y: f64, z: f64| {
        //         Vec3::new(x as f32, y as f32, z as f32)
        //     })
        //     .register_get("x", |vec: &mut Vec3| vec.x as f64)
        //     .register_get("y", |vec: &mut Vec3| vec.y as f64)
        //     .register_get("z", |vec: &mut Vec3| vec.z as f64);
        // #[allow(deprecated)]
        // engine.on_def_var(|_, info, _| Ok(info.name != "entity"));

        LuaRuntime {
            engine: LuaEngine(Arc::new(Mutex::new(engine))),
        }
    }
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

#[derive(Clone, Debug)]
pub struct BevyEntity(Entity);

impl UserData for BevyEntity {}

// TODO: Remove all unwraps, panics and todos
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
        let engine = self.engine.0.lock().unwrap();
        engine
            .globals()
            .set(ENTITY_VAR_NAME, BevyEntity(entity))
            .unwrap();
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
        let engine = self.engine.0.lock().unwrap();

        let func = if !arg_types.is_empty() {
            engine
                .create_function(move |_, args: mlua::Variadic<mlua::Value>| {
                    let args: Vec<LuaValue> = args
                        .into_iter()
                        .map(|arg| match arg {
                            mlua::Value::Nil => LuaValue::Nil,
                            mlua::Value::Boolean(b) => LuaValue::Boolean(b),
                            mlua::Value::LightUserData(_) => todo!(),
                            mlua::Value::Integer(n) => LuaValue::Integer(n),
                            mlua::Value::Number(n) => LuaValue::Number(n),
                            mlua::Value::String(s) => {
                                LuaValue::String(s.to_string_lossy().to_string())
                            }
                            mlua::Value::Table(_) => todo!(),
                            mlua::Value::Function(_) => todo!(),
                            mlua::Value::Thread(_) => todo!(),
                            mlua::Value::UserData(d) => {
                                let e = d.borrow::<BevyEntity>().unwrap().clone();
                                LuaValue::Entity(e)
                            }
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

    fn call_fn(
        &self,
        name: &str,
        _script_data: &mut Self::ScriptData,
        _entity: bevy::prelude::Entity,
        args: impl FuncArgs<Self::Value>,
    ) -> Result<(), crate::ScriptingError> {
        // TODO: Should this return () ?
        let engine = self.engine.0.lock().unwrap();
        let func = engine.globals().get::<_, Function>(name).unwrap();
        let args: Vec<mlua::Value> = args.parse().into_iter().map(|a| a.into()).collect();
        let r = func.call::<_, ()>(args).unwrap();
        Ok(())
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        if let LuaValue::Function(f) = value {
            let f = f.0.lock().unwrap();
            let args: Vec<mlua::Value> = args.into_iter().map(|a| a.into()).collect();
            let r = f.call::<_, mlua::Value>(args).unwrap();
            let r = match r {
                mlua::Value::Nil => LuaValue::Nil,
                mlua::Value::Boolean(_) => todo!(),
                mlua::Value::LightUserData(_) => todo!(),
                mlua::Value::Integer(_) => todo!(),
                mlua::Value::Number(_) => todo!(),
                mlua::Value::String(_) => todo!(),
                mlua::Value::Table(_) => todo!(),
                mlua::Value::Function(_) => todo!(),
                mlua::Value::Thread(_) => todo!(),
                mlua::Value::UserData(_) => todo!(),
                mlua::Value::Error(_) => todo!(),
            };
            Ok(r)
        } else {
            panic!("{:?} cannot be called", value)
        }
    }
}

impl IntoValue<LuaValue> for String {
    fn into_value(self) -> LuaValue {
        LuaValue::String(self)
    }
}

// impl<T: Any + Clone + Send + Sync> IntoValue<LuaValue> for T {
//     fn into_value(self) -> LuaValue {
//         LuaValue::Nil
//     }
// }
//
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

impl From<LuaValue> for mlua::Value<'_> {
    fn from(value: LuaValue) -> Self {
        match value {
            LuaValue::Nil => mlua::Value::Nil,
            LuaValue::Integer(_) => todo!(),
            LuaValue::Number(_) => todo!(),
            LuaValue::String(s) => panic!("{:?}", s),
            LuaValue::Boolean(_) => todo!(),
            LuaValue::Function(_) => todo!(),
            LuaValue::Entity(_) => todo!(),
        }
    }
}

impl CloneCast for LuaValue {
    // TODO: This probably should not panic, or maybe add TryCloneCast trait?
    fn clone_cast<T: Any + Clone + 'static>(&self) -> T {
        match self {
            LuaValue::Nil if TypeId::of::<T>() == TypeId::of::<()>() => unsafe {
                transmute_copy(&())
            },
            LuaValue::Integer(n) if TypeId::of::<T>() == Any::type_id(n) => unsafe {
                transmute_copy(n)
            },
            LuaValue::Number(n) if TypeId::of::<T>() == Any::type_id(n) => unsafe {
                transmute_copy(n)
            },
            LuaValue::String(s) if TypeId::of::<T>() == Any::type_id(s) => unsafe {
                transmute_copy(&std::mem::ManuallyDrop::new(s.to_string()))
            },
            LuaValue::Boolean(b) if TypeId::of::<T>() == Any::type_id(b) => unsafe {
                transmute_copy(b)
            },
            LuaValue::Entity(e) if TypeId::of::<T>() == TypeId::of::<Entity>() => unsafe {
                transmute_copy(&e.0)
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
