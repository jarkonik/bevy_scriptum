use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::{TypePath, TypeUuid},
    utils::BoxedFuture,
};
use serde::Deserialize;

/// A script that can be loaded by the [crate::ScriptingPlugin].
#[derive(Debug, Deserialize, TypeUuid, TypePath)]
#[uuid = "3ed4b68b-4f5d-4d82-96f6-5194e358921a"]
pub struct ScriptAsset(pub String);

/// A loader for [RhaiScript] assets.
#[derive(Default)]
pub struct ScriptLoader;

impl AssetLoader for ScriptLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let rhai_script = ScriptAsset(String::from_utf8(bytes.to_vec())?);
            load_context.set_default_asset(LoadedAsset::new(rhai_script));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rhai"]
    }
}
