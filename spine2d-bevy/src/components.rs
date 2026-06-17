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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SpineBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl SpineBounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpineReady;

#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpineLifecycleEvent {
    pub entity: Entity,
    pub kind: SpineLifecycleEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpineLifecycleEventKind {
    Ready,
    Released(SpineReleaseReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpineReleaseReason {
    ComponentRemoved,
    EntityDespawned,
    ComponentChanged,
    AssetReload,
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

#[derive(Message, Clone, Debug)]
pub struct SpineAnimationCommand {
    pub entity: Entity,
    pub command: SpineAnimationCommandKind,
}

impl SpineAnimationCommand {
    pub fn set(
        entity: Entity,
        track_index: usize,
        animation: impl Into<String>,
        loop_animation: bool,
    ) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::Set {
                track_index,
                animation: animation.into(),
                loop_animation,
            },
        }
    }

    pub fn add(
        entity: Entity,
        track_index: usize,
        animation: impl Into<String>,
        loop_animation: bool,
        delay: f32,
    ) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::Add {
                track_index,
                animation: animation.into(),
                loop_animation,
                delay,
            },
        }
    }

    pub fn set_empty(entity: Entity, track_index: usize, mix_duration: f32) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::SetEmpty {
                track_index,
                mix_duration,
            },
        }
    }

    pub fn add_empty(entity: Entity, track_index: usize, mix_duration: f32, delay: f32) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::AddEmpty {
                track_index,
                mix_duration,
                delay,
            },
        }
    }

    pub fn clear_track(entity: Entity, track_index: usize) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::ClearTrack { track_index },
        }
    }

    pub fn clear_tracks(entity: Entity) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::ClearTracks,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpineAnimationCommandKind {
    Set {
        track_index: usize,
        animation: String,
        loop_animation: bool,
    },
    Add {
        track_index: usize,
        animation: String,
        loop_animation: bool,
        delay: f32,
    },
    SetEmpty {
        track_index: usize,
        mix_duration: f32,
    },
    AddEmpty {
        track_index: usize,
        mix_duration: f32,
        delay: f32,
    },
    ClearTrack {
        track_index: usize,
    },
    ClearTracks,
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpineSystemSet {
    Cleanup,
    Spawn,
    Commands,
    Update,
    Render,
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
