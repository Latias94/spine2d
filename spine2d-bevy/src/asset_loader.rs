use std::sync::Arc;
use bevy::asset::{AssetLoader, LoadContext, AsyncReadExt};
use bevy::prelude::*;
use spine2d::{SkeletonData, Atlas};
use anyhow::Result;

#[derive(Asset, TypePath, Debug)]
pub struct SpineSkeletonAsset {
    pub data: Arc<SkeletonData>,
}

#[derive(Asset, TypePath, Debug)]
pub struct SpineAtlasAsset {
    pub atlas: Atlas,
    /// Asset-relative directory of the atlas file, e.g. "Spine/spineboy/export".
    /// Used to resolve texture paths like "spineboy-pma.png" at render time.
    pub directory: String,
}

#[derive(Default, TypePath)]
pub struct JsonSkeletonLoader;

impl AssetLoader for JsonSkeletonLoader {
    type Asset = SpineSkeletonAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<SpineSkeletonAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let json_str = String::from_utf8(bytes)?;
        let data = SkeletonData::from_json_str(&json_str)?;
        Ok(SpineSkeletonAsset { data })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Default, TypePath)]
pub struct BinarySkeletonLoader;

impl AssetLoader for BinarySkeletonLoader {
    type Asset = SpineSkeletonAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<SpineSkeletonAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data = SkeletonData::from_skel_bytes(&bytes)?;
        Ok(SpineSkeletonAsset { data })
    }

    fn extensions(&self) -> &[&str] {
        &["skel"]
    }
}

#[derive(Default, TypePath)]
pub struct AtlasLoader;

impl AssetLoader for AtlasLoader {
    type Asset = SpineAtlasAsset;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<SpineAtlasAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let atlas_text = String::from_utf8(bytes)?;
        let atlas = Atlas::from_str(&atlas_text)?;

        let directory = load_context
            .path()
            .parent()
            .and_then(|p| Some(p.to_string()))
            .unwrap_or("".to_owned())
            .replace('\\', "/")  // normalise Windows separators
            .to_string();

        Ok(SpineAtlasAsset { atlas, directory })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas"]
    }
}