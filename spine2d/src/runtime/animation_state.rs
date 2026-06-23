use super::animation::{
    ANIMATION_STATE_SETUP, MixFrom, RotateTimelineState, StateTimelineApply, apply_state_timeline,
};
use crate::{
    Animation, Error, Event, MixBlend, MixDirection, Skeleton, SkeletonData, TimelineKind,
};
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

const TIME_EPSILON: f32 = 1e-6;
const EMPTY_ANIMATION_INDEX: usize = usize::MAX;
const UNKNOWN_ANIMATION_INDEX: usize = usize::MAX - 1;
const EMPTY_ANIMATION_ID: u64 = u64::MAX;
const EMPTY_ANIMATION_NAME: &str = "<empty>";
static NEXT_STATE_ID: AtomicU64 = AtomicU64::new(1);

fn empty_animation() -> Animation {
    Animation {
        name: EMPTY_ANIMATION_NAME.to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: Vec::new(),
        slot_color_timelines: Vec::new(),
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    }
}

fn animation_identity(animation: &Animation) -> u64 {
    animation as *const Animation as usize as u64
}

// Matches `spine::Property` in upstream `spine-cpp` (bit flags).
const PROPERTY_ROTATE: u64 = 1 << 0;
const PROPERTY_X: u64 = 1 << 1;
const PROPERTY_Y: u64 = 1 << 2;
const PROPERTY_SCALE_X: u64 = 1 << 3;
const PROPERTY_SCALE_Y: u64 = 1 << 4;
const PROPERTY_SHEAR_X: u64 = 1 << 5;
const PROPERTY_SHEAR_Y: u64 = 1 << 6;
const PROPERTY_INHERIT: u64 = 1 << 7;
const PROPERTY_RGB: u64 = 1 << 8;
const PROPERTY_ALPHA: u64 = 1 << 9;
const PROPERTY_RGB2: u64 = 1 << 10;
const PROPERTY_ATTACHMENT: u64 = 1 << 11;
const PROPERTY_DEFORM: u64 = 1 << 12;
#[allow(dead_code)]
const PROPERTY_EVENT: u64 = 1 << 13;
const PROPERTY_DRAW_ORDER: u64 = 1 << 14;
const PROPERTY_IK_CONSTRAINT: u64 = 1 << 15;
const PROPERTY_TRANSFORM_CONSTRAINT: u64 = 1 << 16;
const PROPERTY_PATH_CONSTRAINT_POSITION: u64 = 1 << 17;
const PROPERTY_PATH_CONSTRAINT_SPACING: u64 = 1 << 18;
const PROPERTY_PATH_CONSTRAINT_MIX: u64 = 1 << 19;
const PROPERTY_PHYSICS_CONSTRAINT_INERTIA: u64 = 1 << 20;
const PROPERTY_PHYSICS_CONSTRAINT_STRENGTH: u64 = 1 << 21;
const PROPERTY_PHYSICS_CONSTRAINT_DAMPING: u64 = 1 << 22;
const PROPERTY_PHYSICS_CONSTRAINT_MASS: u64 = 1 << 23;
const PROPERTY_PHYSICS_CONSTRAINT_WIND: u64 = 1 << 24;
const PROPERTY_PHYSICS_CONSTRAINT_GRAVITY: u64 = 1 << 25;
const PROPERTY_PHYSICS_CONSTRAINT_MIX: u64 = 1 << 26;
#[allow(dead_code)]
const PROPERTY_PHYSICS_CONSTRAINT_RESET: u64 = 1 << 27;
const PROPERTY_SEQUENCE: u64 = 1 << 28;
const PROPERTY_SLIDER_TIME: u64 = 1 << 29;
const PROPERTY_SLIDER_MIX: u64 = 1 << 30;
const PROPERTY_DRAW_ORDER_FOLDER: u64 = 1 << 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimelineMode {
    Current,
    Setup,
    First,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimelineApplyMode {
    from: TimelineMode,
    hold: bool,
}

fn property_id(property: u64, data: u32) -> u64 {
    (property << 32) | u64::from(data)
}

fn vertex_attachment_id(attachment: &crate::AttachmentData) -> Option<u32> {
    match attachment {
        crate::AttachmentData::Mesh(m) => Some(m.vertex_id),
        crate::AttachmentData::Path(p) => Some(p.vertex_id),
        crate::AttachmentData::BoundingBox(b) => Some(b.vertex_id),
        crate::AttachmentData::Clipping(c) => Some(c.vertex_id),
        crate::AttachmentData::Region(_) | crate::AttachmentData::Point(_) => None,
    }
}

fn deform_timeline_vertex_id(data: &SkeletonData, timeline: &crate::DeformTimeline) -> Option<u32> {
    let attachment = data
        .skin(timeline.skin.as_str())
        .and_then(|s| s.attachment(timeline.slot_index, timeline.attachment.as_str()))?;

    match attachment {
        crate::AttachmentData::Mesh(m) => {
            let target = data
                .skin(m.timeline_skin.as_str())
                .and_then(|s| s.attachment(timeline.slot_index, m.timeline_attachment.as_str()))?;
            vertex_attachment_id(target)
        }
        _ => vertex_attachment_id(attachment),
    }
}

fn sequence_timeline_sequence_id(
    data: &SkeletonData,
    timeline: &crate::SequenceTimeline,
) -> Option<u32> {
    data.skin(timeline.skin.as_str())
        .and_then(|s| s.attachment(timeline.slot_index, timeline.attachment.as_str()))
        .and_then(|a| match a {
            crate::AttachmentData::Region(r) => r.sequence.as_ref().map(|s| s.id),
            crate::AttachmentData::Mesh(m) => m.sequence.as_ref().map(|s| s.id),
            _ => None,
        })
}

fn timeline_kind_additive(animation: &Animation, kind: TimelineKind) -> bool {
    match kind {
        TimelineKind::Deform(_)
        | TimelineKind::TransformConstraint(_)
        | TimelineKind::SliderMix(_) => true,
        TimelineKind::Bone(i) => {
            !matches!(animation.bone_timelines[i], crate::BoneTimeline::Inherit(_))
        }
        TimelineKind::PathConstraint(i) => matches!(
            animation.path_constraint_timelines[i],
            crate::PathConstraintTimeline::Position(_)
        ),
        TimelineKind::PhysicsConstraint(i) => matches!(
            animation.physics_constraint_timelines[i],
            crate::PhysicsConstraintTimeline::Wind(_)
                | crate::PhysicsConstraintTimeline::Gravity(_)
        ),
        TimelineKind::SlotAttachment(_)
        | TimelineKind::Sequence(_)
        | TimelineKind::SlotColor(_)
        | TimelineKind::SlotRgb(_)
        | TimelineKind::SlotAlpha(_)
        | TimelineKind::SlotRgba2(_)
        | TimelineKind::SlotRgb2(_)
        | TimelineKind::IkConstraint(_)
        | TimelineKind::PhysicsReset(_)
        | TimelineKind::SliderTime(_)
        | TimelineKind::DrawOrder
        | TimelineKind::DrawOrderFolder(_) => false,
    }
}

fn timeline_kind_instant(animation: &Animation, kind: TimelineKind) -> bool {
    match kind {
        TimelineKind::SlotAttachment(_)
        | TimelineKind::Sequence(_)
        | TimelineKind::DrawOrder
        | TimelineKind::DrawOrderFolder(_)
        | TimelineKind::PhysicsReset(_) => true,
        TimelineKind::Bone(i) => {
            matches!(animation.bone_timelines[i], crate::BoneTimeline::Inherit(_))
        }
        TimelineKind::Deform(_)
        | TimelineKind::SlotColor(_)
        | TimelineKind::SlotRgb(_)
        | TimelineKind::SlotAlpha(_)
        | TimelineKind::SlotRgba2(_)
        | TimelineKind::SlotRgb2(_)
        | TimelineKind::IkConstraint(_)
        | TimelineKind::TransformConstraint(_)
        | TimelineKind::PathConstraint(_)
        | TimelineKind::PhysicsConstraint(_)
        | TimelineKind::SliderTime(_)
        | TimelineKind::SliderMix(_) => false,
    }
}

fn entry_additive_blend(blend: MixBlend, entry_additive: bool) -> MixBlend {
    if entry_additive && blend != MixBlend::Setup {
        MixBlend::Add
    } else {
        blend
    }
}

fn timeline_mode_blend(mode: TimelineMode, current_blend: MixBlend) -> MixBlend {
    match mode {
        TimelineMode::Current => current_blend,
        TimelineMode::Setup => MixBlend::Setup,
        TimelineMode::First => MixBlend::First,
    }
}

fn timeline_mode_from(mode: TimelineMode) -> MixFrom {
    match mode {
        TimelineMode::Current => MixFrom::Current,
        TimelineMode::Setup => MixFrom::Setup,
        TimelineMode::First => MixFrom::First,
    }
}

fn draw_order_timeline_out(draw_order: bool, from: TimelineMode) -> Option<bool> {
    if !draw_order && from == TimelineMode::Current {
        return None;
    }

    Some(!draw_order || from == TimelineMode::Current)
}

fn timeline_property_ids(
    data: &SkeletonData,
    animation: &Animation,
    kind: TimelineKind,
) -> Vec<u64> {
    match kind {
        TimelineKind::SlotAttachment(i) => {
            let slot = animation.slot_attachment_timelines[i].slot_index as u32;
            vec![property_id(PROPERTY_ATTACHMENT, slot)]
        }
        TimelineKind::Deform(i) => {
            let t = &animation.deform_timelines[i];
            let deform_id = deform_timeline_vertex_id(data, t).unwrap_or(0);
            let low = (t.slot_index as u32) << 16 | deform_id;
            vec![property_id(PROPERTY_DEFORM, low)]
        }
        TimelineKind::Sequence(i) => {
            let t = &animation.sequence_timelines[i];
            let sequence_id = sequence_timeline_sequence_id(data, t).unwrap_or(0);
            let low = (t.slot_index as u32) << 16 | sequence_id;
            vec![property_id(PROPERTY_SEQUENCE, low)]
        }
        TimelineKind::Bone(i) => match &animation.bone_timelines[i] {
            crate::BoneTimeline::Rotate(t) => {
                vec![property_id(PROPERTY_ROTATE, t.bone_index as u32)]
            }
            crate::BoneTimeline::Translate(t) => vec![
                property_id(PROPERTY_X, t.bone_index as u32),
                property_id(PROPERTY_Y, t.bone_index as u32),
            ],
            crate::BoneTimeline::TranslateX(t) => {
                vec![property_id(PROPERTY_X, t.bone_index as u32)]
            }
            crate::BoneTimeline::TranslateY(t) => {
                vec![property_id(PROPERTY_Y, t.bone_index as u32)]
            }
            crate::BoneTimeline::Scale(t) => vec![
                property_id(PROPERTY_SCALE_X, t.bone_index as u32),
                property_id(PROPERTY_SCALE_Y, t.bone_index as u32),
            ],
            crate::BoneTimeline::ScaleX(t) => {
                vec![property_id(PROPERTY_SCALE_X, t.bone_index as u32)]
            }
            crate::BoneTimeline::ScaleY(t) => {
                vec![property_id(PROPERTY_SCALE_Y, t.bone_index as u32)]
            }
            crate::BoneTimeline::Shear(t) => vec![
                property_id(PROPERTY_SHEAR_X, t.bone_index as u32),
                property_id(PROPERTY_SHEAR_Y, t.bone_index as u32),
            ],
            crate::BoneTimeline::ShearX(t) => {
                vec![property_id(PROPERTY_SHEAR_X, t.bone_index as u32)]
            }
            crate::BoneTimeline::ShearY(t) => {
                vec![property_id(PROPERTY_SHEAR_Y, t.bone_index as u32)]
            }
            crate::BoneTimeline::Inherit(t) => {
                vec![property_id(PROPERTY_INHERIT, t.bone_index as u32)]
            }
        },
        TimelineKind::SlotColor(i) => {
            let slot = animation.slot_color_timelines[i].slot_index as u32;
            vec![
                property_id(PROPERTY_RGB, slot),
                property_id(PROPERTY_ALPHA, slot),
            ]
        }
        TimelineKind::SlotRgb(i) => {
            let slot = animation.slot_rgb_timelines[i].slot_index as u32;
            vec![property_id(PROPERTY_RGB, slot)]
        }
        TimelineKind::SlotAlpha(i) => {
            let slot = animation.slot_alpha_timelines[i].slot_index as u32;
            vec![property_id(PROPERTY_ALPHA, slot)]
        }
        TimelineKind::SlotRgba2(i) => {
            let slot = animation.slot_rgba2_timelines[i].slot_index as u32;
            vec![
                property_id(PROPERTY_RGB, slot),
                property_id(PROPERTY_ALPHA, slot),
                property_id(PROPERTY_RGB2, slot),
            ]
        }
        TimelineKind::SlotRgb2(i) => {
            let slot = animation.slot_rgb2_timelines[i].slot_index as u32;
            vec![
                property_id(PROPERTY_RGB, slot),
                property_id(PROPERTY_RGB2, slot),
            ]
        }
        TimelineKind::IkConstraint(i) => {
            let c = animation.ik_constraint_timelines[i].constraint_index as u32;
            vec![property_id(PROPERTY_IK_CONSTRAINT, c)]
        }
        TimelineKind::TransformConstraint(i) => {
            let c = animation.transform_constraint_timelines[i].constraint_index as u32;
            vec![property_id(PROPERTY_TRANSFORM_CONSTRAINT, c)]
        }
        TimelineKind::PathConstraint(i) => {
            let c = match &animation.path_constraint_timelines[i] {
                crate::PathConstraintTimeline::Position(t) => t.constraint_index as u32,
                crate::PathConstraintTimeline::Spacing(t) => t.constraint_index as u32,
                crate::PathConstraintTimeline::Mix(t) => t.constraint_index as u32,
            };
            match &animation.path_constraint_timelines[i] {
                crate::PathConstraintTimeline::Position(_) => {
                    vec![property_id(PROPERTY_PATH_CONSTRAINT_POSITION, c)]
                }
                crate::PathConstraintTimeline::Spacing(_) => {
                    vec![property_id(PROPERTY_PATH_CONSTRAINT_SPACING, c)]
                }
                crate::PathConstraintTimeline::Mix(_) => {
                    vec![property_id(PROPERTY_PATH_CONSTRAINT_MIX, c)]
                }
            }
        }
        TimelineKind::PhysicsConstraint(i) => {
            let (constraint_index, property) = match &animation.physics_constraint_timelines[i] {
                crate::PhysicsConstraintTimeline::Inertia(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_INERTIA)
                }
                crate::PhysicsConstraintTimeline::Strength(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_STRENGTH)
                }
                crate::PhysicsConstraintTimeline::Damping(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_DAMPING)
                }
                crate::PhysicsConstraintTimeline::Mass(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_MASS)
                }
                crate::PhysicsConstraintTimeline::Wind(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_WIND)
                }
                crate::PhysicsConstraintTimeline::Gravity(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_GRAVITY)
                }
                crate::PhysicsConstraintTimeline::Mix(t) => {
                    (t.constraint_index, PROPERTY_PHYSICS_CONSTRAINT_MIX)
                }
            };
            vec![property_id(property, constraint_index as u32)]
        }
        TimelineKind::SliderTime(i) => {
            let c = animation.slider_time_timelines[i].constraint_index as u32;
            vec![property_id(PROPERTY_SLIDER_TIME, c)]
        }
        TimelineKind::SliderMix(i) => {
            let c = animation.slider_mix_timelines[i].constraint_index as u32;
            vec![property_id(PROPERTY_SLIDER_MIX, c)]
        }
        TimelineKind::DrawOrder => vec![property_id(PROPERTY_DRAW_ORDER, 0)],
        TimelineKind::DrawOrderFolder(i) => animation.draw_order_folder_timelines[i]
            .slots
            .iter()
            .map(|&slot| property_id(PROPERTY_DRAW_ORDER_FOLDER, slot as u32))
            .collect(),
        TimelineKind::PhysicsReset(_) => vec![property_id(PROPERTY_PHYSICS_CONSTRAINT_RESET, 0)],
    }
}

