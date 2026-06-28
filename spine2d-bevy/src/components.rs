use bevy::prelude::*;

use crate::{SpineAtlasAsset, SpineSkeletonAsset};

#[derive(Component, Clone, Debug)]
#[require(Transform, Visibility)]
pub struct Spine {
    skeleton: Handle<SpineSkeletonAsset>,
    atlas: Handle<SpineAtlasAsset>,
    animation: Option<String>,
    looped: bool,
    time_scale: f32,
    skin: Option<String>,
}

impl Spine {
    pub fn new(skeleton: Handle<SpineSkeletonAsset>, atlas: Handle<SpineAtlasAsset>) -> Self {
        Self {
            skeleton,
            atlas,
            animation: None,
            looped: true,
            time_scale: 1.0,
            skin: None,
        }
    }

    pub fn get_skeleton(&self) -> &Handle<SpineSkeletonAsset> {
        &self.skeleton
    }

    pub fn set_skeleton(&mut self, skeleton: Handle<SpineSkeletonAsset>) {
        self.skeleton = skeleton;
    }

    pub fn get_atlas(&self) -> &Handle<SpineAtlasAsset> {
        &self.atlas
    }

    pub fn set_atlas(&mut self, atlas: Handle<SpineAtlasAsset>) {
        self.atlas = atlas;
    }

    pub fn get_animation_name(&self) -> Option<&str> {
        self.animation.as_deref()
    }

    pub fn set_animation_name(&mut self, animation: Option<impl Into<String>>) {
        self.animation = animation.map(Into::into);
    }

    pub fn get_loop(&self) -> bool {
        self.looped
    }

    pub fn set_loop(&mut self, looped: bool) {
        self.looped = looped;
    }

    pub fn get_time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn set_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale;
    }

    pub fn get_skin_name(&self) -> Option<&str> {
        self.skin.as_deref()
    }

    pub fn set_skin_name(&mut self, skin: Option<impl Into<String>>) {
        self.skin = skin.map(Into::into);
    }

    pub fn with_animation(mut self, animation: impl Into<String>, looped: bool) -> Self {
        self.set_animation_name(Some(animation));
        self.set_loop(looped);
        self
    }

    pub fn with_skin(mut self, skin: impl Into<String>) -> Self {
        self.set_skin_name(Some(skin));
        self
    }
}

impl Default for Spine {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpineFlipY(bool);

impl SpineFlipY {
    pub fn new(flip_y: bool) -> Self {
        Self(flip_y)
    }

    pub fn flipped() -> Self {
        Self(true)
    }

    pub fn get_flip_y(&self) -> bool {
        self.0
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct SpineAnimation {
    name: Option<String>,
    looped: bool,
    time_scale: f32,
}

impl SpineAnimation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            looped: true,
            time_scale: 1.0,
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: Option<impl Into<String>>) {
        self.name = name.map(Into::into);
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn get_loop(&self) -> bool {
        self.looped
    }

    pub fn set_loop(&mut self, looped: bool) {
        self.looped = looped;
    }

    pub fn with_loop(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }

    pub fn get_time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn set_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale;
    }

