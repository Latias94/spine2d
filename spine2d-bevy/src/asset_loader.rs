use anyhow::Result;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use spine2d::{Atlas, SkeletonData};
use std::sync::Arc;

#[derive(Asset, TypePath, Debug)]
pub struct SpineSkeletonAsset {
    pub data: Arc<SkeletonData>,
}

impl SpineSkeletonAsset {
    pub fn info(&self) -> SpineSkeletonInfo<'_> {
        let mut animations = self
            .data
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
            .collect::<Vec<_>>();
        let mut skins = self
            .data
            .skins
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let mut events = self
            .data
            .events
            .keys()
            .map(String::as_str)
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
            .animations
            .iter()
            .map(|animation| animation.name.as_str())
    }

    pub fn skins(&self) -> impl Iterator<Item = &str> {
        self.data.skins.keys().map(String::as_str)
    }

    pub fn events(&self) -> impl Iterator<Item = &str> {
        self.data.events.keys().map(String::as_str)
    }

    pub fn has_animation(&self, name: &str) -> bool {
        self.data.animation(name).is_some()
    }

    pub fn has_skin(&self, name: &str) -> bool {
        self.data.skin(name).is_some()
    }

    pub fn has_event(&self, name: &str) -> bool {
        self.data.events.contains_key(name)
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
        let atlas = Atlas::parse(&atlas_text)?;

        let directory = load_context
            .path()
            .parent()
            .map(|p| p.to_string())
            .unwrap_or("".to_owned())
            .replace('\\', "/") // normalise Windows separators
            .to_string();

        Ok(SpineAtlasAsset { atlas, directory })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skeleton_asset() -> SpineSkeletonAsset {
        SpineSkeletonAsset {
            data: SkeletonData::from_json_str(
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
        }
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