fn animation_has_any_property(data: &SkeletonData, animation: &Animation, ids: &[u64]) -> bool {
    if ids.is_empty() {
        return false;
    }
    let want: HashSet<u64> = ids.iter().copied().collect();
    for kind in animation_timeline_order(animation).iter().copied() {
        let props = timeline_property_ids(data, animation, kind);
        if props.iter().any(|p| want.contains(p)) {
            return true;
        }
    }
    false
}

fn animation_timeline_order(animation: &Animation) -> &[TimelineKind] {
    animation.timeline_order.as_slice()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct EntryId {
    index: usize,
    generation: u32,
}

#[derive(Debug)]
struct EntrySlot {
    generation: u32,
    entry: Option<TrackEntry>,
}

#[derive(Clone, Debug)]
pub struct AnimationStateData {
    skeleton_data: Arc<SkeletonData>,
    default_mix: f32,
    mixes: HashMap<(String, String), f32>,
}

impl AnimationStateData {
    pub fn new(skeleton_data: Arc<SkeletonData>) -> Self {
        Self {
            skeleton_data,
            default_mix: 0.0,
            mixes: HashMap::new(),
        }
    }

    pub fn skeleton_data(&self) -> &SkeletonData {
        self.skeleton_data.as_ref()
    }

    pub fn default_mix(&self) -> f32 {
        self.default_mix
    }

    pub fn set_default_mix(&mut self, duration: f32) {
        self.default_mix = duration;
    }

    pub fn set_mix(&mut self, from: &str, to: &str, duration: f32) -> Result<(), Error> {
        self.skeleton_data
            .animation(from)
            .ok_or_else(|| Error::UnknownAnimation {
                name: from.to_string(),
            })?;
        self.skeleton_data
            .animation(to)
            .ok_or_else(|| Error::UnknownAnimation {
                name: to.to_string(),
            })?;
        self.mixes
            .insert((from.to_string(), to.to_string()), duration);
        Ok(())
    }

    pub fn set_mix_animation(&mut self, from: &Animation, to: &Animation, duration: f32) {
        self.mixes
            .insert((from.name.clone(), to.name.clone()), duration);
    }

    pub fn get_mix_animation(&self, from: &Animation, to: &Animation) -> f32 {
        self.mix_duration(&from.name, &to.name)
    }

    pub fn clear(&mut self) {
        self.default_mix = 0.0;
        self.mixes.clear();
    }

    fn mix_duration(&self, from: &str, to: &str) -> f32 {
        self.mixes
            .get(&(from.to_string(), to.to_string()))
            .copied()
            .unwrap_or(self.default_mix)
    }
}

pub struct TrackEntry {
    track_index: usize,
    animation_index: usize,
    animation_identity: u64,
    animation: Animation,
    looped: bool,
    hold_previous: bool,
    mix_blend: MixBlend,
    reverse: bool,
    shortest_rotation: bool,
    keep_hold: bool,

    animation_start: f32,
    animation_end: f32,
    mix_duration: f32,
    mix_time: f32,
    previous: Option<EntryId>,
    next: Option<EntryId>,
    mixing_from: Option<EntryId>,
    delay: f32,
    track_time: f32,
    track_end: f32,
    time_scale: f32,

    animation_last_time: f32,
    track_last_time: f32,
    next_animation_last_time: f32,
    next_track_last_time: f32,

    alpha: f32,
    total_alpha: f32,
    mixing_to: Option<EntryId>,
    alpha_attachment_threshold: f32,
    mix_attachment_threshold: f32,
    mix_draw_order_threshold: f32,
    event_threshold: f32,

    listener: Option<Box<dyn TrackEntryListener>>,

    timeline_mode: Vec<TimelineApplyMode>,
    timeline_hold_mix: Vec<Option<EntryId>>,
    rotation_state: Vec<f32>,
}

impl std::fmt::Debug for TrackEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackEntry")
            .field("track_index", &self.track_index)
            .field("animation_index", &self.animation_index)
            .field("animation_identity", &self.animation_identity)
            .field("animation", &self.animation)
            .field("looped", &self.looped)
            .field("hold_previous", &self.hold_previous)
            .field("mix_blend", &self.mix_blend)
            .field("reverse", &self.reverse)
            .field("shortest_rotation", &self.shortest_rotation)
            .field("animation_start", &self.animation_start)
            .field("animation_end", &self.animation_end)
            .field("mix_duration", &self.mix_duration)
            .field("mix_time", &self.mix_time)
            .field("previous", &self.previous)
            .field("next", &self.next)
            .field("mixing_from", &self.mixing_from)
            .field("delay", &self.delay)
            .field("track_time", &self.track_time)
            .field("track_end", &self.track_end)
            .field("time_scale", &self.time_scale)
            .field("animation_last_time", &self.animation_last_time)
            .field("track_last_time", &self.track_last_time)
            .field("event_threshold", &self.event_threshold)
            .finish()
    }
}

