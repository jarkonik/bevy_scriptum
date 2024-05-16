use std::sync::{Arc, Mutex};

use crate::{Runtime, ScriptingError};

/// A struct that represents a function that will get called when the Promise is resolved.
pub(crate) struct PromiseCallback<C: Send, V: Send> {
    callback: V,
    following_promise: Arc<Mutex<PromiseInner<C, V>>>,
}

/// Internal representation of a Promise.
pub(crate) struct PromiseInner<C: Send, V: Send> {
    pub(crate) callbacks: Vec<PromiseCallback<C, V>>,
    #[allow(deprecated)]
    pub(crate) context: C,
}

/// A struct that represents a Promise.
#[derive(Clone)]
pub struct Promise<C: Send, V: Send> {
    pub(crate) inner: Arc<Mutex<PromiseInner<C, V>>>,
}

impl<C: Send, V: Send> PromiseInner<C, V> {
    /// Resolve the Promise. This will call all the callbacks that were added to the Promise.
    fn resolve(&mut self, runtime: &mut impl Runtime, val: V) -> Result<(), ScriptingError> {
        for callback in &self.callbacks {
            runtime.call_fn_from_value(callback.callback, self.context, val);
            // let result = callback.callback.try_call(self.context, val);

            // let f = callback.callback.clone_cast::<FnPtr>();
            //
            //

            // let next_val = if val.is_unit() {
            //     f.call_raw(&self.context, None, [()])?
            // } else {
            //     f.call_raw(&self.context, None, [val.clone()])?
            // };
            // callback
            //     .following_promise
            //     .lock()
            //     .unwrap()
            //     .resolve(runtime, next_val)?;
            // callback
            //     .following_promise
            //     .lock()
            //     .unwrap()
            //     .resolve(runtime, ())?;
        }
        Ok(())
    }
}

impl<C: Clone + Send + 'static, V: Send> Promise<C, V> {
    /// Acquire [Mutex] for writing the promise and resolve it. Call will be forwarded to [PromiseInner::resolve].
    pub(crate) fn resolve(
        &mut self,
        runtime: &mut impl Runtime,
        val: V,
    ) -> Result<(), ScriptingError> {
        if let Ok(mut inner) = self.inner.lock() {
            inner.resolve(runtime, val)?;
        }
        Ok(())
    }

    /// Register a callback that will be called when the [Promise] is resolved.
    pub(crate) fn then(&mut self, callback: V) -> Self {
        let mut inner = self.inner.lock().unwrap();
        let following_inner = Arc::new(Mutex::new(PromiseInner {
            callbacks: vec![],
            context: inner.context.clone(),
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
