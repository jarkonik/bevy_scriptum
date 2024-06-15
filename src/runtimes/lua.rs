use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, schedule::ScheduleLabel, system::Resource},
    math::Vec3,
    reflect::TypePath,
};
use mlua::{
    FromLua, Function, IntoLua, IntoLuaMulti, Lua, RegistryKey, UserData, UserDataFields,
    UserDataMethods, Variadic,
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    promise::Promise,
    FuncArgs, Runtime, ScriptingError, ENTITY_VAR_NAME,
};

type LuaEngine = Arc<Mutex<Lua>>;

#[derive(Clone)]
pub struct LuaValue(Arc<RegistryKey>);

impl LuaValue {
    fn new<'a, T: IntoLua<'a>>(engine: &'a Lua, value: T) -> Self {
        Self(Arc::new(
            engine
                .create_registry_value(value)
                .expect("Error creating a registry key for value"),
        ))
    }
}

#[derive(Resource)]
pub struct LuaRuntime {
    engine: LuaEngine,
}

#[derive(Clone, Copy)]
pub struct BevyEntity(pub Entity);

impl UserData for BevyEntity {}

impl FromLua<'_> for BevyEntity {
    fn from_lua(
        value: mlua::prelude::LuaValue<'_>,
        _lua: &'_ Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => panic!("got {:?} instead of BevyEntity", value),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BevyVec3(pub Vec3);

impl UserData for BevyVec3 {}

impl FromLua<'_> for BevyVec3 {
    fn from_lua(
        value: mlua::prelude::LuaValue<'_>,
        _lua: &'_ Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => panic!("got {:?} instead of BevyVec3", value),
        }
    }
}

impl Default for LuaRuntime {
    fn default() -> Self {
        let engine = LuaEngine::default();

        {
            let engine = engine.lock().expect("Failed to lock engine");
            engine
                .register_userdata_type::<BevyEntity>(|typ| {
                    typ.add_field_method_get("index", |_, entity| Ok(entity.0.index()));
                })
                .expect("Failed to register BevyEntity userdata type");

            engine
                .register_userdata_type::<Promise<(), LuaValue>>(|typ| {
                    typ.add_method_mut("and_then", |engine, promise, callback: Function| {
                        Ok(Promise::then(promise, LuaValue::new(engine, callback)))
                    });
                })
                .expect("Failed to register Promise userdata type");

            engine
                .register_userdata_type::<BevyVec3>(|typ| {
                    typ.add_field_method_get("x", |_engine, vec| Ok(vec.0.x));
                    typ.add_field_method_get("y", |_engine, vec| Ok(vec.0.y));
                    typ.add_field_method_get("z", |_engine, vec| Ok(vec.0.z));
                })
                .expect("Failed to register BevyVec3 userdata type");
            let vec3_constructor = engine
                .create_function(|_, (x, y, z)| Ok(BevyVec3(Vec3::new(x, y, z))))
                .expect("Failed to create Vec3 constructor");
            engine
                .globals()
                .set("Vec3", vec3_constructor)
                .expect("Failed to set Vec3 global");
        }

        Self { engine }
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

impl Runtime for LuaRuntime {
    type Schedule = LuaSchedule;

    type ScriptAsset = LuaScript;

    type ScriptData = LuaScriptData;

    type CallContext = ();

    type Value = LuaValue;

    type RawEngine = Lua;

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        self.with_engine(|engine| {
            engine
                .globals()
                .set(ENTITY_VAR_NAME, BevyEntity(entity))
                .expect("Error setting entity global variable");
            let result = engine.load(&script.0).exec();
            engine
                .globals()
                .set(ENTITY_VAR_NAME, mlua::Value::Nil)
                .expect("Error clearing entity global variable");
            result
        })
        .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;
        Ok(LuaScriptData)
    }

    fn register_fn(
        &mut self,
        name: String,
        _arg_types: Vec<std::any::TypeId>,
        f: impl Fn(
                Self::CallContext,
                Vec<Self::Value>,
            ) -> Result<
                crate::promise::Promise<Self::CallContext, Self::Value>,
                crate::ScriptingError,
            > + Send
            + Sync
            + 'static,
    ) -> Result<(), crate::ScriptingError> {
        self.with_engine(|engine| {
            let func = engine
                .create_function(move |engine, args: Variadic<mlua::Value>| {
                    let args = { args.into_iter().map(|x| LuaValue::new(engine, x)).collect() };
                    let result = f((), args).unwrap();
                    Ok(result)
                })
                .unwrap();
            engine
                .globals()
                .set(name, func)
                .expect("Error registering function in global lua scope");
        });
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        _script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl for<'a> FuncArgs<'a, Self::Value, Self>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        self.with_engine(|engine| {
            engine
                .globals()
                .set(ENTITY_VAR_NAME, BevyEntity(entity))
                .expect("Error setting entity global variable");
            let func = engine
                .globals()
                .get::<_, Function>(name)
                .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;
            let args = args
                .parse(engine)
                .into_iter()
                .map(|a| engine.registry_value::<mlua::Value>(&a.0).unwrap());
            let result = func
                .call::<_, mlua::Value>(Variadic::from_iter(args))
                .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;
            engine
                .globals()
                .set(ENTITY_VAR_NAME, mlua::Value::Nil)
                .expect("Error clearing entity global variable");
            Ok(LuaValue::new(engine, result))
        })
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        _context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        self.with_engine(|engine| {
            let val = engine
                .registry_value::<Function>(&value.0)
                .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;
            let args = args
                .into_iter()
                .map(|a| engine.registry_value::<mlua::Value>(&a.0).unwrap());
            let result = val
                .call::<_, mlua::Value>(Variadic::from_iter(args))
                .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;
            Ok(LuaValue::new(engine, result))
        })
    }

    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        let mut engine = self.engine.lock().unwrap();
        f(&mut engine)
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        let engine = self.engine.lock().unwrap();
        f(&engine)
    }
}

