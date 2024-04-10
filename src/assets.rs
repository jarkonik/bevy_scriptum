use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt as _, LoadContext},
    reflect::{TypePath, TypeUuid},
    utils::BoxedFuture,
};
use serde::Deserialize;

/// A script that can be loaded by the [crate::ScriptingPlugin].
#[derive(Asset, Debug, Deserialize, TypeUuid, TypePath)]
#[uuid = "3ed4b68b-4f5d-4d82-96f6-5194e358921a"]
pub struct RhaiScript(pub String);

/// A loader for [RhaiScript] assets.
#[derive(Default)]
pub struct RhaiScriptLoader;

impl AssetLoader for RhaiScriptLoader {
    type Asset = RhaiScript;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<RhaiScript, anyhow::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let rhai_script = RhaiScript(String::from_utf8(bytes.to_vec())?);
            Ok(rhai_script)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rhai"]
    }
}