    pub fn with_time_scale(mut self, time_scale: f32) -> Self {
        self.time_scale = time_scale;
        self
    }
}

impl Default for SpineAnimation {
    fn default() -> Self {
        Self {
            name: None,
            looped: true,
            time_scale: 1.0,
        }
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct SpineSkin {
    name: Option<String>,
}

impl SpineSkin {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: Option<impl Into<String>>) {
        self.name = name.map(Into::into);
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpineAnimationStateMix {
    from: String,
    to: String,
    duration: f32,
}

impl SpineAnimationStateMix {
    pub fn new(from: impl Into<String>, to: impl Into<String>, duration: f32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            duration,
        }
    }

    pub fn get_from(&self) -> &str {
        &self.from
    }

    pub fn get_to(&self) -> &str {
        &self.to
    }

    pub fn get_duration(&self) -> f32 {
        self.duration
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct SpineAnimationStateConfig {
    default_mix: f32,
    mixes: Vec<SpineAnimationStateMix>,
}

impl SpineAnimationStateConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_default_mix(&self) -> f32 {
        self.default_mix
    }

    pub fn with_default_mix(mut self, default_mix: f32) -> Self {
        self.default_mix = default_mix;
        self
    }

    pub fn get_mixes(&self) -> &[SpineAnimationStateMix] {
        &self.mixes
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
    physics: spine2d::Physics,
    wind: Vec2,
    gravity: Vec2,
    time: Option<f32>,
}

impl SpineSkeletonControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_physics(&self) -> spine2d::Physics {
        self.physics
    }

    pub fn with_physics(mut self, physics: spine2d::Physics) -> Self {
        self.set_physics(physics);
        self
    }

    pub fn set_physics(&mut self, physics: spine2d::Physics) {
        self.physics = physics;
    }

    pub fn get_wind(&self) -> Vec2 {
        self.wind
    }

    pub fn with_wind(mut self, wind: Vec2) -> Self {
        self.set_wind(wind);
        self
    }

    pub fn set_wind(&mut self, wind: Vec2) {
        self.wind = wind;
    }

    pub fn get_gravity(&self) -> Vec2 {
        self.gravity
    }

    pub fn with_gravity(mut self, gravity: Vec2) -> Self {
        self.set_gravity(gravity);
        self
    }

    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    pub fn get_time(&self) -> Option<f32> {
        self.time
    }

    pub fn with_time(mut self, time: f32) -> Self {
        self.set_time(time);
        self
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = Some(time);
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
    min: Vec2,
    max: Vec2,
}

impl SpineBounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn get_min(&self) -> Vec2 {
        self.min
    }

    pub fn get_max(&self) -> Vec2 {
        self.max
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
    ready: bool,
    tracks: Vec<SpineTrackState>,
    skeleton_time: f32,
    physics: spine2d::Physics,
    wind: Vec2,
    gravity: Vec2,
    bounds: SpineBounds,
}

impl SpineRuntimeState {
    pub fn new(
        ready: bool,
        tracks: Vec<SpineTrackState>,
        skeleton_time: f32,
        physics: spine2d::Physics,
        wind: Vec2,
        gravity: Vec2,
        bounds: SpineBounds,
    ) -> Self {
        Self {
            ready,
            tracks,
            skeleton_time,
            physics,
            wind,
            gravity,
            bounds,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn get_tracks(&self) -> &[SpineTrackState] {
        &self.tracks
    }

    pub fn get_skeleton_time(&self) -> f32 {
        self.skeleton_time
    }

    pub fn get_physics(&self) -> spine2d::Physics {
        self.physics
    }

    pub fn get_wind(&self) -> Vec2 {
        self.wind
    }

    pub fn get_gravity(&self) -> Vec2 {
        self.gravity
    }

    pub fn get_bounds(&self) -> &SpineBounds {
        &self.bounds
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpineTrackState {
    track_index: usize,
    animation_name: String,
    track_time: f32,
    animation_time: f32,
    looped: bool,
    delay: f32,
    mix_duration: f32,
    mix_time: f32,
    alpha: f32,
    additive: bool,
    mix_interpolation: spine2d::MixInterpolation,
    reverse: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpineTrackStateParts {
    pub track_index: usize,
    pub animation_name: String,
    pub track_time: f32,
    pub animation_time: f32,
    pub looped: bool,
    pub delay: f32,
    pub mix_duration: f32,
    pub mix_time: f32,
    pub alpha: f32,
    pub additive: bool,
    pub mix_interpolation: spine2d::MixInterpolation,
    pub reverse: bool,
}

impl SpineTrackState {
    pub fn new(parts: SpineTrackStateParts) -> Self {
        Self {
            track_index: parts.track_index,
            animation_name: parts.animation_name,
            track_time: parts.track_time,
            animation_time: parts.animation_time,
            looped: parts.looped,
            delay: parts.delay,
            mix_duration: parts.mix_duration,
            mix_time: parts.mix_time,
            alpha: parts.alpha,
            additive: parts.additive,
            mix_interpolation: parts.mix_interpolation,
            reverse: parts.reverse,
        }
    }

    pub fn get_track_index(&self) -> usize {
        self.track_index
    }

    pub fn get_animation_name(&self) -> &str {
        &self.animation_name
    }

    pub fn get_track_time(&self) -> f32 {
        self.track_time
    }

    pub fn get_animation_time(&self) -> f32 {
        self.animation_time
    }

    pub fn get_loop(&self) -> bool {
        self.looped
    }

    pub fn get_delay(&self) -> f32 {
        self.delay
    }

    pub fn get_mix_duration(&self) -> f32 {
        self.mix_duration
    }

    pub fn get_mix_time(&self) -> f32 {
        self.mix_time
    }

    pub fn get_alpha(&self) -> f32 {
        self.alpha
    }

    pub fn get_additive(&self) -> bool {
        self.additive
    }

    pub fn get_mix_interpolation(&self) -> spine2d::MixInterpolation {
        self.mix_interpolation
    }

    pub fn get_reverse(&self) -> bool {
        self.reverse
    }
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

    pub fn setup_pose(entity: Entity) -> Self {
        Self {
            entity,
            command: SpineSkeletonCommandKind::SetupPose,
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
    SetupPose,
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
            | SpineAnimationCommandKind::ClearMixes => {}
        }
        self
    }

    pub fn set(
        entity: Entity,
        track_index: usize,
        animation: impl Into<String>,
        looped: bool,
    ) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::Set {
                track_index,
                animation: animation.into(),
                looped,
                settings: SpineTrackEntrySettings::default(),
            },
        }
    }

    pub fn add(
        entity: Entity,
        track_index: usize,
        animation: impl Into<String>,
        looped: bool,
        delay: f32,
    ) -> Self {
        Self {
            entity,
            command: SpineAnimationCommandKind::Add {
                track_index,
                animation: animation.into(),
                looped,
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
        looped: bool,
        settings: SpineTrackEntrySettings,
    },
    Add {
        track_index: usize,
        animation: String,
        looped: bool,
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
    ClearMixes,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpineTrackEntrySettings {
    track_end: Option<f32>,
    delay: Option<f32>,
    time_scale: Option<f32>,
    looped: Option<bool>,
    mix_duration: Option<f32>,
    mix_interpolation: Option<spine2d::MixInterpolation>,
    additive: Option<bool>,
    alpha: Option<f32>,
    reverse: Option<bool>,
    shortest_rotation: Option<bool>,
    reset_rotation_directions: bool,
    alpha_attachment_threshold: Option<f32>,
    mix_attachment_threshold: Option<f32>,
    mix_draw_order_threshold: Option<f32>,
    event_threshold: Option<f32>,
    animation_start: Option<f32>,
    animation_end: Option<f32>,
    animation_last: Option<f32>,
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

    pub fn with_loop(mut self, looped: bool) -> Self {
        self.looped = Some(looped);
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

    pub(crate) fn apply(
        &self,
        state: &mut spine2d::AnimationState,
        handle: spine2d::TrackEntryHandle,
    ) {
        if let Some(track_end) = self.track_end {
            handle.set_track_end(state, track_end);
        }

        match (self.mix_duration, self.delay) {
            (Some(mix_duration), Some(delay)) => {
                handle.set_mix_duration_with_delay(state, mix_duration, delay);
            }
            (Some(mix_duration), None) => {
                handle.set_mix_duration(state, mix_duration);
            }
            (None, Some(delay)) => {
                handle.set_delay(state, delay);
            }
            (None, None) => {}
        }

        if let Some(time_scale) = self.time_scale {
            handle.set_time_scale(state, time_scale);
        }
        if let Some(looped) = self.looped {
            handle.set_loop(state, looped);
        }
        if let Some(mix_interpolation) = self.mix_interpolation {
            handle.set_mix_interpolation(state, mix_interpolation);
        }
        if let Some(additive) = self.additive {
            handle.set_additive(state, additive);
        }
        if let Some(alpha) = self.alpha {
            handle.set_alpha(state, alpha);
        }
        if let Some(reverse) = self.reverse {
            handle.set_reverse(state, reverse);
        }
        if let Some(shortest_rotation) = self.shortest_rotation {
            handle.set_shortest_rotation(state, shortest_rotation);
        }
        if self.reset_rotation_directions {
            handle.reset_rotation_directions(state);
        }
        if let Some(threshold) = self.alpha_attachment_threshold {
            handle.set_alpha_attachment_threshold(state, threshold);
        }
        if let Some(threshold) = self.mix_attachment_threshold {
            handle.set_mix_attachment_threshold(state, threshold);
        }
        if let Some(threshold) = self.mix_draw_order_threshold {
            handle.set_mix_draw_order_threshold(state, threshold);
        }
        if let Some(threshold) = self.event_threshold {
            handle.set_event_threshold(state, threshold);
        }
        if let Some(animation_start) = self.animation_start {
            handle.set_animation_start(state, animation_start);
        }
        if let Some(animation_end) = self.animation_end {
            handle.set_animation_end(state, animation_end);
        }
        if let Some(animation_last) = self.animation_last {
            handle.set_animation_last(state, animation_last);
        }
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
    signature: SpineRenderSignature,
}

impl SpineDrawSignatureCache {
    pub(crate) fn get_signature(&self) -> &SpineRenderSignature {
        &self.signature
    }

    pub(crate) fn set_signature(&mut self, signature: SpineRenderSignature) {
        self.signature = signature;
    }

    pub(crate) fn set_render_layers(
        &mut self,
        render_layers: Option<bevy::camera::visibility::RenderLayers>,
    ) {
        self.signature.set_render_layers(render_layers);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpineRenderSignature {
    draws: Vec<SpineDrawSignature>,
    render_layers: Option<bevy::camera::visibility::RenderLayers>,
}

impl SpineRenderSignature {
    pub(crate) fn new(
        draws: Vec<SpineDrawSignature>,
        render_layers: Option<bevy::camera::visibility::RenderLayers>,
    ) -> Self {
        Self {
            draws,
            render_layers,
        }
    }

    pub(crate) fn get_draws(&self) -> &[SpineDrawSignature] {
        &self.draws
    }

    pub(crate) fn get_render_layers(&self) -> Option<&bevy::camera::visibility::RenderLayers> {
        self.render_layers.as_ref()
    }

    pub(crate) fn set_render_layers(
        &mut self,
        render_layers: Option<bevy::camera::visibility::RenderLayers>,
    ) {
        self.render_layers = render_layers;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpineDrawSignature {
    texture_path: String,
    blend: spine2d::BlendMode,
    premultiplied_alpha: bool,
}

impl SpineDrawSignature {
    pub(crate) fn from_draw(draw: &spine2d::Draw, texture_path: String) -> Self {
        Self {
            texture_path,
            blend: draw.blend,
            premultiplied_alpha: draw.premultiplied_alpha,
        }
    }
}

#[derive(Component)]
pub(crate) struct SpineMeshChild {
    mesh: Handle<Mesh>,
}

impl SpineMeshChild {
    pub(crate) fn new(mesh: Handle<Mesh>) -> Self {
        Self { mesh }
    }

    pub(crate) fn get_mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }
}