impl<'a, T: IntoLuaMulti<'a>> IntoRuntimeValueWithEngine<'a, T, LuaRuntime> for T {
    fn into_runtime_value_with_engine(value: T, engine: &'a Lua) -> LuaValue {
        let mut iter = value.into_lua_multi(engine).unwrap().into_iter();
        if iter.len() > 1 {
            unimplemented!("Returning multiple values from function");
        }
        LuaValue(Arc::new(engine.create_registry_value(iter.next()).unwrap()))
    }
}

impl<'a, T: FromLua<'a>> FromRuntimeValueWithEngine<'a, LuaRuntime> for T {
    fn from_runtime_value_with_engine(value: LuaValue, engine: &'a Lua) -> Self {
        engine.registry_value(&value.0).unwrap()
    }
}

impl FuncArgs<'_, LuaValue, LuaRuntime> for () {
    fn parse(self, _engine: &Lua) -> Vec<LuaValue> {
        Vec::new()
    }
}

impl<'a, A: IntoLua<'a>, B: IntoLua<'a>> FuncArgs<'a, LuaValue, LuaRuntime> for (A, B) {
    fn parse(self, engine: &'a Lua) -> Vec<LuaValue> {
        vec![LuaValue::new(engine, self.0), LuaValue::new(engine, self.1)]
    }
}

impl<'a, T: IntoLua<'a>> FuncArgs<'a, LuaValue, LuaRuntime> for Vec<T> {
    fn parse(self, engine: &'a Lua) -> Vec<LuaValue> {
        self.into_iter().map(|x| LuaValue::new(engine, x)).collect()
    }
}

impl UserData for Promise<(), LuaValue> {}

pub mod prelude {
    pub use super::{BevyEntity, LuaRuntime, LuaScript, LuaScriptData};
}
