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

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpineFlipY(pub bool);

impl SpineFlipY {
    pub fn new(flip_y: bool) -> Self {
        Self(flip_y)
    }

    pub fn flipped() -> Self {
        Self(true)
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

#[derive(Clone, Debug, PartialEq)]
pub struct SpineAnimationStateMix {
    pub from: String,
    pub to: String,
    pub duration: f32,
}

impl SpineAnimationStateMix {
    pub fn new(from: impl Into<String>, to: impl Into<String>, duration: f32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            duration,
        }
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct SpineAnimationStateConfig {
    pub default_mix: f32,
    pub mixes: Vec<SpineAnimationStateMix>,
}

impl SpineAnimationStateConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_mix(mut self, default_mix: f32) -> Self {
        self.default_mix = default_mix;
        self
    }

    pub fn with_mix(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        duration: f32,
    ) -> Self {
        self.mixes
            .push(SpineAnimationStateMix::new(from, to, duration));
        self
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SpineSkeletonControl {
    pub physics: spine2d::Physics,
    pub wind: Vec2,
    pub gravity: Vec2,
    pub time: Option<f32>,
}

impl SpineSkeletonControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_physics(mut self, physics: spine2d::Physics) -> Self {
        self.physics = physics;
        self
    }

    pub fn with_wind(mut self, wind: Vec2) -> Self {
        self.wind = wind;
        self
    }

    pub fn with_gravity(mut self, gravity: Vec2) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_time(mut self, time: f32) -> Self {
        self.time = Some(time);
        self
    }
}

impl Default for SpineSkeletonControl {
    fn default() -> Self {
        Self {
            physics: spine2d::Physics::None,
            wind: Vec2::new(1.0, 0.0),
            gravity: Vec2::new(0.0, 1.0),
            time: None,
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

#[derive(Component, Clone, Debug, PartialEq)]
pub struct SpineRuntimeState {
    pub ready: bool,
    pub tracks: Vec<SpineTrackState>,
    pub skeleton_time: f32,
    pub physics: spine2d::Physics,
    pub wind: Vec2,
    pub gravity: Vec2,
    pub bounds: SpineBounds,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpineTrackState {
    pub track_index: usize,
    pub animation_index: i32,
    pub animation_name: String,
    pub track_time: f32,
    pub animation_time: f32,
    pub loop_animation: bool,
    pub delay: f32,
    pub mix_duration: f32,
    pub mix_time: f32,
    pub alpha: f32,
    pub additive: bool,
    pub reverse: bool,
}

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

#[derive(Message, Clone, Debug)]
pub struct SpineSkeletonCommand {
    pub entity: Entity,
    pub command: SpineSkeletonCommandKind,
}

impl SpineSkeletonCommand {
    pub fn set_control(entity: Entity, control: SpineSkeletonControl) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::SetControl(control),
        }
    }

    pub fn set_physics(entity: Entity, physics: spine2d::Physics) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::SetPhysics(physics),
        }
    }

    pub fn set_wind(entity: Entity, wind: Vec2) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::SetWind(wind),
        }
    }

    pub fn set_gravity(entity: Entity, gravity: Vec2) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::SetGravity(gravity),
        }
    }

    pub fn set_time(entity: Entity, time: f32) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::SetTime(time),
        }
    }

    pub fn reset_to_setup_pose(entity: Entity) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::ResetToSetupPose,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpineSkeletonCommandKind {
    SetControl(SpineSkeletonControl),
    SetPhysics(spine2d::Physics),
    SetWind(Vec2),
    SetGravity(Vec2),
    SetTime(f32),
    ResetToSetupPose,
}