impl TrackEntry {
    fn new(
        track_index: usize,
        animation_index: usize,
        animation_identity: u64,
        animation: &Animation,
        looped: bool,
    ) -> Self {
        let track_end = f32::INFINITY;
        Self {
            track_index,
            animation_index,
            animation_identity,
            animation: animation.clone(),
            looped,
            hold_previous: false,
            mix_blend: MixBlend::Replace,
            reverse: false,
            shortest_rotation: false,
            keep_hold: false,
            animation_start: 0.0,
            animation_end: animation.duration,
            mix_duration: 0.0,
            mix_time: 0.0,
            previous: None,
            next: None,
            mixing_from: None,
            delay: 0.0,
            track_time: 0.0,
            track_end,
            time_scale: 1.0,
            animation_last_time: -1.0,
            track_last_time: -1.0,
            next_animation_last_time: -1.0,
            next_track_last_time: -1.0,
            alpha: 1.0,
            total_alpha: 0.0,
            mixing_to: None,
            alpha_attachment_threshold: 0.0,
            mix_attachment_threshold: 0.0,
            mix_draw_order_threshold: 0.0,
            event_threshold: 0.0,
            listener: None,
            timeline_mode: Vec::new(),
            timeline_hold_mix: Vec::new(),
            rotation_state: Vec::new(),
        }
    }

    pub fn track_index(&self) -> usize {
        self.track_index
    }

    pub fn animation(&self) -> &Animation {
        &self.animation
    }

    pub fn looped(&self) -> bool {
        self.looped
    }

    pub fn is_complete(&self) -> bool {
        self.track_time >= self.animation_end - self.animation_start
    }

    pub fn was_applied(&self) -> bool {
        self.next_track_last_time != -1.0
    }

    pub fn is_empty_animation(&self) -> bool {
        self.animation_identity == EMPTY_ANIMATION_ID
    }

    pub fn hold_previous(&self) -> bool {
        self.hold_previous
    }

    pub fn mix_blend(&self) -> MixBlend {
        self.mix_blend
    }

    pub fn reverse(&self) -> bool {
        self.reverse
    }

    pub fn shortest_rotation(&self) -> bool {
        self.shortest_rotation
    }

    pub fn animation_start(&self) -> f32 {
        self.animation_start
    }

    pub fn animation_end(&self) -> f32 {
        self.animation_end
    }

    pub fn animation_last(&self) -> f32 {
        self.animation_last_time
    }

    pub fn delay(&self) -> f32 {
        self.delay
    }

    pub fn track_time(&self) -> f32 {
        self.track_time
    }

    pub fn track_end(&self) -> f32 {
        self.track_end
    }

    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn mix_duration(&self) -> f32 {
        self.mix_duration
    }

    pub fn mix_time(&self) -> f32 {
        self.mix_time
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn alpha_attachment_threshold(&self) -> f32 {
        self.alpha_attachment_threshold
    }

    pub fn mix_attachment_threshold(&self) -> f32 {
        self.mix_attachment_threshold
    }

    pub fn mix_draw_order_threshold(&self) -> f32 {
        self.mix_draw_order_threshold
    }

    pub fn event_threshold(&self) -> f32 {
        self.event_threshold
    }

    pub fn animation_time(&self) -> f32 {
        if self.looped {
            let duration = self.animation_end - self.animation_start;
            if duration.abs() <= TIME_EPSILON {
                return self.animation_start;
            }
            // Keep it in [0, duration).
            let mut t = self.track_time % duration;
            if t < 0.0 {
                t += duration;
            }
            // When looping with a non-zero AnimationStart, treat exact loop boundaries as AnimationEnd.
            // This avoids wrapping to AnimationStart and matches Spine's event/complete behavior tests.
            if self.animation_start.abs() > TIME_EPSILON
                && self.track_time > 0.0
                && t.abs() <= TIME_EPSILON
            {
                t = duration;
            }
            t + self.animation_start
        } else {
            let animation_time = self.track_time + self.animation_start;
            if self.animation_end + TIME_EPSILON >= self.animation.duration {
                animation_time
            } else {
                animation_time.min(self.animation_end)
            }
        }
    }

    pub fn track_complete(&self) -> f32 {
        let duration = self.animation_end - self.animation_start;
        if duration != 0.0 {
            if self.looped {
                return duration * (1.0 + (self.track_time / duration).floor());
            }
            if self.track_time < duration {
                return duration;
            }
        }
        self.track_time
    }

    fn mix_percent(&self) -> f32 {
        if self.mix_duration == 0.0 {
            return 1.0;
        }

        let mix = self.mix_time / self.mix_duration;
        if mix > 1.0 { 1.0 } else { mix }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackEntryHandle {
    state_id: u64,
    id: EntryId,
}

impl TrackEntryHandle {
    pub fn animation_state<'a>(&self, state: &'a AnimationState) -> Option<&'a AnimationState> {
        state.entry_for_handle(*self)?;
        Some(state)
    }

    pub fn with_entry<R>(
        &self,
        state: &AnimationState,
        f: impl FnOnce(&TrackEntry) -> R,
    ) -> Option<R> {
        state.entry_for_handle(*self).map(f)
    }

    pub fn mixing_from(&self, state: &AnimationState) -> Option<TrackEntryHandle> {
        state
            .entry_for_handle(*self)
            .and_then(|entry| entry.mixing_from)
            .map(|id| state.handle(id))
    }

    pub fn mixing_to(&self, state: &AnimationState) -> Option<TrackEntryHandle> {
        state
            .entry_for_handle(*self)
            .and_then(|entry| entry.mixing_to)
            .map(|id| state.handle(id))
    }

    pub fn previous(&self, state: &AnimationState) -> Option<TrackEntryHandle> {
        state
            .entry_for_handle(*self)
            .and_then(|entry| entry.previous)
            .map(|id| state.handle(id))
    }

    pub fn next(&self, state: &AnimationState) -> Option<TrackEntryHandle> {
        state
            .entry_for_handle(*self)
            .and_then(|entry| entry.next)
            .map(|id| state.handle(id))
    }

    pub fn is_next_ready(&self, state: &AnimationState) -> bool {
        let Some(entry) = state.entry_for_handle(*self) else {
            return false;
        };
        let Some(next) = self
            .next(state)
            .and_then(|handle| state.entry_for_handle(handle))
        else {
            return false;
        };
        entry.next_track_last_time - next.delay >= 0.0
    }

    fn with_entry_mut(&self, state: &mut AnimationState, f: impl FnOnce(&mut TrackEntry)) {
        if let Some(entry) = state.entry_for_handle_mut(*self) {
            f(entry);
        }
    }

    pub fn set_listener<L: TrackEntryListener + 'static>(
        &self,
        state: &mut AnimationState,
        listener: L,
    ) {
        self.with_entry_mut(state, |entry| {
            entry.listener = Some(Box::new(listener));
        });
    }

    pub fn set_track_end(&self, state: &mut AnimationState, track_end: f32) {
        self.with_entry_mut(state, |entry| {
            entry.track_end = track_end;
        });
    }

    pub fn set_animation(&self, state: &mut AnimationState, animation: &Animation) {
        let animation_index = state
            .data
            .skeleton_data()
            .animation(&animation.name)
            .map(|(animation_index, _)| animation_index)
            .unwrap_or(UNKNOWN_ANIMATION_INDEX);
        self.with_entry_mut(state, |entry| {
            entry.animation_identity = animation_identity(animation);
            entry.animation_index = animation_index;
            entry.animation = animation.clone();
        });
    }

    pub fn set_delay(&self, state: &mut AnimationState, delay: f32) {
        if delay < 0.0 {
            return;
        }
        self.with_entry_mut(state, |entry| {
            entry.delay = delay;
        });
    }

    pub fn set_time_scale(&self, state: &mut AnimationState, time_scale: f32) {
        self.with_entry_mut(state, |entry| {
            entry.time_scale = time_scale;
        });
    }

    pub fn set_track_time(&self, state: &mut AnimationState, track_time: f32) {
        self.with_entry_mut(state, |entry| {
            entry.track_time = track_time;
        });
    }

    pub fn set_loop(&self, state: &mut AnimationState, looped: bool) {
        self.with_entry_mut(state, |entry| {
            entry.looped = looped;
        });
    }

    pub fn set_mix_duration(&self, state: &mut AnimationState, mix_duration: f32) {
        self.with_entry_mut(state, |entry| {
            entry.mix_duration = mix_duration;
        });
    }

    pub fn set_mix_time(&self, state: &mut AnimationState, mix_time: f32) {
        self.with_entry_mut(state, |entry| {
            entry.mix_time = mix_time;
        });
    }

    pub fn set_mix_duration_with_delay(
        &self,
        state: &mut AnimationState,
        mix_duration: f32,
        delay: f32,
    ) {
        let previous = state.previous_entry_for(self.id);
        let resolved_delay = if delay > 0.0 {
            delay
        } else if delay <= 0.0 {
            previous
                .and_then(|id| state.entry(id))
                .map(|entry| (delay + entry.track_complete() - mix_duration).max(0.0))
                .unwrap_or(0.0)
        } else {
            delay
        };

        self.with_entry_mut(state, |entry| {
            entry.mix_duration = mix_duration;
            entry.delay = resolved_delay;
        });
    }

    pub fn set_mix_blend(&self, state: &mut AnimationState, mix_blend: MixBlend) {
        self.with_entry_mut(state, |entry| {
            entry.mix_blend = mix_blend;
        });
    }

    pub fn set_hold_previous(&self, state: &mut AnimationState, hold_previous: bool) {
        self.with_entry_mut(state, |entry| {
            entry.hold_previous = hold_previous;
        });
    }

    pub fn set_alpha(&self, state: &mut AnimationState, alpha: f32) {
        self.with_entry_mut(state, |entry| {
            entry.alpha = alpha;
        });
    }

    pub fn set_reverse(&self, state: &mut AnimationState, reverse: bool) {
        self.with_entry_mut(state, |entry| {
            entry.reverse = reverse;
        });
    }

    pub fn set_shortest_rotation(&self, state: &mut AnimationState, shortest_rotation: bool) {
        self.with_entry_mut(state, |entry| {
            entry.shortest_rotation = shortest_rotation;
        });
    }

    pub fn reset_rotation_directions(&self, state: &mut AnimationState) {
        self.with_entry_mut(state, |entry| {
            entry.rotation_state.clear();
        });
    }

    pub fn set_alpha_attachment_threshold(&self, state: &mut AnimationState, threshold: f32) {
        self.with_entry_mut(state, |entry| {
            entry.alpha_attachment_threshold = threshold;
        });
    }

    pub fn set_mix_attachment_threshold(&self, state: &mut AnimationState, threshold: f32) {
        self.with_entry_mut(state, |entry| {
            entry.mix_attachment_threshold = threshold;
        });
    }

    pub fn set_mix_draw_order_threshold(&self, state: &mut AnimationState, threshold: f32) {
        self.with_entry_mut(state, |entry| {
            entry.mix_draw_order_threshold = threshold;
        });
    }

    pub fn set_event_threshold(&self, state: &mut AnimationState, threshold: f32) {
        self.with_entry_mut(state, |entry| {
            entry.event_threshold = threshold;
        });
    }

