use std::sync::Arc;

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};

use bevy::asset::{AssetIoError, AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;

/// A source of audio data
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "7a14806a-672b-443b-8d16-4f18afefa464"]
pub struct GrpAsset {
    /// Raw data of the audio source
    pub idx: Vec<usize>,
    pub data: Arc<[u8]>,
}

impl GrpAsset {
    pub fn idx(&self, i: usize) -> Option<&[u8]> {
        let next = *self.idx.get(i + 1).unwrap_or(&0);
        let cur = *self.idx.get(i).unwrap_or(&0);
        if next - cur <= 0 {
            None
        } else {
            Some(&self.data.as_ref()[cur..next])
        }
    }
}

#[derive(Default)]
pub struct GrpLoader;

impl AssetLoader for GrpLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move { Ok(load_grp(bytes, load_context).await?) })
    }


    fn extensions(&self) -> &[&str] {
        &["grp"]
    }
}

/// Loads an entire glTF file.
async fn load_grp<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), anyhow::Error> {
    let path = load_context.path().to_str().unwrap().to_string();
    println!("try load path {}: {}", path, bytes.len());
    let pos = path.rfind(".").unwrap();
    let idx_file = path.split_at(pos).0.to_string() + ".idx";
    println!("try load idx {}", idx_file);


    let mut idx = vec![0];
    let out = load_context.read_asset_bytes(idx_file).await;
    match out {
        Ok(buffer_bytes) => {
            println!("get idx len {}", buffer_bytes.len());
            let mut cursor = std::io::Cursor::new(buffer_bytes);
            while let Ok(ret) = cursor.read_u32::<LittleEndian>() {
                idx.push(ret as usize);
            }
        }
        Err(err) => {
            if let AssetIoError::NotFound(p) = err {
                println!("not exists...{}", p.to_str().unwrap());
            } else {
                return Err(err.into());
            }
        }
    };
    println!("get idx {:?}", idx);
    load_context.set_default_asset(LoadedAsset::new(GrpAsset {
        idx: idx,
        data: bytes.into(),
    }));

    Ok(())
}
