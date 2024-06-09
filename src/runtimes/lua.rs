use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use mlua::{FromLua, Function, IntoLua, Lua, RegistryKey, UserData, UserDataMethods, Variadic};
use serde::Deserialize;
use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    promise::Promise,
    FuncArgs, Runtime, ENTITY_VAR_NAME,
};

type LuaEngine = Arc<Mutex<Lua>>;

#[derive(Clone)]
pub struct LuaValue(Arc<RegistryKey>);

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
        lua: &'_ Lua,
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
            let engine = engine.lock().unwrap();
            engine
                .register_userdata_type::<BevyEntity>(|typ| {
                    typ.add_method("index", |_, entity, ()| Ok(entity.0.index()));
                })
                .unwrap();

            engine
                .register_userdata_type::<Promise<(), LuaValue>>(|typ| {
                    typ.add_method_mut("and_then", |engine, promise, callback: Function| {
                        let val = engine.create_registry_value(callback).unwrap();
                        Ok(Promise::then(promise, LuaValue(Arc::new(val))))
                    });
                })
                .unwrap();
        }

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
        //
        // RhaiRuntime { engine }

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
    fn create_script_data(
        &self,
        script: &Self::ScriptAsset,
        entity: bevy::prelude::Entity,
    ) -> Result<Self::ScriptData, crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
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
            ) -> Result<
                crate::promise::Promise<Self::CallContext, Self::Value>,
                crate::ScriptingError,
            > + Send
            + Sync
            + 'static,
    ) -> Result<(), crate::ScriptingError> {
        let engine_closure = self.engine.clone();
        let engine = self.engine.lock().unwrap();
        let func = engine
            .create_function(move |engine, args: Variadic<mlua::Value>| {
                let args = {
                    args.into_iter()
                        .map(|x| LuaValue(Arc::new(engine.create_registry_value(x).unwrap())))
                        .collect()
                };
                Ok(f((), args).unwrap())
            })
            .unwrap();
        engine.globals().set(name, func).unwrap();
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: bevy::prelude::Entity,
        args: impl FuncArgs<Self::Value, Self>,
    ) -> Result<(), crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        let func = engine.globals().get::<_, Function>(name).unwrap();
        let args: Vec<mlua::Value> = args
            .parse(&engine)
            .into_iter()
            .map(|a| engine.registry_value(&a.0).unwrap())
            .collect();
        func.call::<_, ()>(args).unwrap();
        Ok(())
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, crate::ScriptingError> {
        let engine = self.engine.lock().unwrap();
        let val = engine.registry_value::<Function>(&value.0).unwrap();
        let args: Vec<mlua::Value> = args
            .into_iter()
            .map(|a| engine.registry_value(&a.0).unwrap())
            .collect();
        let result = val.call::<_, mlua::Value>(args).unwrap();
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
impl<'a, T: IntoLua<'a>> IntoRuntimeValueWithEngine<'a, T, LuaRuntime> for T {
    fn into_runtime_value_with_engine(value: T, engine: &'a Lua) -> LuaValue {
        let e = value.into_lua(engine).unwrap();
        let key = engine.create_registry_value(e).unwrap();
        LuaValue(Arc::new(key))
    }
}

impl<'a, T: FromLua<'a>> FromRuntimeValueWithEngine<'a, LuaRuntime> for T {
    fn from_runtime_value_with_engine(value: LuaValue, engine: &'a Lua) -> Self {
        engine.registry_value(&value.0).unwrap()
    }
}

impl<'a> FuncArgs<LuaValue, LuaRuntime> for () {
    fn parse(self, engine: &Lua) -> Vec<LuaValue> {
        Vec::new()
    }
}

impl<'a, T: IntoLua<'static>> FuncArgs<LuaValue, LuaRuntime> for Vec<T> {
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
