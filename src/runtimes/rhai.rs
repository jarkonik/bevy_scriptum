use std::fmt::Debug;

use bevy::{
    asset::Asset,
    ecs::{component::Component, entity::Entity, schedule::ScheduleLabel, resource::Resource},
    math::Vec3,
    reflect::TypePath,
};
use rhai::{CallFnOptions, Dynamic, Engine, FnPtr, Scope, Variant};
use serde::Deserialize;

use crate::{
    assets::GetExtensions,
    callback::{FromRuntimeValueWithEngine, IntoRuntimeValueWithEngine},
    promise::Promise,
    FuncArgs, Runtime, ScriptingError, ENTITY_VAR_NAME,
};

#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct RhaiScript(pub String);

impl GetExtensions for RhaiScript {
    fn extensions() -> &'static [&'static str] {
        &["rhai"]
    }
}

impl From<String> for RhaiScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Resource)]
pub struct RhaiRuntime {
    engine: rhai::Engine,
}

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct RhaiSchedule;

/// A component that represents the data of a script. It stores the [rhai::Scope](basically the state of the script, any declared variable etc.)
/// and [rhai::AST] which is a cached AST representation of the script.
#[derive(Component)]
pub struct RhaiScriptData {
    pub scope: rhai::Scope<'static>,
    pub(crate) ast: rhai::AST,
}

#[derive(Debug, Clone)]
pub struct RhaiValue(rhai::Dynamic);

impl Runtime for RhaiRuntime {
    type Schedule = RhaiSchedule;
    type ScriptAsset = RhaiScript;
    type ScriptData = RhaiScriptData;
    #[allow(deprecated)]
    type CallContext = rhai::NativeCallContextStore;
    type Value = RhaiValue;
    type RawEngine = rhai::Engine;

    fn eval(
        &self,
        script: &Self::ScriptAsset,
        entity: Entity,
    ) -> Result<Self::ScriptData, ScriptingError> {
        let mut scope = Scope::new();
        scope.push(ENTITY_VAR_NAME, entity);

        let engine = &self.engine;

        let ast = engine
            .compile_with_scope(&scope, script.0.as_str())
            .map_err(|e| ScriptingError::CompileError(Box::new(e)))?;

        engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| ScriptingError::RuntimeError(Box::new(e)))?;

        scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();

        Ok(Self::ScriptData { ast, scope })
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
    ) -> Result<(), ScriptingError> {
        self.engine
            .register_raw_fn(name, arg_types, move |context, args| {
                let args = args.iter_mut().map(|arg| RhaiValue(arg.clone())).collect();
                #[allow(deprecated)]
                let promise = f(context.store_data(), args).unwrap();
                Ok(promise)
            });
        Ok(())
    }

    fn call_fn(
        &self,
        name: &str,
        script_data: &mut Self::ScriptData,
        entity: Entity,
        args: impl for<'a> FuncArgs<'a, Self::Value, Self>,
    ) -> Result<RhaiValue, ScriptingError> {
        let ast = script_data.ast.clone();
        let scope = &mut script_data.scope;
        scope.push(ENTITY_VAR_NAME, entity);
        let options = CallFnOptions::new().eval_ast(false);
        let args = args
            .parse(&self.engine)
            .into_iter()
            .map(|a| a.0)
            .collect::<Vec<Dynamic>>();
        let result = self
            .engine
            .call_fn_with_options::<Dynamic>(options, scope, &ast, name, args);
        scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
        match result {
            Ok(val) => Ok(RhaiValue(val)),
            Err(e) => Err(ScriptingError::RuntimeError(Box::new(e))),
        }
    }

    fn call_fn_from_value(
        &self,
        value: &Self::Value,
        context: &Self::CallContext,
        args: Vec<Self::Value>,
    ) -> Result<Self::Value, ScriptingError> {
        let f = value.0.clone_cast::<FnPtr>();

        #[allow(deprecated)]
        let ctx = &context.create_context(&self.engine);

        let result = if args.len() == 1 && args.first().unwrap().0.is_unit() {
            f.call_raw(ctx, None, [])
                .map_err(|e| ScriptingError::RuntimeError(e))?
        } else {
            let args = args.into_iter().map(|a| a.0).collect::<Vec<Dynamic>>();
            f.call_raw(ctx, None, args)
                .map_err(|e| ScriptingError::RuntimeError(e))?
        };

        Ok(RhaiValue(result))
    }

    fn with_engine_mut<T>(&mut self, f: impl FnOnce(&mut Self::RawEngine) -> T) -> T {
        f(&mut self.engine)
    }

    fn with_engine<T>(&self, f: impl FnOnce(&Self::RawEngine) -> T) -> T {
        f(&self.engine)
    }
}

impl Default for RhaiRuntime {
    fn default() -> Self {
        let mut engine = Engine::new();

        engine
            .register_type_with_name::<Entity>("Entity")
            .register_get("index", |entity: &mut Entity| entity.index());
        #[allow(deprecated)]
        engine
            .register_type_with_name::<Promise<rhai::NativeCallContextStore, RhaiValue>>("Promise")
            .register_fn(
                "then",
                |promise: &mut Promise<rhai::NativeCallContextStore, RhaiValue>,
                 callback: rhai::Dynamic| {
                    Promise::then(promise, RhaiValue(callback));
                },
            );

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

        RhaiRuntime { engine }
    }
}

impl<'a, T: Clone + Variant> IntoRuntimeValueWithEngine<'a, T, RhaiRuntime> for T {
    fn into_runtime_value_with_engine(value: T, _engine: &'a rhai::Engine) -> RhaiValue {
        RhaiValue(Dynamic::from(value))
    }
}

impl FuncArgs<'_, RhaiValue, RhaiRuntime> for () {
    fn parse(self, _engnie: &rhai::Engine) -> Vec<RhaiValue> {
        Vec::new()
    }
}
impl<T: Clone + Send + Sync + 'static> FuncArgs<'_, RhaiValue, RhaiRuntime> for Vec<T> {
    fn parse(self, _engine: &rhai::Engine) -> Vec<RhaiValue> {
        self.into_iter()
            .map(|v| RhaiValue(Dynamic::from(v)))
            .collect()
    }
}

impl<T: Clone + 'static> FromRuntimeValueWithEngine<'_, RhaiRuntime> for T {
    fn from_runtime_value_with_engine(value: RhaiValue, _engine: &rhai::Engine) -> Self {
        value.0.clone_cast()
    }
}

pub mod prelude {
    pub use super::{RhaiRuntime, RhaiScript, RhaiScriptData};
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<$($t: Clone +Variant,)+> FuncArgs<'_, RhaiValue, RhaiRuntime>
            for ($($t,)+)
        {
            fn parse(self, _engine: &rhai::Engine) -> Vec<RhaiValue> {
                vec![
                    $(RhaiValue(Dynamic::from(self.$idx)), )+
                ]
            }
        }
    };
}

impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W, 23 X, 24 Y, 25 Z);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W, 23 X, 24 Y);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W, 23 X);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V, 22 W);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U, 21 V);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T, 20 U);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S, 19 T);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R, 18 S);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q, 17 R);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P, 16 Q);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O, 15 P);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N, 14 O);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M, 13 N);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L, 12 M);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K, 11 L);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J, 10 K);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I, 9 J);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H, 8 I);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G, 7 H);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F, 6 G);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);
impl_tuple!(0 A, 1 B, 2 C, 3 D);
impl_tuple!(0 A, 1 B, 2 C);
impl_tuple!(0 A, 1 B);
impl_tuple!(0 A);
