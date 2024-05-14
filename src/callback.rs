use bevy::prelude::*;
use core::any::TypeId;
use std::sync::{Arc, Mutex};

use rhai::{Dynamic, Variant};

use crate::promise::Promise;

/// A system that can be used to call a script function.
pub struct CallbackSystem<V> {
    pub(crate) system: Box<dyn System<In = Vec<V>, Out = V>>,
    pub(crate) arg_types: Vec<TypeId>,
}

pub(crate) struct FunctionCallEvent<C: Send, V> {
    pub(crate) params: Vec<V>,
    pub(crate) promise: Promise<C>,
}

/// A struct representing a Bevy system that can be called from a script.
#[derive(Clone)]
pub(crate) struct Callback<C: Send, V> {
    pub(crate) name: String,
    pub(crate) system: Arc<Mutex<CallbackSystem<V>>>,
    pub(crate) calls: Arc<Mutex<Vec<FunctionCallEvent<C, V>>>>,
}

impl<V: Clone> CallbackSystem<V> {
    pub(crate) fn call<C: Send>(&mut self, call: &FunctionCallEvent<C, V>, world: &mut World) -> V {
        self.system.run(call.params.clone(), world)
    }
}

/// Trait that alllows to convert a script callback function into a Bevy [`System`].
pub trait RegisterCallbackFunction<
    Out,
    Marker,
    A: 'static,
    const N: usize,
    const X: bool,
    R: 'static,
    const F: bool,
    Args,
>: IntoSystem<Args, Out, Marker>
{
    /// Convert this function into a [CallbackSystem].
    #[must_use]
    fn into_callback_system(self, world: &mut World) -> CallbackSystem<R>;
}

impl<Out, FN, Marker> RegisterCallbackFunction<Out, Marker, (), 1, false, Dynamic, false, ()> for FN
where
    FN: IntoSystem<(), Out, Marker>,
    Out: Sync + Variant + Clone,
{
    fn into_callback_system(self, world: &mut World) -> CallbackSystem<Dynamic> {
        let mut inner_system = IntoSystem::into_system(self);
        inner_system.initialize(world);
        let system_fn = move |_args: In<Vec<Dynamic>>, world: &mut World| {
            let result = inner_system.run((), world);
            inner_system.apply_deferred(world);
            Dynamic::from(result)
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
        impl<$($t,)+ Out, FN, Marker> RegisterCallbackFunction<Out, Marker, ($($t,)+), 1, false, Dynamic, false, ($($t,)+)>
            for FN
        where
            FN: IntoSystem<($($t,)+), Out, Marker>,
            Out: Sync + Variant + Clone,
            $($t: 'static + Clone,)+
        {
            fn into_callback_system(self, world: &mut World) -> CallbackSystem<Dynamic> {
                let mut inner_system = IntoSystem::into_system(self);
                inner_system.initialize(world);
                let system_fn = move |args: In<Vec<Dynamic>>, world: &mut World| {
                    let args = (
                        $(args.0.get($idx).unwrap().clone_cast::<$t>(), )+
                    );
                    let result = inner_system.run(args, world);
                    inner_system.apply_deferred(world);
                    Dynamic::from(result)
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
