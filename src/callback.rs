use bevy::prelude::*;
use core::any::TypeId;
use std::sync::{Arc, Mutex};

use crate::{promise::Promise, Runtime};

/// A system that can be used to call a script function.
pub struct CallbackSystem<R: Runtime> {
    pub(crate) system: Box<dyn System<In = Vec<R::Value>, Out = R::Value>>,
    pub(crate) arg_types: Vec<TypeId>,
}

pub(crate) struct FunctionCallEvent<C: Send, V: Send> {
    pub(crate) params: Vec<V>,
    pub(crate) promise: Promise<C, V>,
}

/// A struct representing a Bevy system that can be called from a script.
pub(crate) struct Callback<R: Runtime> {
    pub(crate) name: String,
    pub(crate) system: Arc<Mutex<CallbackSystem<R>>>,
    pub(crate) calls: Arc<Mutex<Vec<FunctionCallEvent<R::CallContext, R::Value>>>>,
}

impl<R: Runtime> Clone for Callback<R> {
    fn clone(&self) -> Self {
        Callback {
            name: self.name.clone(),
            system: self.system.clone(),
            calls: self.calls.clone(),
        }
    }
}

impl<R: Runtime> CallbackSystem<R> {
    pub(crate) fn call(
        &mut self,
        call: &FunctionCallEvent<R::CallContext, R::Value>,
        world: &mut World,
    ) -> R::Value {
        self.system.run(call.params.clone(), world)
    }
}

pub trait FromWithEngine<V, R: Runtime> {
    fn from_with_runtime(value: V, engine: &mut R::RawEngine) -> R::Value;
}

/// Trait that alllows to convert a script callback function into a Bevy [`System`].
pub trait IntoCallbackSystem<R: Runtime, In, Out, Marker>: IntoSystem<In, Out, Marker> {
    /// Convert this function into a [CallbackSystem].
    #[must_use]
    fn into_callback_system(self, world: &mut World) -> CallbackSystem<R>;
}

// TODO: Move
pub trait CloneCast {
    fn clone_cast<T: Clone + 'static>(&self) -> T;
}

impl<R: Runtime, Out, FN, Marker> IntoCallbackSystem<R, (), Out, Marker> for FN
where
    FN: IntoSystem<(), Out, Marker>,
    Out: FromWithEngine<Out, R>,
{
    fn into_callback_system(self, world: &mut World) -> CallbackSystem<R> {
        let mut inner_system = IntoSystem::into_system(self);
        inner_system.initialize(world);
        let system_fn = move |_args: In<Vec<R::Value>>, world: &mut World| {
            let result = inner_system.run((), world);
            inner_system.apply_deferred(world);
            let mut runtime = world.get_resource_mut::<R>().expect("No runtime resource");
            runtime.with_engine_mut(move |engine| Out::from_with_runtime(result, engine))
        };
        let system = IntoSystem::into_system(system_fn);
        CallbackSystem {
            arg_types: vec![],
            system: Box::new(system),
        }
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<RN: Runtime, $($t,)+ Out, FN, Marker> IntoCallbackSystem<RN, ($($t,)+), Out, Marker>
            for FN
        where
            FN: IntoSystem<($($t,)+), Out, Marker>,
            Out: FromWithEngine<Out, RN>,
            $($t: 'static + Clone,)+
        {
            fn into_callback_system(self, world: &mut World) -> CallbackSystem<RN> {
                let mut inner_system = IntoSystem::into_system(self);
                inner_system.initialize(world);
                let system_fn = move |args: In<Vec<RN::Value>>, world: &mut World| {
                    let args = (
                        $(args.0.get($idx).expect("Failed to get function argument").clone_cast::<$t>(), )+
                    );
                    let result = inner_system.run(args, world);
                    inner_system.apply_deferred(world);
                    let mut runtime = world.get_resource_mut::<RN>().expect("No runtime resouce");
                    runtime.with_engine_mut(move |engine| {
                        Out::from_with_runtime(result, engine)
                    })
                };
                let system = IntoSystem::into_system(system_fn);
                CallbackSystem {
                    arg_types: vec![$(TypeId::of::<$t>(),)+],
                    system: Box::new(system),
                }
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
