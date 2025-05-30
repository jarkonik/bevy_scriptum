use std::sync::{Arc, Mutex};

use magnus::Fiber;

use crate::{Runtime, ScriptingError};

/// A struct that represents a function that will get called when the Promise is resolved.
pub(crate) struct PromiseCallback<C: Send, V: Send, F: Send> {
    callback: V,
    following_promise: Arc<Mutex<PromiseInner<C, V, F>>>,
}

/// Internal representation of a Promise.
pub(crate) struct PromiseInner<C: Send, V: Send, F: Send> {
    pub(crate) callbacks: Vec<PromiseCallback<C, V, F>>,
    #[allow(deprecated)]
    pub(crate) context: C,
    pub(crate) resolved_value: Option<V>,
    pub(crate) fibers: Vec<F>, // TODO: should htis be vec or option
}

/// A struct that represents a Promise.
#[derive(Clone)]
pub struct Promise<C: Send, V: Send, F: Send> {
    pub(crate) inner: Arc<Mutex<PromiseInner<C, V, F>>>,
}

impl<C: Send, V: Send + Clone, F: Send + Clone> PromiseInner<C, V, F> {
    /// Resolve the Promise. This will call all the callbacks that were added to the Promise.
    fn resolve<R>(&mut self, runtime: &mut R, val: R::Value) -> Result<(), ScriptingError>
    where
        R: Runtime<Value = V, CallContext = C>,
    {
        for callback in &self.callbacks {
            let next_val =
                runtime.call_fn_from_value(&callback.callback, &self.context, vec![val.clone()])?;

            callback
                .following_promise
                .lock()
                .expect("Failed to lock promise mutex")
                .resolve(runtime, next_val)?;
        }
        Ok(())
    }
}

impl<C: Clone + Send + 'static, V: Send + Clone, F: Send + Clone> Promise<C, V, F> {
    /// Acquire [Mutex] for writing the promise and resolve it. Call will be forwarded to [PromiseInner::resolve].
    pub(crate) fn resolve<R>(
        &mut self,
        runtime: &mut R,
        val: R::Value,
    ) -> Result<(), ScriptingError>
    where
        R: Runtime<Value = V, CallContext = C>,
    {
        if let Ok(mut inner) = self.inner.lock() {
            inner.resolved_value = Some(val.clone());
            inner.resolve(runtime, val)?;
            for fiber in inner.fibers.drain(..) {
                println!("resume");
            }
        }
        Ok(())
    }

    /// Register a fiber that will be resumed when the [Promise] is resolved.
    #[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
    pub(crate) fn await_promise(&mut self, fiber: F) {
        let mut inner = self
            .inner
            .lock()
            .expect("Failed to lock inner promise mutex");
        inner.fibers.push(fiber);
    }

    /// Register a callback that will be called when the [Promise] is resolved.
    #[cfg(any(feature = "rhai", feature = "lua", feature = "ruby"))]
    pub(crate) fn then(&mut self, callback: V) -> Self {
        let mut inner = self
            .inner
            .lock()
            .expect("Failed to lock inner promise mutex");
        let following_inner = Arc::new(Mutex::new(PromiseInner {
            fibers: vec![],
            callbacks: vec![],
            context: inner.context.clone(),
            resolved_value: None,
        }));

        inner.callbacks.push(PromiseCallback {
            following_promise: following_inner.clone(),
            callback,
        });

        Promise {
            inner: following_inner,
        }
    }
}
