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

// TODO: add example and implementation for builting types vec3 and others?

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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
                    typ.add_method("index", |_, entity, ()| Ok(entity.0.index()));
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

    // TODO: Should be renamed or even split as it also evals
    // should also be private to crate
    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        self.with_engine(|engine| {
            // TODO: We somehow need to set it per script not here in globals
            engine
                .globals()
                .set(ENTITY_VAR_NAME, BevyEntity(entity))
                .expect("Error setting entity global variable");
            engine.load(&script.0).exec()
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
        _entity: bevy::prelude::Entity,
        args: impl FuncArgs<Self::Value, Self>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        self.with_engine(|engine| {
            let func = engine.globals().get::<_, Function>(name).unwrap();
            let args = args
                .parse(engine)
                .into_iter()
                .map(|a| engine.registry_value::<mlua::Value>(&a.0).unwrap());
            let result = func
                .call::<_, mlua::Value>(Variadic::from_iter(args))
                .unwrap();
            Ok(LuaValue(Arc::new(
                engine.create_registry_value(result).unwrap(),
            )))
        })
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        _context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        let val = engine.registry_value::<Function>(&value.0).unwrap();
        let args = args
            .into_iter()
            .map(|a| engine.registry_value::<mlua::Value>(&a.0).unwrap());
        let result = val
            .call::<_, mlua::Value>(Variadic::from_iter(args))
            .unwrap();
        Ok(LuaValue(Arc::new(
            engine.create_registry_value(result).unwrap(),
        )))
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

impl FuncArgs<LuaValue, LuaRuntime> for () {
    fn parse(self, _engine: &Lua) -> Vec<LuaValue> {
        Vec::new()
    }
}

impl<T: IntoLua<'static>> FuncArgs<LuaValue, LuaRuntime> for Vec<T> {
    fn parse(self, engine: &Lua) -> Vec<LuaValue> {
        self.into_iter()
            .map(|_| {
                LuaValue(Arc::new(
                    engine.create_registry_value(mlua::Value::Nil).unwrap(),
                ))
            })
            .collect()
    }
}

impl UserData for Promise<(), LuaValue> {}

pub mod prelude {
    pub use super::{BevyEntity, LuaRuntime, LuaScript, LuaScriptData};
}
