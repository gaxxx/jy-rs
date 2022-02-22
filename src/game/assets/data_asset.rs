use anyhow::Result;

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;

/// A source of audio data
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "7a14806a-672b-443b-8d16-4f18afefa465"]
pub struct DataAsset {
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct DataAssetLoader;

impl AssetLoader for DataAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { Ok(load_data(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        &["col", "002"]
    }
}

async fn load_data<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), anyhow::Error> {
    load_context.set_default_asset(LoadedAsset::new(DataAsset { data: bytes.into() }));

    Ok(())
}
