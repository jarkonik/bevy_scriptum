use std::marker::PhantomData;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt as _, LoadContext},
    utils::BoxedFuture,
};

/// A loader for script assets.
pub struct ScriptLoader<A: Asset + From<String>> {
    _phantom_data: PhantomData<A>,
}

impl<A: Asset + From<String>> Default for ScriptLoader<A> {
    fn default() -> Self {
        Self {
            _phantom_data: Default::default(),
        }
    }
}

/// Allows providing an allow-list for extensions of AssetLoader for a Script
/// asset
pub trait GetExtensions {
    fn extensions() -> &'static [&'static str];
}

impl<A: Asset + From<String> + GetExtensions> AssetLoader for ScriptLoader<A> {
    type Asset = A;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<A, anyhow::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let script_text = String::from_utf8(bytes.to_vec())?;
            let rhai_script: A = script_text.into();
            Ok(rhai_script)
        })
    }

    fn extensions(&self) -> &[&str] {
        A::extensions()
    }
}
