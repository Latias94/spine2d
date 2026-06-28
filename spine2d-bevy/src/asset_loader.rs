use anyhow::Result;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use spine2d::{Atlas, SkeletonData};
use std::sync::Arc;

#[derive(Asset, TypePath, Debug)]
pub struct SpineSkeletonAsset {
    data: Arc<SkeletonData>,
}

impl SpineSkeletonAsset {
    pub fn new(data: Arc<SkeletonData>) -> Self {
        Self { data }
    }

    pub fn get_data(&self) -> &Arc<SkeletonData> {
        &self.data
    }

    pub fn info(&self) -> SpineSkeletonInfo<'_> {
        let mut animations = self
            .data
            .get_animations()
            .iter()
            .map(|animation| animation.get_name())
            .collect::<Vec<_>>();
        let mut skins = self
            .data
            .get_skins()
            .iter()
            .map(|skin| skin.get_name())
            .collect::<Vec<_>>();
        let mut events = self
            .data
            .get_events()
            .iter()
            .map(|event| event.get_name())
            .collect::<Vec<_>>();

        animations.sort_unstable();
        skins.sort_unstable();
        events.sort_unstable();

        SpineSkeletonInfo {
            animations,
            skins,
            events,
        }
    }

    pub fn animations(&self) -> impl Iterator<Item = &str> {
        self.data
            .get_animations()
            .iter()
            .map(|animation| animation.get_name())
    }

    pub fn skins(&self) -> impl Iterator<Item = &str> {
        self.data.get_skins().iter().map(|skin| skin.get_name())
    }

    pub fn events(&self) -> impl Iterator<Item = &str> {
        self.data.get_events().iter().map(|event| event.get_name())
    }

    pub fn has_animation(&self, name: &str) -> bool {
        self.data.find_animation(name).is_some()
    }

    pub fn has_skin(&self, name: &str) -> bool {
        self.data.find_skin(name).is_some()
    }

    pub fn has_event(&self, name: &str) -> bool {
        self.data.find_event(name).is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpineSkeletonInfo<'a> {
    pub animations: Vec<&'a str>,
    pub skins: Vec<&'a str>,
    pub events: Vec<&'a str>,
}

#[derive(Asset, TypePath, Debug)]
pub struct SpineAtlasAsset {
    atlas: Atlas,
    /// Asset-relative directory of the atlas file, e.g. "Spine/spineboy/export".
    /// Used to resolve texture paths like "spineboy-pma.png" at render time.
    directory: String,
}

impl SpineAtlasAsset {
    pub fn new(atlas: Atlas, directory: String) -> Self {
        Self { atlas, directory }
    }

    pub fn get_atlas(&self) -> &Atlas {
        &self.atlas
    }

    pub fn get_directory(&self) -> &str {
        &self.directory
    }
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
        Ok(SpineSkeletonAsset::new(data))
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
        Ok(SpineSkeletonAsset::new(data))
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
        let atlas = atlas_text.parse::<Atlas>()?;

        let directory = load_context
            .path()
            .parent()
            .map(|p| p.to_string())
            .unwrap_or("".to_owned())
            .replace('\\', "/") // normalise Windows separators
            .to_string();

        Ok(SpineAtlasAsset::new(atlas, directory))
    }

    fn extensions(&self) -> &[&str] {
        &["atlas"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skeleton_asset() -> SpineSkeletonAsset {
        SpineSkeletonAsset::new(
            SkeletonData::from_json_str(
                r#"
                {
                  "skeleton": { "spine": "4.3.00" },
                  "bones": [ { "name": "root" } ],
                  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
                  "skins": {
                    "default": {
                      "slot0": {
                        "mesh0": {
                          "type": "mesh",
                          "path": "mesh0",
                          "uvs": [0,0, 1,0, 1,1, 0,1],
                          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
                          "triangles": [0,1,2, 2,3,0]
                        }
                      }
                    }
                  },
                  "events": { "hit": { "int": 7 } },
                  "animations": {
                    "idle": {},
                    "attack": {}
                  }
                }
                "#,
            )
            .expect("parse skeleton"),
        )
    }

    #[test]
    fn skeleton_asset_reports_available_runtime_names() {
        let asset = skeleton_asset();
        let info = asset.info();

        assert_eq!(info.animations, vec!["attack", "idle"]);
        assert!(info.skins.contains(&"default"));
        assert!(info.events.contains(&"hit"));
        assert!(asset.has_animation("idle"));
        assert!(asset.has_skin("default"));
        assert!(asset.has_event("hit"));
        assert!(!asset.has_animation("missing"));
        assert!(!asset.has_skin("missing"));
        assert!(!asset.has_event("missing"));
    }
}
