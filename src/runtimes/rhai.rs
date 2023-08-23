use bevy::prelude::Entity;
use rhai::{CallFnOptions, Dynamic};

use crate::{ScriptingError, ScriptingRuntime, ENTITY_VAR_NAME};

pub struct Runtime {
    rhai_engine: rhai::Engine,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            rhai_engine: rhai::Engine::new(),
        }
    }
}

impl ScriptingRuntime for Runtime {
    fn register_fn(
        &mut self,
        name: String,
        arg_types: Vec<std::any::TypeId>,
        callback: Box<dyn Fn() -> () + Send + Sync>,
    ) {
        self.rhai_engine
            .register_raw_fn(name, arg_types, move |context, args| {
                callback();
                // #[allow(deprecated)]
                // let context_data = context.store_data();
                // let promise = Promise {
                //     inner: Arc::new(Mutex::new(PromiseInner {
                //         callbacks: vec![],
                //         context_data,
                //     })),
                // };

                // let mut calls = callback.calls.lock().unwrap();
                // calls.push(FunctionCallEvent {
                //     promise: promise.clone(),
                //     params: args.iter_mut().map(|arg| arg.clone()).collect(),
                // });
                // Ok(promise)
                Ok(())
            });
    }

    fn eval(&mut self, code: &str) -> Result<(), ScriptingError> {
        let mut scope = rhai::Scope::new();

        let ast = self
            .rhai_engine
            .compile_with_scope(&scope, code)
            .map_err(ScriptingError::CompileError)?;

        self.rhai_engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(ScriptingError::RuntimeError)?;

        Ok(())
    }

    fn call_fn(
        &mut self,
        ast: &rhai::AST,
        scope: &mut rhai::Scope,
        function_name: &str,
        args: Vec<rhai::Dynamic>,
    ) -> Result<rhai::Dynamic, ScriptingError> {
        // scope.push(ENTITY_VAR_NAME, entity);
        let options = CallFnOptions::new().eval_ast(false);
        let result = self.rhai_engine.call_fn_with_options::<Dynamic>(
            options,
            scope,
            &ast,
            function_name,
            args,
        )?;
        // scope.remove::<Entity>(ENTITY_VAR_NAME).unwrap();
        return Ok(result);
    }

    fn set_global_variable(&mut self, name: &str, value: Dynamic) {
        todo!()
    }

    fn unset_global_variable(&mut self, name: &str) {
        todo!()
    }
}