    pub fn set_animation_start(&self, state: &mut AnimationState, animation_start: f32) {
        self.with_entry_mut(state, |entry| {
            entry.animation_start = animation_start;
        });
    }

    pub fn set_animation_end(&self, state: &mut AnimationState, animation_end: f32) {
        self.with_entry_mut(state, |entry| {
            entry.animation_end = animation_end;
        });
    }

    pub fn set_animation_last(&self, state: &mut AnimationState, animation_last: f32) {
        self.with_entry_mut(state, |entry| {
            entry.animation_last_time = animation_last;
            entry.next_animation_last_time = animation_last;
        });
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackEntrySettings {
    pub track_end: Option<f32>,
    pub delay: Option<f32>,
    pub time_scale: Option<f32>,
    pub looped: Option<bool>,
    pub mix_duration: Option<f32>,
    pub mix_blend: Option<MixBlend>,
    pub hold_previous: Option<bool>,
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

#[derive(Clone, Debug)]
pub enum TrackAnimationInput<'a> {
    Name(Cow<'a, str>),
    Animation(&'a Animation),
}

impl<'a> From<&'a str> for TrackAnimationInput<'a> {
    fn from(value: &'a str) -> Self {
        Self::Name(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a String> for TrackAnimationInput<'a> {
    fn from(value: &'a String) -> Self {
        Self::Name(Cow::Borrowed(value.as_str()))
    }
}

impl<'a> From<String> for TrackAnimationInput<'a> {
    fn from(value: String) -> Self {
        Self::Name(Cow::Owned(value))
    }
}

impl<'a> From<&'a Animation> for TrackAnimationInput<'a> {
    fn from(value: &'a Animation) -> Self {
        Self::Animation(value)
    }
}

impl TrackEntrySettings {
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

    pub fn with_looped(mut self, looped: bool) -> Self {
        self.looped = Some(looped);
        self
    }

    pub fn with_mix_duration(mut self, mix_duration: f32) -> Self {
        self.mix_duration = Some(mix_duration);
        self
    }

    pub fn with_mix_blend(mut self, mix_blend: MixBlend) -> Self {
        self.mix_blend = Some(mix_blend);
        self
    }

    pub fn with_hold_previous(mut self, hold_previous: bool) -> Self {
        self.hold_previous = Some(hold_previous);
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

    pub fn apply(&self, state: &mut AnimationState, handle: TrackEntryHandle) {
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
        if let Some(mix_blend) = self.mix_blend {
            handle.set_mix_blend(state, mix_blend);
        }
        if let Some(hold_previous) = self.hold_previous {
            handle.set_hold_previous(state, hold_previous);
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

#[derive(Clone, Debug)]
pub struct TrackEntrySnapshot {
    pub track_index: usize,
    pub animation_name: String,
    pub track_time: f32,
    pub animation_time: f32,
    pub looped: bool,
    pub delay: f32,
    pub mix_duration: f32,
    pub mix_time: f32,
    pub alpha: f32,
    pub hold_previous: bool,
    pub mix_blend: MixBlend,
    pub reverse: bool,
}

#[derive(Clone, Debug)]
pub enum AnimationStateEvent {
    Start,
    Interrupt,
    End,
    Dispose,
    Complete,
    Event(Event),
}

pub trait TrackEntryListener {
    fn on_event(
        &mut self,
        state: &mut AnimationState,
        entry: &TrackEntrySnapshot,
        event: &AnimationStateEvent,
    );
}

pub trait AnimationStateListener {
    fn on_event(
        &mut self,
        state: &mut AnimationState,
        entry: &TrackEntrySnapshot,
        event: &AnimationStateEvent,
    );
}

#[derive(Clone, Debug)]
struct QueuedEvent {
    entry: EntryId,
    event: AnimationStateEvent,
}

#[derive(Default)]
struct Track {
    current: Option<EntryId>,
    queue: VecDeque<EntryId>,
}

pub struct AnimationState {
    state_id: u64,
    data: AnimationStateData,
    tracks: Vec<Track>,
    entries: Vec<EntrySlot>,
    free_list: Vec<usize>,
    event_queue: VecDeque<QueuedEvent>,
    time: Cell<f32>,
    time_scale: f32,
    listener: Option<Box<dyn AnimationStateListener>>,
    draining_events: bool,
    drain_disabled: bool,
    manual_track_entry_disposal: bool,
    animations_changed: bool,
    property_ids: HashMap<u64, EntryId>,
    unkeyed_state: i32,
}

impl AnimationState {
    pub fn new(data: AnimationStateData) -> Self {
        Self {
            state_id: NEXT_STATE_ID.fetch_add(1, Ordering::Relaxed),
            data,
            tracks: Vec::new(),
            entries: Vec::new(),
            free_list: Vec::new(),
            event_queue: VecDeque::new(),
            time: Cell::new(0.0),
            time_scale: 1.0,
            listener: None,
            draining_events: false,
            drain_disabled: false,
            manual_track_entry_disposal: false,
            animations_changed: false,
            property_ids: HashMap::new(),
            unkeyed_state: 0,
        }
    }

    pub fn set_listener<L: AnimationStateListener + 'static>(&mut self, listener: L) {
        self.listener = Some(Box::new(listener));
    }

    pub fn disable_queue(&mut self) {
        self.drain_disabled = true;
    }

    pub fn enable_queue(&mut self) {
        self.drain_disabled = false;
    }

    pub fn set_manual_track_entry_disposal(&mut self, manual: bool) {
        self.manual_track_entry_disposal = manual;
    }

    pub fn manual_track_entry_disposal(&self) -> bool {
        self.manual_track_entry_disposal
    }

    pub fn dispose_track_entry(&mut self, handle: TrackEntryHandle) {
        if handle.state_id == self.state_id {
            self.free_entry(handle.id);
        }
    }

    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn set_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale;
    }

    pub fn data_mut(&mut self) -> &mut AnimationStateData {
        &mut self.data
    }

    pub fn data(&self) -> &AnimationStateData {
        &self.data
    }

    pub fn tracks(&self) -> Vec<Option<TrackEntryHandle>> {
        self.tracks
            .iter()
            .map(|track| track.current.map(|id| self.handle(id)))
            .collect()
    }

    fn handle(&self, id: EntryId) -> TrackEntryHandle {
        TrackEntryHandle {
            state_id: self.state_id,
            id,
        }
    }

    fn entry_for_handle(&self, handle: TrackEntryHandle) -> Option<&TrackEntry> {
        if handle.state_id != self.state_id {
            return None;
        }
        self.entry(handle.id)
    }

    fn entry_for_handle_mut(&mut self, handle: TrackEntryHandle) -> Option<&mut TrackEntry> {
        if handle.state_id != self.state_id {
            return None;
        }
        self.entry_mut(handle.id)
    }

    fn previous_entry_for(&self, entry_id: EntryId) -> Option<EntryId> {
        self.entry(entry_id).and_then(|entry| entry.previous)
    }

    fn compute_mix_from(
        &mut self,
        track_id: EntryId,
        kind: TimelineKind,
        ids: &[u64],
    ) -> TimelineMode {
        let mut mix_from = TimelineMode::Setup;
        for (i, id) in ids.iter().copied().enumerate() {
            match self.property_ids.get(&id).copied() {
                None => {
                    self.property_ids.insert(id, track_id);
                }
                Some(owner) if owner == track_id => {
                    mix_from = TimelineMode::First;
                }
                Some(_) => {
                    for id in ids.iter().skip(i + 1).copied() {
                        self.property_ids.entry(id).or_insert(track_id);
                    }
                    return TimelineMode::Current;
                }
            }
        }

        if matches!(kind, TimelineKind::DrawOrderFolder(_)) {
            let draw_order_id = property_id(PROPERTY_DRAW_ORDER, 0);
            if let Some(owner) = self.property_ids.get(&draw_order_id).copied() {
                return if owner == track_id {
                    TimelineMode::First
                } else {
                    TimelineMode::Current
                };
            }
        }

        mix_from
    }

    fn animations_changed(&mut self) {
        self.animations_changed = false;
        self.property_ids.clear();

        let current_ids = self
            .tracks
            .iter()
            .filter_map(|t| t.current)
            .collect::<Vec<_>>();
        for track_id in current_ids {
            let mut entry_id = track_id;
            while let Some(from) = self.entry(entry_id).and_then(|e| e.mixing_from) {
                entry_id = from;
            }

            let mut chain = Vec::new();
            let mut cur = Some(entry_id);
            while let Some(id) = cur {
                chain.push(id);
                cur = self.entry(id).and_then(|e| e.mixing_to);
            }

            for id in chain {
                self.compute_hold(id, track_id);
            }
        }
    }

    fn compute_hold(&mut self, entry_id: EntryId, track_id: EntryId) {
        let (animation, to_id, keep_hold, previous_timeline_mode) = match self.entry(entry_id) {
            Some(entry) => (
                entry.animation.clone(),
                entry.mixing_to,
                entry.keep_hold,
                entry.timeline_mode.clone(),
            ),
            None => return,
        };
        let to_hold_previous = to_id
            .and_then(|id| self.entry(id).map(|entry| entry.hold_previous))
            .unwrap_or(false);

        let kinds = animation_timeline_order(&animation).to_vec();
        let mut timeline_mode = vec![
            TimelineApplyMode {
                from: TimelineMode::Setup,
                hold: false,
            };
            kinds.len()
        ];
        let mut timeline_hold_mix = vec![None; kinds.len()];

        for (i, kind) in kinds.iter().copied().enumerate() {
            let ids = timeline_property_ids(&self.data.skeleton_data, &animation, kind);
            let mix_from = self.compute_mix_from(track_id, kind, &ids);
            if to_hold_previous {
                timeline_mode[i].from = if mix_from == TimelineMode::Setup {
                    TimelineMode::Setup
                } else {
                    TimelineMode::Current
                };
                timeline_mode[i].hold = true;
                continue;
            }
            timeline_mode[i].from = mix_from;

            let Some(to_id) = to_id else {
                continue;
            };

            if timeline_kind_instant(&animation, kind) {
                continue;
            }

            let timeline_additive = timeline_kind_additive(&animation, kind);
            let entry_additive = self
                .entry(entry_id)
                .is_some_and(|e| e.mix_blend == MixBlend::Add);
            if entry_additive && timeline_additive {
                continue;
            }

            let to_holds_property = match self.entry(to_id) {
                Some(to) => {
                    !(to.mix_blend == MixBlend::Add && timeline_additive)
                        && animation_has_any_property(&self.data.skeleton_data, &to.animation, &ids)
                }
                None => {
                    continue;
                }
            };

            if !to_holds_property {
                continue;
            }

            let mut next = self.entry(to_id).and_then(|e| e.mixing_to);
            let mut hold_mix = None;
            while let Some(next_id) = next {
                let Some(next_entry) = self.entry(next_id) else {
                    break;
                };

                if next_entry.mix_blend == MixBlend::Add && timeline_additive
                    || !animation_has_any_property(
                        &self.data.skeleton_data,
                        &next_entry.animation,
                        &ids,
                    )
                {
                    if next_entry.mix_duration > 0.0 {
                        hold_mix = Some(next_id);
                    }
                    break;
                }

                next = next_entry.mixing_to;
            }

            timeline_mode[i].hold = true;
            timeline_hold_mix[i] = hold_mix;
        }

        if keep_hold && !to_hold_previous {
            for (mode, previous) in timeline_mode.iter_mut().zip(previous_timeline_mode.iter()) {
                mode.hold = previous.hold;
            }
        }

        if let Some(entry) = self.entry_mut(entry_id) {
            entry.timeline_mode = timeline_mode;
            entry.timeline_hold_mix = timeline_hold_mix;
        }
    }

    pub fn current(&self, track_index: usize) -> Option<TrackEntryHandle> {
        let id = self.tracks.get(track_index)?.current?;
        self.entry(id)?;
        Some(self.handle(id))
    }

    fn resolve_track_animation<'a>(
        &'a self,
        input: TrackAnimationInput<'a>,
    ) -> Result<(u64, usize, Animation), Error> {
        match input {
            TrackAnimationInput::Name(name) => {
                let (animation_index, animation) = self
                    .data
                    .skeleton_data()
                    .animation(name.as_ref())
                    .ok_or_else(|| Error::UnknownAnimation {
                        name: name.into_owned(),
                    })?;
                Ok((
                    animation_identity(animation),
                    animation_index,
                    animation.clone(),
                ))
            }
            TrackAnimationInput::Animation(animation) => {
                let animation_index = self
                    .data
                    .skeleton_data()
                    .animation(&animation.name)
                    .map(|(animation_index, _)| animation_index)
                    .unwrap_or(UNKNOWN_ANIMATION_INDEX);
                Ok((
                    animation_identity(animation),
                    animation_index,
                    animation.clone(),
                ))
            }
        }
    }

    pub fn set_animation<'a, A>(
        &mut self,
        track_index: usize,
        animation: A,
        looped: bool,
    ) -> Result<TrackEntryHandle, Error>
    where
        A: Into<TrackAnimationInput<'a>>,
    {
        let (animation_id, animation_index, animation) =
            self.resolve_track_animation(animation.into())?;
        Ok(self.set_animation_internal(
            track_index,
            animation_index,
            animation_id,
            animation,
            looped,
        ))
    }

    pub fn set_empty_animation(
        &mut self,
        track_index: usize,
        mix_duration: f32,
    ) -> TrackEntryHandle {
        let entry = self.set_animation_internal(
            track_index,
            EMPTY_ANIMATION_INDEX,
            EMPTY_ANIMATION_ID,
            empty_animation(),
            false,
        );
        entry.set_mix_duration(self, mix_duration);
        entry.set_track_end(self, mix_duration);
        entry
    }

    pub fn set_empty_animations(&mut self, mix_duration: f32) {
        let current_track_indices = self
            .tracks
            .iter()
            .enumerate()
            .filter_map(|(track_index, track)| track.current.map(|_| track_index))
            .collect::<Vec<_>>();
        if current_track_indices.is_empty() {
            return;
        }

        let old_drain_disabled = self.drain_disabled;
        self.drain_disabled = true;
        for track_index in current_track_indices {
            self.set_empty_animation(track_index, mix_duration);
        }
        self.drain_disabled = old_drain_disabled;
        self.drain_event_queue();
    }

    fn set_animation_internal(
        &mut self,
        track_index: usize,
        animation_index: usize,
        entry_animation_id: u64,
        animation: Animation,
        looped: bool,
    ) -> TrackEntryHandle {
        self.ensure_track(track_index);

        let (old_current, queued_entries) = {
            let track = &mut self.tracks[track_index];
            let old_current = track.current.take();
            let queued_entries = track.queue.drain(..).collect::<Vec<_>>();
            (old_current, queued_entries)
        };
        if let Some(old_current) = old_current
            && let Some(entry) = self.entry_mut(old_current)
        {
            entry.next = None;
        }
        for queued in &queued_entries {
            if let Some(entry) = self.entry_mut(*queued) {
                entry.previous = None;
                entry.next = None;
            }
        }

        let entry_id = self.alloc_entry(TrackEntry::new(
            track_index,
            animation_index,
            entry_animation_id,
            &animation,
            looped,
        ));

        let mut previous_for_mix = old_current;
        let mut interrupt_previous = true;
        let mut dispose_old_immediately = false;
        if let Some(old) = old_current {
            let old_is_unapplied = self
                .entry(old)
                .is_some_and(|entry| entry.next_track_last_time < 0.0);
            let old_is_same_animation = self
                .entry(old)
                .is_some_and(|entry| entry.animation_identity == entry_animation_id);

            // Match spine-cpp:
            // - Only skip mixing from an unapplied entry when setting the same animation again.
            // - Otherwise, an entry is mixed from even if it was never applied yet.
            if old_is_unapplied && old_is_same_animation {
                dispose_old_immediately = true;
                previous_for_mix = self.entry(old).and_then(|entry| entry.mixing_from);
                interrupt_previous = false;
            }
        }

        if let Some(previous) = previous_for_mix {
            let previous_name = self
                .entry(previous)
                .map(|entry| entry.animation.name.as_str())
                .unwrap_or(EMPTY_ANIMATION_NAME);
            let mix_duration = self
                .data
                .mix_duration(previous_name, animation.name.as_str());

            if let Some(entry_ref) = self.entry_mut(entry_id) {
                entry_ref.mix_duration = mix_duration;
                entry_ref.mixing_from = Some(previous);
                entry_ref.mix_time = 0.0;
            }

            // Match spine-cpp: reset rotation mixing state when an entry becomes `mixingFrom`.
            if let Some(prev) = self.entry_mut(previous) {
                prev.mixing_to = Some(entry_id);
                prev.rotation_state.clear();
            }
        }
        self.tracks[track_index].current = Some(entry_id);

        // Preserve event ordering without borrowing `self` during track mutation.
        if let Some(old) = old_current {
            if dispose_old_immediately {
                push_event(&mut self.event_queue, old, AnimationStateEvent::Interrupt);
                push_event(&mut self.event_queue, old, AnimationStateEvent::End);
                push_event(&mut self.event_queue, old, AnimationStateEvent::Dispose);
                self.animations_changed = true;
            } else if interrupt_previous {
                push_event(&mut self.event_queue, old, AnimationStateEvent::Interrupt);
            }
        }
        for queued in queued_entries {
            push_event(&mut self.event_queue, queued, AnimationStateEvent::Dispose);
        }
        push_event(&mut self.event_queue, entry_id, AnimationStateEvent::Start);
        self.animations_changed = true;
        self.drain_event_queue();

        self.handle(entry_id)
    }

    pub fn add_animation<'a, A>(
        &mut self,
        track_index: usize,
        animation: A,
        looped: bool,
        delay: f32,
    ) -> Result<TrackEntryHandle, Error>
    where
        A: Into<TrackAnimationInput<'a>>,
    {
        let (animation_id, animation_index, animation) =
            self.resolve_track_animation(animation.into())?;
        self.ensure_track(track_index);
        let last = {
            let track = &self.tracks[track_index];
            track.queue.back().copied().or(track.current)
        };

        let entry_id = self.alloc_entry(TrackEntry::new(
            track_index,
            animation_index,
            animation_id,
            &animation,
            looped,
        ));

        let (resolved_delay, resolved_mix_duration) = if let Some(last) = last {
            let (last_track_complete, mix_duration) = self
                .entry(last)
                .map(|last_ref| {
                    (
                        last_ref.track_complete(),
                        self.data.mix_duration(
                            last_ref.animation.name.as_str(),
                            animation.name.as_str(),
                        ),
                    )
                })
                .unwrap_or((0.0, 0.0));
            let resolved_delay = if delay > 0.0 {
                delay
            } else {
                (delay + last_track_complete - mix_duration).max(0.0)
            };
            (resolved_delay, mix_duration)
        } else {
            (delay.max(0.0), 0.0)
        };

        if let Some(entry_ref) = self.entry_mut(entry_id) {
            entry_ref.delay = resolved_delay;
            entry_ref.mix_duration = resolved_mix_duration;
            entry_ref.previous = last;
        }
        if let Some(last) = last
            && let Some(last_entry) = self.entry_mut(last)
        {
            last_entry.next = Some(entry_id);
        }

        let track_empty = self.tracks[track_index].current.is_none();
        if track_empty {
            self.tracks[track_index].current = Some(entry_id);
            push_event(&mut self.event_queue, entry_id, AnimationStateEvent::Start);
            self.drain_event_queue();
        } else {
            self.tracks[track_index].queue.push_back(entry_id);
        }
        Ok(self.handle(entry_id))
    }

    pub fn add_empty_animation(
        &mut self,
        track_index: usize,
        mix_duration: f32,
        delay: f32,
    ) -> TrackEntryHandle {
        self.ensure_track(track_index);
        let last = {
            let track = &self.tracks[track_index];
            track.queue.back().copied().or(track.current)
        };

        let animation = empty_animation();
        let entry_id = self.alloc_entry(TrackEntry::new(
            track_index,
            EMPTY_ANIMATION_INDEX,
            EMPTY_ANIMATION_ID,
            &animation,
            false,
        ));

        let (mut resolved_delay, resolved_mix_duration) = if let Some(last) = last {
            let (last_track_complete, mix_duration_to_empty) = self
                .entry(last)
                .map(|last_ref| {
                    (
                        last_ref.track_complete(),
                        self.data
                            .mix_duration(last_ref.animation.name.as_str(), EMPTY_ANIMATION_NAME),
                    )
                })
                .unwrap_or((0.0, 0.0));
            let resolved_delay = if delay > 0.0 {
                delay
            } else {
                (delay + last_track_complete - mix_duration_to_empty).max(0.0)
            };
            (resolved_delay, mix_duration_to_empty)
        } else {
            (delay.max(0.0), 0.0)
        };

        // Match upstream runtimes: if delay <= 0, reduce the delay by the difference between the
        // previous->empty mix duration and the requested empty mix duration so the empty mix ends
        // at the same time the previous entry ends.
        if delay <= 0.0 {
            resolved_delay = (resolved_delay + resolved_mix_duration - mix_duration).max(0.0);
        }

        if let Some(entry_ref) = self.entry_mut(entry_id) {
            entry_ref.delay = resolved_delay;
            entry_ref.mix_duration = mix_duration;
            entry_ref.track_end = mix_duration;
            entry_ref.previous = last;
        }
        if let Some(last) = last
            && let Some(last_entry) = self.entry_mut(last)
        {
            last_entry.next = Some(entry_id);
        }

        let track_empty = self.tracks[track_index].current.is_none();
        if track_empty {
            self.tracks[track_index].current = Some(entry_id);
            push_event(&mut self.event_queue, entry_id, AnimationStateEvent::Start);
            self.drain_event_queue();
        } else {
            self.tracks[track_index].queue.push_back(entry_id);
        }
        self.handle(entry_id)
    }

    pub fn update(&mut self, delta: f32) {
        let delta = delta * self.time_scale;
        self.time.set(self.time.get() + delta);

        let mut pending = VecDeque::new();

        let tracks_len = self.tracks.len();
        for track_index in 0..tracks_len {
            let Some(current_id) = self.tracks[track_index].current else {
                continue;
            };

            let (current_delta, track_last, mixing_from, track_end) = {
                let Some(current) = self.entry_mut(current_id) else {
                    self.tracks[track_index].current = None;
                    continue;
                };

                current.animation_last_time = current.next_animation_last_time;
                current.track_last_time = current.next_track_last_time;

                let mut current_delta = delta * current.time_scale;
                if current.delay > 0.0 {
                    current.delay -= current_delta;
                    if current.delay > 0.0 {
                        continue;
                    }
                    current_delta = -current.delay;
                    current.delay = 0.0;
                }

                (
                    current_delta,
                    current.track_last_time,
                    current.mixing_from,
                    current.track_end,
                )
            };

            if let Some(next_id) = self.tracks[track_index].queue.front().copied() {
                let next_delay = self.entry(next_id).map(|next| next.delay).unwrap_or(0.0);
                let next_time = track_last - next_delay;
                if next_time >= 0.0 {
                    let old_time_scale =
                        self.entry(current_id).map(|e| e.time_scale).unwrap_or(0.0);
                    if let Some(current) = self.entry_mut(current_id) {
                        current.track_time += current_delta;
                    }

                    let next_id = self.tracks[track_index]
                        .queue
                        .pop_front()
                        .expect("queue front exists");
                    if let Some(next) = self.entry_mut(next_id) {
                        next.delay = 0.0;
                        next.previous = None;
                        // Preserve leftover time when switching (Spine C# Update semantics).
                        if old_time_scale != 0.0 {
                            next.track_time +=
                                (next_time / old_time_scale + delta) * next.time_scale;
                        }
                        next.mixing_from = Some(current_id);
                        next.mix_time = 0.0;
                        if next.mix_duration <= 0.0 {
                            next.mix_duration = delta;
                        }
                    }
                    if let Some(current) = self.entry_mut(current_id) {
                        current.mixing_to = Some(next_id);
                        current.rotation_state.clear();
                    }

                    // Match C# behavior: increment mixTime along the mixing chain.
                    let mut mix_id = next_id;
                    while let Some(from_id) = self.entry(mix_id).and_then(|e| e.mixing_from) {
                        if let Some(entry) = self.entry_mut(mix_id) {
                            entry.mix_time += delta;
                        }
                        mix_id = from_id;
                    }

                    push_event(&mut pending, current_id, AnimationStateEvent::Interrupt);
                    push_event(&mut pending, next_id, AnimationStateEvent::Start);
                    self.animations_changed = true;
                    self.tracks[track_index].current = Some(next_id);
                    continue;
                }
            } else if mixing_from.is_none() && track_last >= 0.0 && track_last >= track_end {
                push_event(&mut pending, current_id, AnimationStateEvent::End);
                push_event(&mut pending, current_id, AnimationStateEvent::Dispose);
                self.animations_changed = true;
                self.tracks[track_index].current = None;
                continue;
            }

            if mixing_from.is_some() {
                self.update_mixing_from(current_id, delta, &mut pending);
            }
            if let Some(current) = self.entry_mut(current_id) {
                current.track_time += current_delta;
            }
        }

        self.event_queue.append(&mut pending);
        self.drain_event_queue();
    }

    pub fn apply(&mut self, skeleton: &mut Skeleton) -> bool {
        if self.animations_changed {
            self.animations_changed();
        }

        let mut applied = false;
        let mut pending = VecDeque::new();

        let current_ids = self
            .tracks
            .iter()
            .filter_map(|track| track.current)
            .collect::<Vec<_>>();
        for current_id in current_ids {
            let (track_index, delay) = match self.entry(current_id) {
                Some(entry) => (entry.track_index, entry.delay),
                None => continue,
            };
            if delay > 0.0 {
                continue;
            }
            applied = true;

            let blend = if track_index == 0 {
                MixBlend::First
            } else {
                self.entry(current_id)
                    .map(|entry| {
                        if entry.mix_blend == MixBlend::Add {
                            MixBlend::Replace
                        } else {
                            entry.mix_blend
                        }
                    })
                    .unwrap_or(MixBlend::Replace)
            };

            let mut alpha = self.entry(current_id).map(|e| e.alpha).unwrap_or(1.0);

            if self.entry(current_id).and_then(|e| e.mixing_from).is_some() {
                alpha *= self.apply_mixing_from_pose(current_id, skeleton, blend, &mut pending);
            } else {
                let track_end_reached = {
                    let track = &self.tracks[track_index];
                    let queued_empty = track.queue.is_empty();
                    let reached = self
                        .entry(current_id)
                        .is_some_and(|e| e.track_time >= e.track_end);
                    queued_empty && reached
                };
                if track_end_reached {
                    alpha = 0.0;
                }
            }

            let (animation, time, looped, alpha_attachment_threshold, reverse) =
                match self.entry(current_id) {
                    Some(e) => (
                        e.animation.clone(),
                        e.animation_time(),
                        e.looped,
                        e.alpha_attachment_threshold,
                        e.reverse,
                    ),
                    None => continue,
                };

            let apply_time = if reverse {
                animation.duration - time
            } else {
                time
            };

            let mut attachments = alpha >= alpha_attachment_threshold;
            if track_index == 0 && alpha == 1.0 {
                attachments = true;
            }

            self.apply_entry_pose(
                current_id,
                &animation,
                skeleton,
                apply_time,
                looped,
                alpha,
                blend,
                attachments,
                MixDirection::In,
            );

            self.apply_entry_events_and_complete(current_id, None, true, &mut pending);
        }

        let setup_state = self.unkeyed_state + ANIMATION_STATE_SETUP;
        for (i, slot) in skeleton.slots.iter_mut().enumerate() {
            if slot.attachment_state == setup_state {
                slot.attachment = skeleton
                    .data
                    .slots
                    .get(i)
                    .and_then(|s| s.attachment.clone());
            }
        }
        self.unkeyed_state = self.unkeyed_state.wrapping_add(2);

        self.event_queue.append(&mut pending);
        self.drain_event_queue();
        applied
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_entry_pose(
        &mut self,
        entry_id: EntryId,
        animation: &Animation,
        skeleton: &mut Skeleton,
        time: f32,
        looped: bool,
        alpha: f32,
        blend: MixBlend,
        attachments: bool,
        direction: MixDirection,
    ) {
        let (entry_additive, shortest_rotation) = self
            .entry(entry_id)
            .map(|e| {
                let additive = e.mix_blend == MixBlend::Add;
                (additive, additive || e.shortest_rotation)
            })
            .unwrap_or((false, false));

        let mut time = time;
        if looped && animation.duration > 0.0 {
            time = time.rem_euclid(animation.duration);
        }

        let track_index = self.entry(entry_id).map(|e| e.track_index).unwrap_or(0);
        let special_case = track_index == 0 && alpha == 1.0 && direction == MixDirection::In;

        let kinds = animation_timeline_order(animation).to_vec();
        let mut timeline_mode = self
            .entry(entry_id)
            .map(|e| e.timeline_mode.clone())
            .unwrap_or_default();
        if timeline_mode.len() != kinds.len() {
            self.animations_changed = true;
            self.animations_changed();
            timeline_mode = self
                .entry(entry_id)
                .map(|e| e.timeline_mode.clone())
                .unwrap_or_default();
        }

        let first_frame = self
            .entry_mut(entry_id)
            .map(|entry| {
                let expected_len = kinds.len() * 2;
                let first = entry.rotation_state.len() != expected_len;
                if first {
                    entry.rotation_state.resize(expected_len, 0.0);
                }
                first
            })
            .unwrap_or(false);
        let unkeyed_state = self.unkeyed_state;
        let physics_last_time = self
            .entry(entry_id)
            .map(|e| e.animation_last_time)
            .unwrap_or(-1.0);

        for (i, kind) in kinds.into_iter().enumerate() {
            let mode = timeline_mode.get(i).copied().unwrap_or(TimelineApplyMode {
                from: TimelineMode::First,
                hold: false,
            });
            let timeline_blend = if special_case {
                blend
            } else {
                timeline_mode_blend(mode.from, blend)
            };
            let effective_from = if special_case {
                TimelineMode::Setup
            } else {
                mode.from
            };
            let effective_add = entry_additive && !special_case;
            let additive_blend = entry_additive_blend(timeline_blend, effective_add);
            let from = timeline_mode_from(effective_from);
            let uses_mixed_rotation = !shortest_rotation
                && !special_case
                && alpha < 1.0
                && additive_blend != MixBlend::Add
                && matches!(
                    kind,
                    TimelineKind::Bone(ti)
                        if matches!(&animation.bone_timelines[ti], crate::BoneTimeline::Rotate(_))
                );
            let rotate = if uses_mixed_rotation {
                let Some(entry) = self.entry_mut(entry_id) else {
                    continue;
                };
                Some(RotateTimelineState {
                    state: entry.rotation_state.as_mut_slice(),
                    index: i,
                    first_frame,
                })
            } else {
                None
            };

            apply_state_timeline(
                animation,
                kind,
                skeleton,
                StateTimelineApply {
                    time,
                    alpha,
                    timeline_blend,
                    additive_blend,
                    from,
                    additive: effective_add,
                    transform_additive: entry_additive,
                    direction,
                    attachments,
                    unkeyed_state,
                    draw_order_from_current: effective_from == TimelineMode::Current,
                    draw_order_out: Some(false),
                    draw_order_folder_out: false,
                    physics_last_time,
                    physics_time: time,
                    rotate,
                },
            );
        }
    }

    fn apply_mixing_from_pose(
        &mut self,
        to: EntryId,
        skeleton: &mut Skeleton,
        blend: MixBlend,
        out: &mut VecDeque<QueuedEvent>,
    ) -> f32 {
        let Some(from) = self.entry(to).and_then(|entry| entry.mixing_from) else {
            return 1.0;
        };

        let from_mix = if self
            .entry(from)
            .and_then(|entry| entry.mixing_from)
            .is_some()
        {
            self.apply_mixing_from_pose(from, skeleton, blend, out)
        } else {
            1.0
        };

        let (mix_duration, mix, to_alpha) = self
            .entry(to)
            .map(|to_ref| (to_ref.mix_duration, to_ref.mix_percent(), to_ref.alpha))
            .unwrap_or((0.0, 1.0, 1.0));

        let (
            from_animation,
            from_time,
            from_looped,
            from_reverse,
            from_additive,
            from_shortest_rotation,
            from_alpha,
            from_thresholds,
        ) = match self.entry(from) {
            Some(from_ref) => (
                from_ref.animation.clone(),
                from_ref.animation_time(),
                from_ref.looped,
                from_ref.reverse,
                from_ref.mix_blend == MixBlend::Add,
                from_ref.mix_blend == MixBlend::Add || from_ref.shortest_rotation,
                from_ref.alpha,
                (
                    from_ref.alpha_attachment_threshold,
                    from_ref.mix_attachment_threshold,
                    from_ref.mix_draw_order_threshold,
                ),
            ),
            None => return 1.0,
        };

        let from_blend = if mix_duration <= 0.0 && blend == MixBlend::First {
            MixBlend::Setup
        } else {
            blend
        };

        let a = from_alpha * from_mix;
        let keep = 1.0 - mix * to_alpha;
        let alpha_mix = a * (1.0 - mix);
        let alpha_hold = if keep > 0.0 { alpha_mix / keep } else { a };

        if let Some(from_entry) = self.entry_mut(from) {
            from_entry.total_alpha = 0.0;
        }

        let attachments = mix < from_thresholds.1;
        let draw_order = mix < from_thresholds.2;

        let from_apply_time = if from_reverse {
            from_animation.duration - from_time
        } else {
            from_time
        };

        {
            let kinds = animation_timeline_order(&from_animation).to_vec();
            let (timeline_mode, timeline_hold_mix) = match self.entry(from) {
                Some(e) => (e.timeline_mode.clone(), e.timeline_hold_mix.clone()),
                None => (Vec::new(), Vec::new()),
            };
            let alpha_attachment_threshold = from_thresholds.0;

            let first_frame = self
                .entry_mut(from)
                .map(|entry| {
                    let expected_len = kinds.len() * 2;
                    let first = entry.rotation_state.len() != expected_len;
                    if first {
                        entry.rotation_state.resize(expected_len, 0.0);
                    }
                    first
                })
                .unwrap_or(false);
            let unkeyed_state = self.unkeyed_state;

            let mut total_alpha = 0.0f32;
            for (i, kind) in kinds.into_iter().enumerate() {
                let mode = timeline_mode.get(i).copied().unwrap_or(TimelineApplyMode {
                    from: TimelineMode::First,
                    hold: false,
                });

                let timeline_blend = timeline_mode_blend(mode.from, from_blend);
                let alpha = if mode.hold {
                    let hold_mix = timeline_hold_mix.get(i).copied().flatten();
                    if let Some(hold_mix) = hold_mix {
                        let factor = self
                            .entry(hold_mix)
                            .map(|e| (1.0 - e.mix_percent()).max(0.0))
                            .unwrap_or(0.0);
                        alpha_hold * factor
                    } else {
                        alpha_hold
                    }
                } else {
                    alpha_mix
                };
                total_alpha += alpha;

                let additive_blend = entry_additive_blend(timeline_blend, from_additive);
                let from_mode = timeline_mode_from(mode.from);
                let mut physics_last_time = self
                    .entry(from)
                    .map(|e| e.animation_last_time)
                    .unwrap_or(-1.0);
                let mut physics_time = from_apply_time;
                if from_looped && from_animation.duration > 0.0 {
                    physics_time = physics_time.rem_euclid(from_animation.duration);
                    if physics_last_time >= 0.0 {
                        physics_last_time = physics_last_time.rem_euclid(from_animation.duration);
                    }
                }

                let uses_mixed_rotation = !from_shortest_rotation
                    && alpha < 1.0
                    && additive_blend != MixBlend::Add
                    && matches!(
                        kind,
                        TimelineKind::Bone(ti)
                            if matches!(&from_animation.bone_timelines[ti], crate::BoneTimeline::Rotate(_))
                    );
                let rotate = if uses_mixed_rotation {
                    let Some(entry) = self.entry_mut(from) else {
                        continue;
                    };
                    Some(RotateTimelineState {
                        state: entry.rotation_state.as_mut_slice(),
                        index: i,
                        first_frame,
                    })
                } else {
                    None
                };

                apply_state_timeline(
                    &from_animation,
                    kind,
                    skeleton,
                    StateTimelineApply {
                        time: from_apply_time,
                        alpha,
                        timeline_blend,
                        additive_blend,
                        from: from_mode,
                        additive: from_additive,
                        transform_additive: from_additive,
                        direction: MixDirection::Out,
                        attachments: attachments && alpha >= alpha_attachment_threshold,
                        unkeyed_state,
                        draw_order_from_current: mode.from == TimelineMode::Current,
                        draw_order_out: draw_order_timeline_out(draw_order, mode.from),
                        draw_order_folder_out: true,
                        physics_last_time,
                        physics_time,
                        rotate,
                    },
                );
            }
            if let Some(from_entry) = self.entry_mut(from) {
                from_entry.total_alpha = total_alpha;
            }
        }

        if mix_duration > 0.0 {
            self.apply_entry_events_and_complete(from, Some(mix), true, out);
        } else if let Some(from_ref) = self.entry_mut(from) {
            let animation_time = from_ref.animation_time();
            from_ref.next_animation_last_time = animation_time;
            from_ref.next_track_last_time = from_ref.track_time;
        }

        mix
    }

    pub fn clear_track(&mut self, track_index: usize) {
        self.clear_track_internal(track_index);
        self.drain_event_queue();
    }

    pub fn clear_tracks(&mut self) {
        let tracks_len = self.tracks.len();
        for i in 0..tracks_len {
            self.clear_track_internal(i);
        }
        self.tracks.clear();
        self.drain_event_queue();
    }

    fn ensure_track(&mut self, track_index: usize) {
        if track_index >= self.tracks.len() {
            self.tracks.resize_with(track_index + 1, Track::default);
        }
    }

    fn alloc_entry(&mut self, entry: TrackEntry) -> EntryId {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.entries[index];
            slot.entry = Some(entry);
            EntryId {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.entries.len();
            self.entries.push(EntrySlot {
                generation: 0,
                entry: Some(entry),
            });
            EntryId {
                index,
                generation: 0,
            }
        }
    }

    fn entry(&self, id: EntryId) -> Option<&TrackEntry> {
        let slot = self.entries.get(id.index)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.entry.as_ref()
    }

    fn entry_mut(&mut self, id: EntryId) -> Option<&mut TrackEntry> {
        let slot = self.entries.get_mut(id.index)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.entry.as_mut()
    }

    fn free_entry(&mut self, id: EntryId) {
        let Some(slot) = self.entries.get_mut(id.index) else {
            return;
        };
        if slot.generation != id.generation {
            return;
        }
        slot.entry = None;
        slot.generation = slot.generation.wrapping_add(1);
        self.free_list.push(id.index);
    }

    fn snapshot(&self, id: EntryId) -> TrackEntrySnapshot {
        if let Some(entry) = self.entry(id) {
            TrackEntrySnapshot {
                track_index: entry.track_index,
                animation_name: entry.animation.name.clone(),
                track_time: entry.track_time,
                animation_time: entry.animation_time(),
                looped: entry.looped,
                delay: entry.delay,
                mix_duration: entry.mix_duration,
                mix_time: entry.mix_time,
                alpha: entry.alpha,
                hold_previous: entry.hold_previous,
                mix_blend: entry.mix_blend,
                reverse: entry.reverse,
            }
        } else {
            TrackEntrySnapshot {
                track_index: 0,
                animation_name: "<disposed>".to_string(),
                track_time: 0.0,
                animation_time: 0.0,
                looped: false,
                delay: 0.0,
                mix_duration: 0.0,
                mix_time: 0.0,
                alpha: 0.0,
                hold_previous: false,
                mix_blend: MixBlend::Replace,
                reverse: false,
            }
        }
    }

    fn take_entry_listener(&mut self, id: EntryId) -> Option<Box<dyn TrackEntryListener>> {
        self.entry_mut(id).and_then(|entry| entry.listener.take())
    }

    fn restore_entry_listener(&mut self, id: EntryId, listener: Box<dyn TrackEntryListener>) {
        if let Some(entry) = self.entry_mut(id)
            && entry.listener.is_none()
        {
            entry.listener = Some(listener);
        }
    }

    fn update_mixing_from(
        &mut self,
        to: EntryId,
        delta: f32,
        out: &mut VecDeque<QueuedEvent>,
    ) -> bool {
        let Some(from) = self.entry(to).and_then(|entry| entry.mixing_from) else {
            return true;
        };

        let finished = self.update_mixing_from(from, delta, out);

        if let Some(from_entry) = self.entry_mut(from) {
            from_entry.animation_last_time = from_entry.next_animation_last_time;
            from_entry.track_last_time = from_entry.next_track_last_time;
        }

        let (to_next_track_last, to_mix_time, to_mix_duration) = self
            .entry(to)
            .map(|to_ref| {
                (
                    to_ref.next_track_last_time,
                    to_ref.mix_time,
                    to_ref.mix_duration,
                )
            })
            .unwrap_or((-1.0, 0.0, 0.0));

        // The to entry was applied at least once and the mix is complete.
        if to_next_track_last >= 0.0 && to_mix_time >= to_mix_duration {
            let from_total_alpha = self.entry(from).map(|e| e.total_alpha).unwrap_or(0.0);
            if to_mix_duration <= 0.0 || from_total_alpha.abs() <= TIME_EPSILON {
                let next_from = self.entry(from).and_then(|from_ref| from_ref.mixing_from);
                if let Some(to_entry) = self.entry_mut(to) {
                    to_entry.mixing_from = next_from;
                }
                if let Some(next_from) = next_from
                    && let Some(entry) = self.entry_mut(next_from)
                {
                    entry.mixing_to = Some(to);
                }
                if from_total_alpha.abs() <= TIME_EPSILON {
                    let mut keep_id = Some(to);
                    while let Some(entry_id) = keep_id {
                        let Some(next_id) = self.entry(entry_id).and_then(|entry| entry.mixing_to)
                        else {
                            break;
                        };
                        if let Some(entry) = self.entry_mut(entry_id) {
                            entry.keep_hold = true;
                        }
                        keep_id = Some(next_id);
                    }
                }
                if let Some(from_entry) = self.entry_mut(from) {
                    from_entry.mixing_to = None;
                    from_entry.mixing_from = None;
                }
                push_event(out, from, AnimationStateEvent::End);
                push_event(out, from, AnimationStateEvent::Dispose);
                self.animations_changed = true;
                return finished && self.entry(to).and_then(|entry| entry.mixing_from).is_none();
            }
            return false;
        }

        // mixTime is not affected by entry time scale, following Spine semantics.
        if let Some(from_entry) = self.entry_mut(from) {
            from_entry.track_time += delta * from_entry.time_scale;
        }
        if let Some(to_entry) = self.entry_mut(to) {
            to_entry.mix_time += delta;
        }

        false
    }

    fn apply_entry_events_and_complete(
        &mut self,
        entry_id: EntryId,
        mix: Option<f32>,
        events_enabled: bool,
        out: &mut VecDeque<QueuedEvent>,
    ) {
        let Some(entry) = self.entry(entry_id) else {
            return;
        };

        let animation_start = entry.animation_start;
        let animation_end = entry.animation_end;
        let duration = animation_end - animation_start;

        let animation_time = entry.animation_time();
        let animation_last = entry.animation_last_time;
        let track_last = entry.track_last_time;
        let track_time = entry.track_time;
        let reverse = entry.reverse;
        let animation_duration = entry.animation.duration;

        let can_issue_events = match mix {
            None => true,
            Some(mix) => mix < entry.event_threshold,
        };

        let mut events = Vec::new();
        if events_enabled
            && can_issue_events
            && let Some(timeline) = &entry.animation.event_timeline
        {
            if reverse {
                collect_reverse_events(
                    timeline,
                    animation_last,
                    animation_time,
                    animation_duration,
                    &mut events,
                );
            } else {
                collect_events(
                    timeline,
                    animation_last,
                    animation_time,
                    entry.looped,
                    animation_start,
                    animation_end,
                    &mut events,
                );
            }
        }

        let complete = if entry.looped {
            if duration.abs() <= TIME_EPSILON {
                true
            } else {
                let cycles = (track_time / duration) as i32;
                cycles > 0 && cycles > (track_last / duration) as i32
            }
        } else {
            animation_time + TIME_EPSILON >= animation_end
                && animation_last + TIME_EPSILON < animation_end
        };

        // Queue events before complete, then complete, then events after complete (Spine semantics).
        if complete && duration.abs() > TIME_EPSILON && !events.is_empty() {
            let mut track_last_wrapped = track_last % duration;
            if track_last_wrapped < 0.0 {
                track_last_wrapped += duration;
            }
            let mut split = events.len();
            for (i, ev) in events.iter().enumerate() {
                let local_time = ev.time - animation_start;
                if local_time + TIME_EPSILON < track_last_wrapped {
                    split = i;
                    break;
                }
            }
            for ev in &events[..split] {
                push_event(out, entry_id, AnimationStateEvent::Event(ev.clone()));
            }
            push_event(out, entry_id, AnimationStateEvent::Complete);
            for ev in &events[split..] {
                push_event(out, entry_id, AnimationStateEvent::Event(ev.clone()));
            }
        } else {
            for ev in &events {
                push_event(out, entry_id, AnimationStateEvent::Event(ev.clone()));
            }
            if complete {
                push_event(out, entry_id, AnimationStateEvent::Complete);
            }
        }

        if let Some(entry) = self.entry_mut(entry_id) {
            entry.next_animation_last_time = animation_time;
            entry.next_track_last_time = track_time;
        }
    }

    fn clear_track_internal(&mut self, track_index: usize) {
        if track_index >= self.tracks.len() {
            return;
        }
        let (current, queued) = {
            let track = &mut self.tracks[track_index];
            let current = track.current.take();
            let queued = track.queue.drain(..).collect::<Vec<_>>();
            (current, queued)
        };
        if let Some(entry_id) = current {
            let mut from = self.entry_mut(entry_id).and_then(|entry| {
                let from = entry.mixing_from;
                entry.mixing_from = None;
                entry.mixing_to = None;
                entry.next = None;
                from
            });
            push_event(&mut self.event_queue, entry_id, AnimationStateEvent::End);
            push_event(
                &mut self.event_queue,
                entry_id,
                AnimationStateEvent::Dispose,
            );
            self.animations_changed = true;
            while let Some(mixing_from) = from {
                from = self.entry_mut(mixing_from).and_then(|entry| {
                    let from = entry.mixing_from;
                    entry.mixing_from = None;
                    entry.mixing_to = None;
                    entry.next = None;
                    from
                });
                push_event(&mut self.event_queue, mixing_from, AnimationStateEvent::End);
                push_event(
                    &mut self.event_queue,
                    mixing_from,
                    AnimationStateEvent::Dispose,
                );
            }
        }
        for entry in queued {
            if let Some(queued_entry) = self.entry_mut(entry) {
                queued_entry.previous = None;
                queued_entry.next = None;
            }
            push_event(&mut self.event_queue, entry, AnimationStateEvent::Dispose);
        }
    }

    fn drain_event_queue(&mut self) {
        if self.draining_events || self.drain_disabled {
            return;
        }
        self.draining_events = true;

        while let Some(queued) = self.event_queue.pop_front() {
            let entry_id = queued.entry;
            let event = queued.event;

            let snapshot = self.snapshot(entry_id);

            let mut entry_listener = self.take_entry_listener(entry_id);
            if let Some(listener) = entry_listener.as_mut() {
                listener.on_event(self, &snapshot, &event);
            }

            let mut state_listener = self.listener.take();
            if let Some(listener) = state_listener.as_mut() {
                listener.on_event(self, &snapshot, &event);
            }
            if self.listener.is_none() {
                self.listener = state_listener;
            }

            if matches!(event, AnimationStateEvent::Dispose) {
                if self.manual_track_entry_disposal {
                    if let Some(listener) = entry_listener {
                        self.restore_entry_listener(entry_id, listener);
                    }
                } else {
                    self.free_entry(entry_id);
                }
            } else if let Some(listener) = entry_listener {
                self.restore_entry_listener(entry_id, listener);
            }
        }

        self.draining_events = false;
    }

    #[cfg(all(test, feature = "json"))]
    pub(crate) fn round_tracks_for_tests(&mut self) {
        fn round_decimals(value: f32, decimals: u32) -> f32 {
            let factor = 10_f32.powi(decimals as i32);
            (value * factor).round() / factor
        }

        let current_ids = self
            .tracks
            .iter()
            .filter_map(|track| track.current)
            .collect::<Vec<_>>();
        for current_id in current_ids {
            if let Some(current) = self.entry_mut(current_id) {
                current.track_time = round_decimals(current.track_time, 6);
                current.delay = round_decimals(current.delay, 3);
            }
            let mut from = self.entry(current_id).and_then(|entry| entry.mixing_from);
            while let Some(id) = from {
                if let Some(entry) = self.entry_mut(id) {
                    entry.track_time = round_decimals(entry.track_time, 6);
                    from = entry.mixing_from;
                } else {
                    break;
                }
            }
        }
    }

    #[cfg(all(test, feature = "json"))]
    pub(crate) fn queue_front_delay_for_tests(&self, track_index: usize) -> Option<f32> {
        let track = self.tracks.get(track_index)?;
        let id = *track.queue.front()?;
        self.entry(id).map(|e| e.delay)
    }

    #[cfg(all(test, feature = "json"))]
    pub(crate) fn track_entry_exists_for_tests(&self, handle: TrackEntryHandle) -> bool {
        self.entry_for_handle(handle).is_some()
    }
}

fn push_event(out: &mut VecDeque<QueuedEvent>, entry: EntryId, event: AnimationStateEvent) {
    out.push_back(QueuedEvent { entry, event });
}

fn collect_events(
    timeline: &crate::EventTimeline,
    last_time: f32,
    time: f32,
    looped: bool,
    animation_start: f32,
    animation_end: f32,
    out: &mut Vec<Event>,
) {
    if timeline.events.is_empty() {
        return;
    }

    // Mirror upstream EventTimeline semantics: when looping (time wraps), the second segment only
    // runs when `time` reaches the first event frame time. This prevents duplicate events when
    // modulo arithmetic produces a `time` slightly below the first frame time.
    let first_time_in_range = timeline
        .events
        .iter()
        .find(|ev| {
            ev.time + TIME_EPSILON >= animation_start && ev.time <= animation_end + TIME_EPSILON
        })
        .map(|ev| ev.time);
    if first_time_in_range.is_none() {
        return;
    }
    let first_time_in_range = first_time_in_range.unwrap();

    let mut emit_range = |from: f32, to: f32| {
        let from = from.max(animation_start - TIME_EPSILON);
        let to = to.min(animation_end);
        if to + TIME_EPSILON < animation_start {
            return;
        }
        if from - TIME_EPSILON > animation_end {
            return;
        }
        for ev in &timeline.events {
            if ev.time + TIME_EPSILON < animation_start || ev.time > animation_end + TIME_EPSILON {
                continue;
            }
            // Match upstream: events fire for frames > lastTime and <= time (no epsilon on the
            // `time` comparison, otherwise near-boundary modulo arithmetic can re-fire events).
            if ev.time > from && ev.time <= to {
                out.push(ev.clone());
            }
        }
    };

    if last_time < 0.0 {
        emit_range(-1.0, time);
        return;
    }

    if looped
        && (animation_end - animation_start).abs() > TIME_EPSILON
        && time + TIME_EPSILON < last_time
    {
        emit_range(last_time, animation_end);
        if time >= first_time_in_range {
            emit_range(-1.0, time);
        }
    } else {
        emit_range(last_time, time);
    }
}

fn collect_reverse_events(
    timeline: &crate::EventTimeline,
    last_time: f32,
    time: f32,
    animation_duration: f32,
    out: &mut Vec<Event>,
) {
    if timeline.events.is_empty() {
        return;
    }

    let from = animation_duration - last_time;
    let to = animation_duration - time;

    if from >= to {
        for ev in &timeline.events {
            if ev.time < to {
                continue;
            }
            if ev.time >= from {
                break;
            }
            out.push(ev.clone());
        }
    } else {
        for ev in &timeline.events {
            if ev.time >= from {
                break;
            }
            out.push(ev.clone());
        }
        for ev in &timeline.events {
            if ev.time >= to {
                break;
            }
            out.push(ev.clone());
        }
    }
}

#[cfg(test)]
pub(super) fn collect_events_for_tests(
    timeline: &crate::EventTimeline,
    last_time: f32,
    time: f32,
    looped: bool,
    animation_start: f32,
    animation_end: f32,
) -> Vec<Event> {
    let mut out = Vec::new();
    collect_events(
        timeline,
        last_time,
        time,
        looped,
        animation_start,
        animation_end,
        &mut out,
    );
    out
}
