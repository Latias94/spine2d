use bevy::prelude::*;

use crate::{SpineAtlasAsset, SpineSkeletonAsset};

#[derive(Component, Clone, Debug)]
#[require(Transform, Visibility)]
pub struct Spine {
    pub skeleton: Handle<SpineSkeletonAsset>,
    pub atlas: Handle<SpineAtlasAsset>,
    pub animation: Option<String>,
    pub loop_animation: bool,
    pub time_scale: f32,
    pub skin: Option<String>,
}

impl Spine {
    pub fn new(skeleton: Handle<SpineSkeletonAsset>, atlas: Handle<SpineAtlasAsset>) -> Self {
        Self {
            skeleton,
            atlas,
            animation: None,
            loop_animation: true,
            time_scale: 1.0,
            skin: None,
        }
    }

    pub fn with_animation(mut self, animation: impl Into<String>, loop_animation: bool) -> Self {
        self.animation = Some(animation.into());
        self.loop_animation = loop_animation;
        self
    }

    pub fn with_skin(mut self, skin: impl Into<String>) -> Self {
        self.skin = Some(skin.into());
        self
    }
}

impl Default for Spine {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct SpineAnimation {
    pub name: Option<String>,
    pub loop_animation: bool,
    pub time_scale: f32,
}

impl SpineAnimation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            loop_animation: true,
            time_scale: 1.0,
        }
    }
}

impl Default for SpineAnimation {
    fn default() -> Self {
        Self {
            name: None,
            loop_animation: true,
            time_scale: 1.0,
        }
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct SpineSkin {
    pub name: Option<String>,
}

impl SpineSkin {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }
}

#[derive(Message, Clone, Debug)]
pub struct SpineAnimationEvent {
    pub entity: Entity,
    pub track_index: usize,
    pub animation_name: String,
    pub track_time: f32,
    pub kind: SpineAnimationEventKind,
}

#[derive(Clone, Debug)]
pub enum SpineAnimationEventKind {
    Start,
    Interrupt,
    End,
    Dispose,
    Complete,
    Event(spine2d::Event),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SpineInstanceId {
    pub index: u32,
    pub generation: u32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SpineInstanceKey(pub SpineInstanceId);

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpineDrawSignatureCache {
    pub signature: SpineRenderSignature,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpineRenderSignature {
    pub draws: Vec<SpineDrawSignature>,
    pub render_layers: Option<bevy::camera::visibility::RenderLayers>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpineDrawSignature {
    pub texture_path: String,
    pub blend: spine2d::BlendMode,
    pub premultiplied_alpha: bool,
}

impl SpineDrawSignature {
    pub fn from_draw(draw: &spine2d::Draw, texture_path: String) -> Self {
        Self {
            texture_path,
            blend: draw.blend,
            premultiplied_alpha: draw.premultiplied_alpha,
        }
    }
}

#[derive(Component)]
pub(crate) struct SpineMeshChild {
    pub mesh: Handle<Mesh>,
}
