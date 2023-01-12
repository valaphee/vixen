use anyhow::Result;
use bevy::{
    asset::{AssetIo, AssetIoError, BoxedFuture, Metadata},
    prelude::*,
    utils::HashMap,
};
use futures_lite::future;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

/// File-based content addressed storage asset io.
///
/// Uses an index for associating file paths with their corresponding hashes, which makes updating
/// assets easier, allows version selection, without updating assets, and eliminates file duplicates.
pub struct CasAssetIo {
    parent: Box<dyn AssetIo>,
    index: HashMap<String, Hash>,
}

impl CasAssetIo {
    pub fn new(parent: Box<dyn AssetIo>) -> Self {
        let index = future::block_on(parent.load_path(Path::new("index.json"))).unwrap();

        Self {
            parent,
            index: serde_json::from_slice(index.as_slice()).unwrap(),
        }
    }
}

impl Default for CasAssetIo {
    fn default() -> Self {
        Self::new(AssetPlugin::default().create_platform_default_asset_io())
    }
}

impl AssetIo for CasAssetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        // Get corresponding file hash
        if let Some(hash) = self.index.get(path.to_str().unwrap()) {
            Box::pin(async move {
                let result = self
                    .parent
                    .load_path(Path::new(
                        format!("{}/{}", hex::encode(&hash.0[..1]), hex::encode(&hash.0)).as_str(),
                    ))
                    .await;

                // Check file integrity
                if let Ok(data) = &result {
                    let mut hasher = Sha1::new();
                    hasher.update(data);
                    let data_hash = hasher.finalize();
                    if hash.0 != data_hash.as_slice() {
                        return Err(AssetIoError::NotFound(path.to_path_buf()));
                    }
                }

                result
            })
        } else {
            Box::pin(async move { Err(AssetIoError::NotFound(path.to_path_buf())) })
        }
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        todo!()
    }

    fn get_metadata(&self, path: &Path) -> Result<Metadata, AssetIoError> {
        todo!()
    }

    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Hash(#[serde(with = "hex")] pub(self) Vec<u8>);
