use std::{marker::PhantomData, sync::{Arc, Mutex}};
use bevy::{app::App, prelude::World};

use crate::{callback::{Callback, IntoCallbackSystem}, Callbacks, Runtime, ScriptingRuntimeBuilder};

pub struct ApiBuilder<'a, R: Runtime> {
    _phantom_data: PhantomData<R>,
    world: &'a mut World,
}

impl<'a, R: Runtime> ApiBuilder<'a, R> {
    fn new(world: &'a mut World) -> Self {
        Self {
            _phantom_data: PhantomData,
            world,
        }
    }

    /// Registers a function for calling from within a script.
    /// Provided function needs to be a valid bevy system and its
    /// arguments and return value need to be convertible to runtime
    /// value types.
    pub fn add_function<In, Out, Marker>(
        self,
        name: String,
        fun: impl IntoCallbackSystem<R, In, Out, Marker>,
    ) -> Self {
        let system = fun.into_callback_system(self.world);

        let mut callbacks_resource = self.world.resource_mut::<Callbacks<R>>();

        callbacks_resource.uninitialized_callbacks.push(Callback {
            name,
            system: Arc::new(Mutex::new(system)),
            calls: Arc::new(Mutex::new(vec![])),
        });

        self
    }
}

pub trait ScriptingApiBuilder {
    fn add_scripting_api<R: Runtime>(&mut self, f: impl Fn(ScriptingRuntimeBuilder<R>)) -> &mut Self;
}

impl ScriptingApiBuilder for App {
    fn add_scripting_api<R: Runtime>(&mut self, f: impl Fn(ScriptingRuntimeBuilder<R>)) -> &mut Self {
        let runtime = ScriptingRuntimeBuilder::<R>::new(&mut self.world);
        
        f(runtime);

        self
    }
}

