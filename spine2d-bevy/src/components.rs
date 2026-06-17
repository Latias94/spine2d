use bevy::prelude::*;
use crate::{SpineSkeletonAsset, SpineAtlasAsset};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpineHandle(pub u32);

#[derive(Component, Default)]
pub struct SpineAnimationPlayer {
    pub animation_name: String,
    pub loop_animation: bool,
    pub time_scale: f32,
}

#[derive(Component, Default)]
pub struct SpineSkin {
    pub skin_name: String,
}

/// The main Spine component. Spawn this to load and display a Spine skeleton.
///
/// `Transform`, `Visibility`, and friends are required and added automatically,
/// so child mesh entities always have a valid parent hierarchy.
#[derive(Component, Clone, Debug)]
#[require(Transform, Visibility)]
pub struct Spine {
    pub skeleton: Handle<SpineSkeletonAsset>,
    pub atlas: Handle<SpineAtlasAsset>,
    pub animation: String,
    pub loop_animation: bool,
    pub time_scale: f32,
    pub skin: Option<String>,
}

impl Default for Spine {
    fn default() -> Self {
        Self {
            skeleton: Default::default(),
            atlas: Default::default(),
            animation: String::new(),
            loop_animation: true,
            time_scale: 1.0,
            skin: None,
        }
    }
}