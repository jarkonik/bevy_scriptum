use std::sync::{Arc, Mutex};

#[allow(deprecated)]
use rhai::Dynamic;
use rhai::{EvalAltResult};

/// A struct that represents a function that will get called when the Promise is resolved.
pub(crate) struct PromiseCallback<D, C> {
    callback: Dynamic,
    following_promise: Arc<Mutex<PromiseInner<D, C>>>,
}

/// Internal representation of a Promise.
pub(crate) struct PromiseInner<D, C> {
    pub(crate) callbacks: Vec<PromiseCallback<D, C>>,
    #[allow(deprecated)]
    pub(crate) context_data: D,
}

/// A struct that represents a Promise.
#[derive(Clone)]
pub struct Promise<D, C> {
    pub(crate) inner: Arc<Mutex<PromiseInner<D, C>>>,
}

impl<D, C> PromiseInner<D, C> {
    /// Resolve the Promise. This will call all the callbacks that were added to the Promise.
    fn resolve(
        &mut self,
        _engine: &mut rhai::Engine,
        _val: Dynamic,
    ) -> Result<(), Box<EvalAltResult>> {
        todo!();
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
        // Ok(())
    }
}

impl<D: Clone + Send + 'static, C: Clone + 'static> Promise<D, C> {
    /// Acquire [Mutex] for writing the promise and resolve it. Call will be forwarded to [PromiseInner::resolve].
    pub(crate) fn resolve(
        &mut self,
        engine: &mut rhai::Engine,
        val: Dynamic,
    ) -> Result<(), Box<EvalAltResult>> {
        if let Ok(mut inner) = self.inner.lock() {
            inner.resolve(engine, val)?;
        }
        Ok(())
    }

    /// Register a callback that will be called when the [Promise] is resolved.
    pub(crate) fn then(&mut self, callback: rhai::Dynamic) -> rhai::Dynamic {
        let mut inner = self.inner.lock().unwrap();
        let following_inner = Arc::new(Mutex::new(PromiseInner {
            callbacks: vec![],
            context_data: inner.context_data.clone(),
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
