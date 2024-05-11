use std::marker::PhantomData;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt as _, LoadContext},
    utils::BoxedFuture,
};

/// A loader for script assets.
#[derive(Default)]
pub struct ScriptLoader<T> {
    _phantdom_data: PhantomData<T>,
}

pub trait FileExtension {
    fn extension() -> &'static [&'static str];
}

impl<T: FileExtension + From<String> + Asset + Send + Sync + 'static> AssetLoader
    for ScriptLoader<T>
{
    type Asset = T;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<T, anyhow::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let script: T = String::from_utf8(bytes.to_vec())?.into();
            Ok(script)
        })
    }

    fn extensions(&self) -> &[&str] {
        T::extension()
    }
}