impl SpineAnimationCommand {
    pub fn with_entry_settings(mut self, settings: SpineTrackEntrySettings) -> Self {
        match &mut self.command {
            SpineAnimationCommandKind::Set { settings: slot, .. }
            | SpineAnimationCommandKind::Add { settings: slot, .. }
            | SpineAnimationCommandKind::SetEmpty { settings: slot, .. }
            | SpineAnimationCommandKind::AddEmpty { settings: slot, .. } => {
                *slot = settings;
            }
            SpineAnimationCommandKind::ClearTrack { .. }
            | SpineAnimationCommandKind::ClearTracks
            | SpineAnimationCommandKind::SetEmptyAnimations { .. }
            | SpineAnimationCommandKind::SetDefaultMix { .. }
            | SpineAnimationCommandKind::SetMix { .. }
            | SpineAnimationCommandKind::RemoveMix { .. }
            | SpineAnimationCommandKind::ClearMixes => {}
        }
        self
    }

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
                settings: SpineTrackEntrySettings::default(),
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
                settings: SpineTrackEntrySettings::default(),
            },
        }
    }

    pub fn set_empty(entity: Entity, track_index: usize, mix_duration: f32) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::SetEmpty {
                track_index,
                mix_duration,
                settings: SpineTrackEntrySettings::default(),
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
                settings: SpineTrackEntrySettings::default(),
            },
        }
    }

    pub fn set_empty_animations(entity: Entity, mix_duration: f32) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::SetEmptyAnimations { mix_duration },
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

    pub fn set_default_mix(entity: Entity, default_mix: f32) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::SetDefaultMix { default_mix },
        }
    }

    pub fn set_mix(
        entity: Entity,
        from: impl Into<String>,
        to: impl Into<String>,
        duration: f32,
    ) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::SetMix {
                from: from.into(),
                to: to.into(),
                duration,
            },
        }
    }

    pub fn remove_mix(entity: Entity, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::RemoveMix {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn clear_mixes(entity: Entity) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::ClearMixes,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SpineAnimationCommandKind {
    Set {
        track_index: usize,
        animation: String,
        loop_animation: bool,
        settings: SpineTrackEntrySettings,
    },
    Add {
        track_index: usize,
        animation: String,
        loop_animation: bool,
        delay: f32,
        settings: SpineTrackEntrySettings,
    },
    SetEmpty {
        track_index: usize,
        mix_duration: f32,
        settings: SpineTrackEntrySettings,
    },
    AddEmpty {
        track_index: usize,
        mix_duration: f32,
        delay: f32,
        settings: SpineTrackEntrySettings,
    },
    SetEmptyAnimations {
        mix_duration: f32,
    },
    ClearTrack {
        track_index: usize,
    },
    ClearTracks,
    SetDefaultMix {
        default_mix: f32,
    },
    SetMix {
        from: String,
        to: String,
        duration: f32,
    },
    RemoveMix {
        from: String,
        to: String,
    },
    ClearMixes,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpineTrackEntrySettings {
    pub track_end: Option<f32>,
    pub delay: Option<f32>,
    pub time_scale: Option<f32>,
    pub mix_duration: Option<f32>,
    pub mix_interpolation: Option<spine2d::MixInterpolation>,
    pub additive: Option<bool>,
    pub alpha: Option<f32>,
    pub reverse: Option<bool>,
    pub shortest_rotation: Option<bool>,
    pub reset_rotation_directions: bool,
    pub alpha_attachment_threshold: Option<f32>,
    pub mix_attachment_threshold: Option<f32>,
    pub mix_draw_order_threshold: Option<f32>,
    pub event_threshold: Option<f32>,
    pub animation_start: Option<f32>,
    pub animation_end: Option<f32>,
    pub animation_last: Option<f32>,
}

impl SpineTrackEntrySettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_track_end(mut self, track_end: f32) -> Self {
        self.track_end = Some(track_end);
        self
    }

    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn with_time_scale(mut self, time_scale: f32) -> Self {
        self.time_scale = Some(time_scale);
        self
    }

    pub fn with_mix_duration(mut self, mix_duration: f32) -> Self {
        self.mix_duration = Some(mix_duration);
        self
    }

    pub fn with_mix_interpolation(mut self, mix_interpolation: spine2d::MixInterpolation) -> Self {
        self.mix_interpolation = Some(mix_interpolation);
        self
    }

    pub fn with_additive(mut self, additive: bool) -> Self {
        self.additive = Some(additive);
        self
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = Some(alpha);
        self
    }

    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = Some(reverse);
        self
    }

    pub fn with_shortest_rotation(mut self, shortest_rotation: bool) -> Self {
        self.shortest_rotation = Some(shortest_rotation);
        self
    }

    pub fn with_reset_rotation_directions(mut self) -> Self {
        self.reset_rotation_directions = true;
        self
    }

    pub fn with_alpha_attachment_threshold(mut self, threshold: f32) -> Self {
        self.alpha_attachment_threshold = Some(threshold);
        self
    }

    pub fn with_mix_attachment_threshold(mut self, threshold: f32) -> Self {
        self.mix_attachment_threshold = Some(threshold);
        self
    }

    pub fn with_mix_draw_order_threshold(mut self, threshold: f32) -> Self {
        self.mix_draw_order_threshold = Some(threshold);
        self
    }

    pub fn with_event_threshold(mut self, threshold: f32) -> Self {
        self.event_threshold = Some(threshold);
        self
    }

    pub fn with_animation_start(mut self, animation_start: f32) -> Self {
        self.animation_start = Some(animation_start);
        self
    }

    pub fn with_animation_end(mut self, animation_end: f32) -> Self {
        self.animation_end = Some(animation_end);
        self
    }

    pub fn with_animation_last(mut self, animation_last: f32) -> Self {
        self.animation_last = Some(animation_last);
        self
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpineSystemSet {
    Cleanup,
    Spawn,
    Config,
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
