use std::sync::{Arc, Mutex};

#[allow(deprecated)]
use rhai::{Dynamic, NativeCallContextStore};
use rhai::{EvalAltResult, FnPtr};

use crate::Runtime;

/// A struct that represents a function that will get called when the Promise is resolved.
pub(crate) struct PromiseCallback<C: Send> {
    callback: Dynamic,
    following_promise: Arc<Mutex<PromiseInner<C>>>,
}

/// Internal representation of a Promise.
pub(crate) struct PromiseInner<C: Send> {
    pub(crate) callbacks: Vec<PromiseCallback<C>>,
    #[allow(deprecated)]
    pub(crate) context: C,
}

/// A struct that represents a Promise.
#[derive(Clone)]
pub struct Promise<C: Send> {
    pub(crate) inner: Arc<Mutex<PromiseInner<C>>>,
}

impl<C: Send> PromiseInner<C> {
    /// Resolve the Promise. This will call all the callbacks that were added to the Promise.
    fn resolve<V>(&mut self, runtime: &mut impl Runtime, val: V) -> Result<(), Box<EvalAltResult>> {
        // for callback in &self.callbacks {
        //     let f = callback.callback.clone_cast::<FnPtr>();
        //     #[allow(deprecated)]
        //     let context = self.context_data.create_context(engine);
        //     let next_val = if val.is_unit() {
        //         f.call_raw(&context, None, [])?
        //     } else {
        //         f.call_raw(&context, None, [val.clone()])?
        //     };
        //     callback
        //         .following_promise
        //         .lock()
        //         .unwrap()
        //         .resolve(engine, next_val)?;
        // }
        Ok(())
    }
}

impl<C: Clone + Send + 'static> Promise<C> {
    /// Acquire [Mutex] for writing the promise and resolve it. Call will be forwarded to [PromiseInner::resolve].
    pub(crate) fn resolve<V>(
        &mut self,
        runtime: &mut impl Runtime,
        val: V,
    ) -> Result<(), Box<EvalAltResult>> {
        if let Ok(mut inner) = self.inner.lock() {
            inner.resolve(runtime, val)?;
        }
        Ok(())
    }

    /// Register a callback that will be called when the [Promise] is resolved.
    pub(crate) fn then(&mut self, callback: rhai::Dynamic) -> rhai::Dynamic {
        let mut inner = self.inner.lock().unwrap();
        let following_inner = Arc::new(Mutex::new(PromiseInner {
            callbacks: vec![],
            context: inner.context.clone(),
        }));

        inner.callbacks.push(PromiseCallback {
            following_promise: following_inner.clone(),
            callback,
        });
        Dynamic::from(Promise {
            inner: following_inner,
        })
    }
}
