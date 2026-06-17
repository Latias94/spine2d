use super::animation::ANIMATION_STATE_SETUP;
use crate::{
    Animation, Error, Event, MixBlend, MixDirection, Skeleton, SkeletonData, apply_attachment,
    apply_deform, apply_draw_order, apply_ik_constraint_timeline, apply_inherit,
    apply_path_constraint_timeline, apply_physics_constraint_timeline,
    apply_physics_reset_timeline, apply_rotate, apply_rotate_mixed, apply_scale, apply_scale_x,
    apply_scale_y, apply_sequence_timeline, apply_shear, apply_shear_x, apply_shear_y,
    apply_slider_mix_timeline, apply_slider_time_timeline, apply_slot_alpha, apply_slot_color,
    apply_slot_rgb, apply_slot_rgb2, apply_slot_rgba2, apply_transform_constraint_timeline,
    apply_translate, apply_translate_x, apply_translate_y,
};
use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

const TIME_EPSILON: f32 = 1e-6;
const EMPTY_ANIMATION_INDEX: usize = usize::MAX;
const EMPTY_ANIMATION_NAME: &str = "<empty>";

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
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimelineMode {
    First,
    Subsequent,
    HoldFirst,
    HoldSubsequent,
    HoldMix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimelineKind {
    SlotAttachment(usize),
    Deform(usize),
    Sequence(usize),
    Bone(usize),
    SlotColor(usize),
    SlotRgb(usize),
    SlotAlpha(usize),
    SlotRgba2(usize),
    SlotRgb2(usize),
    IkConstraint(usize),
    TransformConstraint(usize),
    PathConstraint(usize),
    PhysicsConstraint(usize),
    SliderTime(usize),
    SliderMix(usize),
    DrawOrder,
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

fn timeline_kinds(animation: &Animation) -> Vec<TimelineKind> {
    let mut out = Vec::new();
    out.extend((0..animation.slot_attachment_timelines.len()).map(TimelineKind::SlotAttachment));
    out.extend((0..animation.deform_timelines.len()).map(TimelineKind::Deform));
    out.extend((0..animation.sequence_timelines.len()).map(TimelineKind::Sequence));
    out.extend((0..animation.bone_timelines.len()).map(TimelineKind::Bone));
    out.extend((0..animation.slot_color_timelines.len()).map(TimelineKind::SlotColor));
    out.extend((0..animation.slot_rgb_timelines.len()).map(TimelineKind::SlotRgb));
    out.extend((0..animation.slot_alpha_timelines.len()).map(TimelineKind::SlotAlpha));
    out.extend((0..animation.slot_rgba2_timelines.len()).map(TimelineKind::SlotRgba2));
    out.extend((0..animation.slot_rgb2_timelines.len()).map(TimelineKind::SlotRgb2));
    out.extend((0..animation.ik_constraint_timelines.len()).map(TimelineKind::IkConstraint));
    out.extend(
        (0..animation.transform_constraint_timelines.len()).map(TimelineKind::TransformConstraint),
    );
    out.extend((0..animation.path_constraint_timelines.len()).map(TimelineKind::PathConstraint));
    out.extend(
        (0..animation.physics_constraint_timelines.len()).map(TimelineKind::PhysicsConstraint),
    );
    out.extend((0..animation.slider_time_timelines.len()).map(TimelineKind::SliderTime));
    out.extend((0..animation.slider_mix_timelines.len()).map(TimelineKind::SliderMix));
    if animation.draw_order_timeline.is_some() {
        out.push(TimelineKind::DrawOrder);
    }
    out
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
    }
}

fn animation_has_any_property(data: &SkeletonData, animation: &Animation, ids: &[u64]) -> bool {
    if ids.is_empty() {
        return false;
    }
    let want: HashSet<u64> = ids.iter().copied().collect();
    for kind in timeline_kinds(animation) {
        let props = timeline_property_ids(data, animation, kind);
        if props.iter().any(|p| want.contains(p)) {
            return true;
        }
    }
    false
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
    pub skeleton_data: Arc<SkeletonData>,
    pub default_mix: f32,
    mixes: HashMap<(usize, usize), f32>,
}

impl AnimationStateData {
    pub fn new(skeleton_data: Arc<SkeletonData>) -> Self {
        Self {
            skeleton_data,
            default_mix: 0.0,
            mixes: HashMap::new(),
        }
    }

    pub fn set_mix(&mut self, from: &str, to: &str, duration: f32) -> Result<(), Error> {
        if duration.is_nan() || duration < 0.0 {
            return Err(Error::InvalidValue {
                message: "mix duration must be finite and >= 0".to_string(),
            });
        }
        let Some((from_index, _)) = self.skeleton_data.animation(from) else {
            return Err(Error::UnknownAnimation {
                name: from.to_string(),
            });
        };
        let Some((to_index, _)) = self.skeleton_data.animation(to) else {
            return Err(Error::UnknownAnimation {
                name: to.to_string(),
            });
        };
        self.mixes.insert((from_index, to_index), duration);
        Ok(())
    }

    fn mix_duration(&self, from_index: usize, to_index: usize) -> f32 {
        self.mixes
            .get(&(from_index, to_index))
            .copied()
            .unwrap_or(self.default_mix)
    }
}

pub struct TrackEntry {
    pub track_index: usize,
    pub animation_index: usize,
    pub animation: Animation,
    pub looped: bool,
    pub reverse: bool,
    pub shortest_rotation: bool,

    pub animation_start: f32,
    pub animation_end: f32,
    pub mix_duration: f32,
    pub mix_time: f32,
    mixing_from: Option<EntryId>,
    pub delay: f32,
    pub track_time: f32,
    pub track_end: f32,
    pub time_scale: f32,

    pub animation_last_time: f32,
    pub track_last_time: f32,
    pub next_animation_last_time: f32,
    pub next_track_last_time: f32,

    pub completed: bool,
    pub complete_pending: bool,
    pub ended: bool,

    pub alpha: f32,
    pub interrupt_alpha: f32,
    pub total_alpha: f32,
    mixing_to: Option<EntryId>,
    pub mix_blend: MixBlend,
    pub hold_previous: bool,
    pub alpha_attachment_threshold: f32,
    pub mix_attachment_threshold: f32,
    pub mix_draw_order_threshold: f32,
    pub event_threshold: f32,

    listener: Option<Box<dyn TrackEntryListener>>,

    timeline_mode: Vec<TimelineMode>,
    timeline_hold_mix: Vec<Option<EntryId>>,
    rotation_state: Vec<f32>,
}

impl std::fmt::Debug for TrackEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackEntry")
            .field("track_index", &self.track_index)
            .field("animation_index", &self.animation_index)
            .field("animation", &self.animation)
            .field("looped", &self.looped)
            .field("animation_start", &self.animation_start)
            .field("animation_end", &self.animation_end)
            .field("mix_duration", &self.mix_duration)
            .field("mix_time", &self.mix_time)
            .field("mixing_from", &self.mixing_from)
            .field("delay", &self.delay)
            .field("track_time", &self.track_time)
            .field("track_end", &self.track_end)
            .field("time_scale", &self.time_scale)
            .field("animation_last_time", &self.animation_last_time)
            .field("track_last_time", &self.track_last_time)
            .field("completed", &self.completed)
            .field("complete_pending", &self.complete_pending)
            .field("ended", &self.ended)
            .field("event_threshold", &self.event_threshold)
            .finish()
    }
}

impl TrackEntry {
    fn new(
        track_index: usize,
        animation_index: usize,
        animation: &Animation,
        looped: bool,
    ) -> Self {
        let track_end = f32::INFINITY;
        Self {
            track_index,
            animation_index,
            animation: animation.clone(),
            looped,
            reverse: false,
            shortest_rotation: false,
            animation_start: 0.0,
            animation_end: animation.duration,
            mix_duration: 0.0,
            mix_time: 0.0,
            mixing_from: None,
            delay: 0.0,
            track_time: 0.0,
            track_end,
            time_scale: 1.0,
            animation_last_time: -1.0,
            track_last_time: -1.0,
            next_animation_last_time: -1.0,
            next_track_last_time: -1.0,
            completed: false,
            complete_pending: false,
            ended: false,
            alpha: 1.0,
            interrupt_alpha: 1.0,
            total_alpha: 0.0,
            mixing_to: None,
            mix_blend: MixBlend::Replace,
            hold_previous: false,
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

    fn animation_time(&self) -> f32 {
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

    fn track_complete(&self) -> f32 {
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackEntryHandle {
    id: EntryId,
}

impl TrackEntryHandle {
    fn with_entry_mut(&self, state: &mut AnimationState, f: impl FnOnce(&mut TrackEntry)) {
        if let Some(entry) = state.entry_mut(self.id) {
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

    pub fn set_delay(&self, state: &mut AnimationState, delay: f32) {
        self.with_entry_mut(state, |entry| {
            entry.delay = delay;
        });
    }

    pub fn set_time_scale(&self, state: &mut AnimationState, time_scale: f32) {
        self.with_entry_mut(state, |entry| {
            entry.time_scale = time_scale;
        });
    }

    pub fn set_mix_duration(&self, state: &mut AnimationState, mix_duration: f32) {
        self.with_entry_mut(state, |entry| {
            entry.mix_duration = mix_duration;
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

#[derive(Clone, Debug)]
pub struct TrackEntrySnapshot {
    pub track_index: usize,
    pub animation_index: i32,
    pub animation_name: String,
    pub track_time: f32,
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
    data: AnimationStateData,
    tracks: Vec<Track>,
    entries: Vec<EntrySlot>,
    free_list: Vec<usize>,
    event_queue: VecDeque<QueuedEvent>,
    time: Cell<f32>,
    listener: Option<Box<dyn AnimationStateListener>>,
    draining_events: bool,
    animations_changed: bool,
    property_ids: HashSet<u64>,
    unkeyed_state: i32,
}

impl AnimationState {
    pub fn new(data: AnimationStateData) -> Self {
        Self {
            data,
            tracks: Vec::new(),
            entries: Vec::new(),
            free_list: Vec::new(),
            event_queue: VecDeque::new(),
            time: Cell::new(0.0),
            listener: None,
            draining_events: false,
            animations_changed: false,
            property_ids: HashSet::new(),
            unkeyed_state: 0,
        }
    }

    pub fn set_listener<L: AnimationStateListener + 'static>(&mut self, listener: L) {
        self.listener = Some(Box::new(listener));
    }

    pub fn time(&self) -> f32 {
        self.time.get()
    }

    pub fn data_mut(&mut self) -> &mut AnimationStateData {
        &mut self.data
    }

    pub fn tracks_len(&self) -> usize {
        self.tracks.len()
    }

    fn add_all_property_ids(&mut self, ids: &[u64]) -> bool {
        let mut all_new = true;
        for id in ids {
            if !self.property_ids.insert(*id) {
                all_new = false;
            }
        }
        all_new
    }

    fn animations_changed(&mut self) {
        self.animations_changed = false;
        self.property_ids.clear();

        let current_ids = self
            .tracks
            .iter()
            .filter_map(|t| t.current)
            .collect::<Vec<_>>();
        for mut entry_id in current_ids {
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
                let should_compute = self
                    .entry(id)
                    .is_some_and(|e| e.mixing_to.is_none() || e.mix_blend != MixBlend::Add);
                if should_compute {
                    self.compute_hold(id);
                }
            }
        }
    }

    fn compute_hold(&mut self, entry_id: EntryId) {
        let (animation, to_id, to_hold_previous) = match self.entry(entry_id) {
            Some(entry) => (
                entry.animation.clone(),
                entry.mixing_to,
                entry
                    .mixing_to
                    .and_then(|to| self.entry(to))
                    .map(|to| to.hold_previous)
                    .unwrap_or(false),
            ),
            None => return,
        };

        let kinds = timeline_kinds(&animation);
        let mut timeline_mode = vec![TimelineMode::First; kinds.len()];
        let mut timeline_hold_mix = vec![None; kinds.len()];

        if to_id.is_some() && to_hold_previous {
            for (i, kind) in kinds.iter().copied().enumerate() {
                let ids = timeline_property_ids(&self.data.skeleton_data, &animation, kind);
                let is_first = self.add_all_property_ids(&ids);
                timeline_mode[i] = if is_first {
                    TimelineMode::HoldFirst
                } else {
                    TimelineMode::HoldSubsequent
                };
            }
            if let Some(entry) = self.entry_mut(entry_id) {
                entry.timeline_mode = timeline_mode;
                entry.timeline_hold_mix = timeline_hold_mix;
            }
            return;
        }

        for (i, kind) in kinds.iter().copied().enumerate() {
            let ids = timeline_property_ids(&self.data.skeleton_data, &animation, kind);
            if !self.add_all_property_ids(&ids) {
                timeline_mode[i] = TimelineMode::Subsequent;
                continue;
            }

            let Some(to_id) = to_id else {
                timeline_mode[i] = TimelineMode::First;
                continue;
            };

            let is_special = matches!(
                kind,
                TimelineKind::SlotAttachment(_) | TimelineKind::DrawOrder
            );
            let to_anim = match self.entry(to_id) {
                Some(to) => &to.animation,
                None => {
                    timeline_mode[i] = TimelineMode::First;
                    continue;
                }
            };

            if is_special || !animation_has_any_property(&self.data.skeleton_data, to_anim, &ids) {
                timeline_mode[i] = TimelineMode::First;
                continue;
            }

            let mut next = self.entry(to_id).and_then(|e| e.mixing_to);
            let mut hold_mix = None;
            while let Some(next_id) = next {
                let Some(next_entry) = self.entry(next_id) else {
                    break;
                };
                if animation_has_any_property(&self.data.skeleton_data, &next_entry.animation, &ids)
                {
                    next = next_entry.mixing_to;
                    continue;
                }
                if next_entry.mix_duration > 0.0 {
                    hold_mix = Some(next_id);
                }
                break;
            }

            if let Some(hold_mix) = hold_mix {
                timeline_mode[i] = TimelineMode::HoldMix;
                timeline_hold_mix[i] = Some(hold_mix);
            } else {
                timeline_mode[i] = TimelineMode::HoldFirst;
            }
        }

        if let Some(entry) = self.entry_mut(entry_id) {
            entry.timeline_mode = timeline_mode;
            entry.timeline_hold_mix = timeline_hold_mix;
        }
    }

    pub fn with_track_entry<F: FnOnce(&TrackEntry) -> R, R>(
        &self,
        track_index: usize,
        f: F,
    ) -> Option<R> {
        let id = *self.tracks.get(track_index)?.current.as_ref()?;
        let entry = self.entry(id)?;
        Some(f(entry))
    }

    pub fn set_animation(
        &mut self,
        track_index: usize,
        animation_name: &str,
        looped: bool,
    ) -> Result<TrackEntryHandle, Error> {
        let skeleton_data = self.data.skeleton_data.clone();
        let (animation_index, animation) =
            skeleton_data
                .animation(animation_name)
                .ok_or_else(|| Error::UnknownAnimation {
                    name: animation_name.to_string(),
                })?;
        self.set_animation_internal(track_index, animation_index, animation.clone(), looped)
    }

    pub fn set_empty_animation(
        &mut self,
        track_index: usize,
        mix_duration: f32,
    ) -> Result<TrackEntryHandle, Error> {
        if !mix_duration.is_finite() || mix_duration < 0.0 {
            return Err(Error::InvalidValue {
                message: "mix duration must be finite and >= 0".to_string(),
            });
        }
        let entry = self.set_animation_internal(
            track_index,
            EMPTY_ANIMATION_INDEX,
            empty_animation(),
            false,
        )?;
        entry.set_mix_duration(self, mix_duration);
        entry.set_track_end(self, mix_duration);
        Ok(entry)
    }

    fn set_animation_internal(
        &mut self,
        track_index: usize,
        animation_index: usize,
        animation: Animation,
        looped: bool,
    ) -> Result<TrackEntryHandle, Error> {
        self.ensure_track(track_index);

        let (old_current, queued_entries) = {
            let track = &mut self.tracks[track_index];
            let old_current = track.current.take();
            let queued_entries = track.queue.drain(..).collect::<Vec<_>>();
            (old_current, queued_entries)
        };

        let entry_id = self.alloc_entry(TrackEntry::new(
            track_index,
            animation_index,
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
                .is_some_and(|entry| entry.animation_index == animation_index);

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
            let previous_index = self
                .entry(previous)
                .map(|entry| entry.animation_index)
                .unwrap_or(EMPTY_ANIMATION_INDEX);
            let mix_duration = self.data.mix_duration(previous_index, animation_index);
            let interrupt_alpha_mul = self
                .entry(previous)
                .and_then(|prev| {
                    if prev.mixing_from.is_some() && prev.mix_duration > 0.0 {
                        Some((prev.mix_time / prev.mix_duration).clamp(0.0, 1.0))
                    } else {
                        None
                    }
                })
                .unwrap_or(1.0);

            if let Some(entry_ref) = self.entry_mut(entry_id) {
                entry_ref.mix_duration = mix_duration;
                entry_ref.mixing_from = Some(previous);
                entry_ref.mix_time = 0.0;
                entry_ref.interrupt_alpha *= interrupt_alpha_mul;
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

        Ok(TrackEntryHandle { id: entry_id })
    }

    pub fn add_animation(
        &mut self,
        track_index: usize,
        animation_name: &str,
        looped: bool,
        delay: f32,
    ) -> Result<TrackEntryHandle, Error> {
        let skeleton_data = self.data.skeleton_data.clone();
        let (animation_index, animation) =
            skeleton_data
                .animation(animation_name)
                .ok_or_else(|| Error::UnknownAnimation {
                    name: animation_name.to_string(),
                })?;
        let animation = animation.clone();
        self.ensure_track(track_index);
        let last = {
            let track = &self.tracks[track_index];
            track.queue.back().copied().or(track.current)
        };

        let entry_id = self.alloc_entry(TrackEntry::new(
            track_index,
            animation_index,
            &animation,
            looped,
        ));

        let (resolved_delay, resolved_mix_duration) = if let Some(last) = last {
            let (last_track_complete, mix_duration) = self
                .entry(last)
                .map(|last_ref| {
                    (
                        last_ref.track_complete(),
                        self.data
                            .mix_duration(last_ref.animation_index, animation_index),
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
        }

        let track_empty = self.tracks[track_index].current.is_none();
        if track_empty {
            self.tracks[track_index].current = Some(entry_id);
            push_event(&mut self.event_queue, entry_id, AnimationStateEvent::Start);
            self.drain_event_queue();
        } else {
            self.tracks[track_index].queue.push_back(entry_id);
        }
        Ok(TrackEntryHandle { id: entry_id })
    }

    pub fn add_empty_animation(
        &mut self,
        track_index: usize,
        mix_duration: f32,
        delay: f32,
    ) -> Result<TrackEntryHandle, Error> {
        if !mix_duration.is_finite() || mix_duration < 0.0 {
            return Err(Error::InvalidValue {
                message: "mix duration must be finite and >= 0".to_string(),
            });
        }
        if !delay.is_finite() {
            return Err(Error::InvalidValue {
                message: "delay must be finite".to_string(),
            });
        }

        self.ensure_track(track_index);
        let last = {
            let track = &self.tracks[track_index];
            track.queue.back().copied().or(track.current)
        };

        let animation = empty_animation();
        let entry_id = self.alloc_entry(TrackEntry::new(
            track_index,
            EMPTY_ANIMATION_INDEX,
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
                            .mix_duration(last_ref.animation_index, EMPTY_ANIMATION_INDEX),
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
        }

        let track_empty = self.tracks[track_index].current.is_none();
        if track_empty {
            self.tracks[track_index].current = Some(entry_id);
            push_event(&mut self.event_queue, entry_id, AnimationStateEvent::Start);
            self.drain_event_queue();
        } else {
            self.tracks[track_index].queue.push_back(entry_id);
        }
        Ok(TrackEntryHandle { id: entry_id })
    }

    pub fn update(&mut self, delta: f32) {
        if !(delta.is_finite()) || delta < 0.0 {
            return;
        }
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
                if next_time + TIME_EPSILON >= 0.0 {
                    let old_time_scale =
                        self.entry(current_id).map(|e| e.time_scale).unwrap_or(0.0);
                    let interrupt_alpha_mul = self
                        .entry(current_id)
                        .and_then(|current| {
                            if current.mixing_from.is_some() && current.mix_duration > 0.0 {
                                Some((current.mix_time / current.mix_duration).clamp(0.0, 1.0))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(1.0);
                    if let Some(current) = self.entry_mut(current_id) {
                        current.track_time += current_delta;
                    }

                    let next_id = self.tracks[track_index]
                        .queue
                        .pop_front()
                        .expect("queue front exists");
                    if let Some(next) = self.entry_mut(next_id) {
                        next.delay = 0.0;
                        // Preserve leftover time when switching (Spine C# Update semantics).
                        if old_time_scale.abs() >= TIME_EPSILON {
                            next.track_time +=
                                (next_time / old_time_scale + delta) * next.time_scale;
                        }
                        next.mixing_from = Some(current_id);
                        next.interrupt_alpha *= interrupt_alpha_mul;
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
            } else if mixing_from.is_none()
                && track_last >= 0.0
                && track_last + TIME_EPSILON >= track_end
            {
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

    pub fn apply(&mut self, skeleton: &mut Skeleton) {
        if self.animations_changed {
            self.animations_changed();
        }

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

            let blend = if track_index == 0 {
                MixBlend::First
            } else {
                self.entry(current_id)
                    .map(|e| e.mix_blend)
                    .unwrap_or(MixBlend::Replace)
            };

            let mut alpha = self.entry(current_id).map(|e| e.alpha).unwrap_or(1.0);

            if self.entry(current_id).and_then(|e| e.mixing_from).is_some() {
                alpha *= self.apply_mixing_from_pose(current_id, skeleton, blend, &mut pending);
            } else {
                let track_end_reached = {
                    let track = &self.tracks[track_index];
                    let queued_empty = track.queue.is_empty();
                    let reached = self.entry(current_id).is_some_and(|e| {
                        e.track_time + TIME_EPSILON >= e.track_end && e.track_end.is_finite()
                    });
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
            if track_index == 0 && (alpha >= 1.0 || blend == MixBlend::Add) {
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

            self.apply_entry_events_and_complete(current_id, None, !reverse, &mut pending);
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
        if alpha <= 0.0 {
            return;
        }

        let shortest_rotation = self
            .entry(entry_id)
            .map(|e| e.shortest_rotation)
            .unwrap_or(false);

        let mut time = time;
        if looped && animation.duration > 0.0 {
            time = time.rem_euclid(animation.duration);
        }

        // Physics reset timelines need `lastTime`, so apply them separately.
        if !animation.physics_reset_timelines.is_empty() {
            let reverse = self.entry(entry_id).map(|e| e.reverse).unwrap_or(false);
            if !reverse {
                let mut last_time = self
                    .entry(entry_id)
                    .map(|e| e.animation_last_time)
                    .unwrap_or(-1.0);
                if looped && animation.duration > 0.0 && last_time >= 0.0 {
                    last_time = last_time.rem_euclid(animation.duration);
                }
                for tl in &animation.physics_reset_timelines {
                    apply_physics_reset_timeline(tl, skeleton, last_time, time);
                }
            }
        }

        let track_index = self.entry(entry_id).map(|e| e.track_index).unwrap_or(0);
        let special_case = (track_index == 0 && alpha >= 1.0 && direction == MixDirection::In)
            || blend == MixBlend::Add;

        let kinds = timeline_kinds(animation);
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

        let rotate_count = animation
            .bone_timelines
            .iter()
            .filter(|t| matches!(t, crate::BoneTimeline::Rotate(_)))
            .count();
        let first_frame = self
            .entry_mut(entry_id)
            .map(|entry| {
                let expected_len = rotate_count * 2;
                let first = entry.rotation_state.len() != expected_len;
                if first {
                    entry.rotation_state.resize(expected_len, 0.0);
                }
                first
            })
            .unwrap_or(false);
        let unkeyed_state = self.unkeyed_state;

        let mut rotate_index = 0usize;
        for (i, kind) in kinds.into_iter().enumerate() {
            let timeline_blend =
                if special_case || matches!(timeline_mode.get(i), Some(TimelineMode::Subsequent)) {
                    blend
                } else {
                    MixBlend::Setup
                };

            match kind {
                TimelineKind::SlotAttachment(ti) => {
                    let timeline = &animation.slot_attachment_timelines[ti];
                    apply_attachment(timeline, skeleton, time, blend, attachments, unkeyed_state);
                }
                TimelineKind::Deform(ti) => {
                    let timeline = &animation.deform_timelines[ti];
                    apply_deform(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::Sequence(ti) => {
                    let timeline = &animation.sequence_timelines[ti];
                    apply_sequence_timeline(timeline, skeleton, time, timeline_blend, direction);
                }
                TimelineKind::Bone(ti) => match &animation.bone_timelines[ti] {
                    crate::BoneTimeline::Rotate(tl) => {
                        if !shortest_rotation
                            && !special_case
                            && alpha < 1.0
                            && blend != MixBlend::Add
                        {
                            if let Some(entry) = self.entry_mut(entry_id) {
                                apply_rotate_mixed(
                                    tl,
                                    skeleton,
                                    time,
                                    alpha,
                                    timeline_blend,
                                    entry.rotation_state.as_mut_slice(),
                                    rotate_index,
                                    first_frame,
                                );
                            }
                        } else {
                            apply_rotate(tl, skeleton, time, alpha, timeline_blend);
                        }
                        rotate_index += 1;
                    }
                    crate::BoneTimeline::Translate(tl) => {
                        apply_translate(tl, skeleton, time, alpha, timeline_blend);
                    }
                    crate::BoneTimeline::TranslateX(tl) => {
                        apply_translate_x(tl, skeleton, time, alpha, timeline_blend);
                    }
                    crate::BoneTimeline::TranslateY(tl) => {
                        apply_translate_y(tl, skeleton, time, alpha, timeline_blend);
                    }
                    crate::BoneTimeline::Scale(tl) => {
                        apply_scale(tl, skeleton, time, alpha, timeline_blend, direction);
                    }
                    crate::BoneTimeline::ScaleX(tl) => {
                        apply_scale_x(tl, skeleton, time, alpha, timeline_blend, direction);
                    }
                    crate::BoneTimeline::ScaleY(tl) => {
                        apply_scale_y(tl, skeleton, time, alpha, timeline_blend, direction);
                    }
                    crate::BoneTimeline::Shear(tl) => {
                        apply_shear(tl, skeleton, time, alpha, timeline_blend);
                    }
                    crate::BoneTimeline::ShearX(tl) => {
                        apply_shear_x(tl, skeleton, time, alpha, timeline_blend);
                    }
                    crate::BoneTimeline::ShearY(tl) => {
                        apply_shear_y(tl, skeleton, time, alpha, timeline_blend);
                    }
                    crate::BoneTimeline::Inherit(tl) => {
                        apply_inherit(tl, skeleton, time, timeline_blend, direction);
                    }
                },
                TimelineKind::SlotColor(ti) => {
                    let timeline = &animation.slot_color_timelines[ti];
                    apply_slot_color(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::SlotRgb(ti) => {
                    let timeline = &animation.slot_rgb_timelines[ti];
                    apply_slot_rgb(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::SlotAlpha(ti) => {
                    let timeline = &animation.slot_alpha_timelines[ti];
                    apply_slot_alpha(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::SlotRgba2(ti) => {
                    let timeline = &animation.slot_rgba2_timelines[ti];
                    apply_slot_rgba2(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::SlotRgb2(ti) => {
                    let timeline = &animation.slot_rgb2_timelines[ti];
                    apply_slot_rgb2(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::IkConstraint(ti) => {
                    let timeline = &animation.ik_constraint_timelines[ti];
                    apply_ik_constraint_timeline(
                        timeline,
                        skeleton,
                        time,
                        alpha,
                        timeline_blend,
                        direction,
                    );
                }
                TimelineKind::TransformConstraint(ti) => {
                    let timeline = &animation.transform_constraint_timelines[ti];
                    apply_transform_constraint_timeline(
                        timeline,
                        skeleton,
                        time,
                        alpha,
                        timeline_blend,
                    );
                }
                TimelineKind::PathConstraint(ti) => {
                    let timeline = &animation.path_constraint_timelines[ti];
                    apply_path_constraint_timeline(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::PhysicsConstraint(ti) => {
                    let timeline = &animation.physics_constraint_timelines[ti];
                    apply_physics_constraint_timeline(
                        timeline,
                        skeleton,
                        time,
                        alpha,
                        timeline_blend,
                    );
                }
                TimelineKind::SliderTime(ti) => {
                    let timeline = &animation.slider_time_timelines[ti];
                    apply_slider_time_timeline(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::SliderMix(ti) => {
                    let timeline = &animation.slider_mix_timelines[ti];
                    apply_slider_mix_timeline(timeline, skeleton, time, alpha, timeline_blend);
                }
                TimelineKind::DrawOrder => {
                    if let Some(timeline) = animation.draw_order_timeline.as_ref() {
                        apply_draw_order(timeline, skeleton, time, timeline_blend, direction);
                    }
                }
            }
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

        if self
            .entry(from)
            .and_then(|entry| entry.mixing_from)
            .is_some()
        {
            self.apply_mixing_from_pose(from, skeleton, blend, out);
        }

        let (mix_time, mix_duration, interrupt_alpha) = self
            .entry(to)
            .map(|to_ref| (to_ref.mix_time, to_ref.mix_duration, to_ref.interrupt_alpha))
            .unwrap_or((0.0, 0.0, 1.0));

        let (
            from_animation,
            from_time,
            from_looped,
            from_reverse,
            from_shortest_rotation,
            from_mix_blend,
            from_alpha,
            from_thresholds,
        ) = match self.entry(from) {
            Some(from_ref) => (
                from_ref.animation.clone(),
                from_ref.animation_time(),
                from_ref.looped,
                from_ref.reverse,
                from_ref.shortest_rotation,
                from_ref.mix_blend,
                from_ref.alpha,
                (
                    from_ref.alpha_attachment_threshold,
                    from_ref.mix_attachment_threshold,
                    from_ref.mix_draw_order_threshold,
                ),
            ),
            None => return 1.0,
        };

        let mut from_blend = blend;
        let mix = if mix_duration <= 0.0 {
            if from_blend == MixBlend::First {
                from_blend = MixBlend::Setup;
            }
            1.0
        } else {
            let m = (mix_time / mix_duration).clamp(0.0, 1.0);
            if from_blend != MixBlend::First {
                from_blend = from_mix_blend;
            }
            m
        };

        let alpha_hold = from_alpha * interrupt_alpha;
        let alpha_mix = alpha_hold * (1.0 - mix);

        if let Some(from_entry) = self.entry_mut(from) {
            from_entry.total_alpha = 0.0;
        }

        let attachments = mix + TIME_EPSILON < from_thresholds.1;
        let draw_order = mix + TIME_EPSILON < from_thresholds.2;

        let from_apply_time = if from_reverse {
            from_animation.duration - from_time
        } else {
            from_time
        };

        if from_blend == MixBlend::Add {
            // Match spine-cpp: in Add mode, mixing out applies timelines with `MixDirection::Out`
            // and does not use the per-timeline hold/subsequent machinery. Timelines that perform
            // instant changes (attachment/draw order) become no-ops for Add+Out.
            let mut time = from_apply_time;
            if from_looped && from_animation.duration > 0.0 {
                time = time.rem_euclid(from_animation.duration);
            }

            if !from_animation.physics_reset_timelines.is_empty() && !from_reverse {
                let mut last_time = self
                    .entry(from)
                    .map(|e| e.animation_last_time)
                    .unwrap_or(-1.0);
                if from_looped && from_animation.duration > 0.0 && last_time >= 0.0 {
                    last_time = last_time.rem_euclid(from_animation.duration);
                }
                for tl in &from_animation.physics_reset_timelines {
                    apply_physics_reset_timeline(tl, skeleton, last_time, time);
                }
            }

            for kind in timeline_kinds(&from_animation) {
                match kind {
                    TimelineKind::SlotAttachment(_) => {
                        // AttachmentTimeline::apply for MixDirection::Out only does work for
                        // blend==Setup; for Add it is a no-op.
                    }
                    TimelineKind::Sequence(_) => {
                        // SequenceTimeline::apply for MixDirection::Out only does work for blend==Setup;
                        // for Add it is a no-op.
                    }
                    TimelineKind::Deform(ti) => {
                        let timeline = &from_animation.deform_timelines[ti];
                        apply_deform(timeline, skeleton, time, alpha_mix, MixBlend::Add);
                    }
                    TimelineKind::Bone(ti) => match &from_animation.bone_timelines[ti] {
                        crate::BoneTimeline::Rotate(tl) => {
                            apply_rotate(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::Translate(tl) => {
                            apply_translate(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::TranslateX(tl) => {
                            apply_translate_x(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::TranslateY(tl) => {
                            apply_translate_y(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::Scale(tl) => {
                            apply_scale(
                                tl,
                                skeleton,
                                time,
                                alpha_mix,
                                MixBlend::Add,
                                MixDirection::Out,
                            );
                        }
                        crate::BoneTimeline::ScaleX(tl) => {
                            apply_scale_x(
                                tl,
                                skeleton,
                                time,
                                alpha_mix,
                                MixBlend::Add,
                                MixDirection::Out,
                            );
                        }
                        crate::BoneTimeline::ScaleY(tl) => {
                            apply_scale_y(
                                tl,
                                skeleton,
                                time,
                                alpha_mix,
                                MixBlend::Add,
                                MixDirection::Out,
                            );
                        }
                        crate::BoneTimeline::Shear(tl) => {
                            apply_shear(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::ShearX(tl) => {
                            apply_shear_x(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::ShearY(tl) => {
                            apply_shear_y(tl, skeleton, time, alpha_mix, MixBlend::Add);
                        }
                        crate::BoneTimeline::Inherit(tl) => {
                            apply_inherit(tl, skeleton, time, MixBlend::Add, MixDirection::Out);
                        }
                    },
                    TimelineKind::SlotColor(ti) => {
                        let timeline = &from_animation.slot_color_timelines[ti];
                        apply_slot_color(timeline, skeleton, time, alpha_mix, MixBlend::Add);
                    }
                    TimelineKind::SlotRgb(ti) => {
                        let timeline = &from_animation.slot_rgb_timelines[ti];
                        apply_slot_rgb(timeline, skeleton, time, alpha_mix, MixBlend::Add);
                    }
                    TimelineKind::SlotAlpha(ti) => {
                        let timeline = &from_animation.slot_alpha_timelines[ti];
                        apply_slot_alpha(timeline, skeleton, time, alpha_mix, MixBlend::Add);
                    }
                    TimelineKind::SlotRgba2(ti) => {
                        let timeline = &from_animation.slot_rgba2_timelines[ti];
                        apply_slot_rgba2(timeline, skeleton, time, alpha_mix, MixBlend::Add);
                    }
                    TimelineKind::SlotRgb2(ti) => {
                        let timeline = &from_animation.slot_rgb2_timelines[ti];
                        apply_slot_rgb2(timeline, skeleton, time, alpha_mix, MixBlend::Add);
                    }
                    TimelineKind::IkConstraint(ti) => {
                        let timeline = &from_animation.ik_constraint_timelines[ti];
                        apply_ik_constraint_timeline(
                            timeline,
                            skeleton,
                            time,
                            alpha_mix,
                            MixBlend::Add,
                            MixDirection::Out,
                        );
                    }
                    TimelineKind::TransformConstraint(ti) => {
                        let timeline = &from_animation.transform_constraint_timelines[ti];
                        apply_transform_constraint_timeline(
                            timeline,
                            skeleton,
                            time,
                            alpha_mix,
                            MixBlend::Add,
                        );
                    }
                    TimelineKind::PathConstraint(ti) => {
                        let timeline = &from_animation.path_constraint_timelines[ti];
                        apply_path_constraint_timeline(
                            timeline,
                            skeleton,
                            time,
                            alpha_mix,
                            MixBlend::Add,
                        );
                    }
                    TimelineKind::PhysicsConstraint(ti) => {
                        let timeline = &from_animation.physics_constraint_timelines[ti];
                        apply_physics_constraint_timeline(
                            timeline,
                            skeleton,
                            time,
                            alpha_mix,
                            MixBlend::Add,
                        );
                    }
                    TimelineKind::SliderTime(ti) => {
                        let timeline = &from_animation.slider_time_timelines[ti];
                        apply_slider_time_timeline(
                            timeline,
                            skeleton,
                            time,
                            alpha_mix,
                            MixBlend::Add,
                        );
                    }
                    TimelineKind::SliderMix(ti) => {
                        let timeline = &from_animation.slider_mix_timelines[ti];
                        apply_slider_mix_timeline(
                            timeline,
                            skeleton,
                            time,
                            alpha_mix,
                            MixBlend::Add,
                        );
                    }
                    TimelineKind::DrawOrder => {
                        if let Some(timeline) = from_animation.draw_order_timeline.as_ref() {
                            apply_draw_order(
                                timeline,
                                skeleton,
                                time,
                                MixBlend::Add,
                                MixDirection::Out,
                            );
                        }
                    }
                }
            }
        } else {
            let kinds = timeline_kinds(&from_animation);
            let (timeline_mode, timeline_hold_mix) = match self.entry(from) {
                Some(e) => (e.timeline_mode.clone(), e.timeline_hold_mix.clone()),
                None => (Vec::new(), Vec::new()),
            };
            let alpha_attachment_threshold = from_thresholds.0;

            if !from_animation.physics_reset_timelines.is_empty() && !from_reverse {
                let mut last_time = self
                    .entry(from)
                    .map(|e| e.animation_last_time)
                    .unwrap_or(-1.0);
                let mut time = from_apply_time;
                if from_looped && from_animation.duration > 0.0 {
                    time = time.rem_euclid(from_animation.duration);
                    if last_time >= 0.0 {
                        last_time = last_time.rem_euclid(from_animation.duration);
                    }
                }
                for tl in &from_animation.physics_reset_timelines {
                    apply_physics_reset_timeline(tl, skeleton, last_time, time);
                }
            }

            let rotate_count = from_animation
                .bone_timelines
                .iter()
                .filter(|t| matches!(t, crate::BoneTimeline::Rotate(_)))
                .count();
            let first_frame = self
                .entry_mut(from)
                .map(|entry| {
                    let expected_len = rotate_count * 2;
                    let first = entry.rotation_state.len() != expected_len;
                    if first {
                        entry.rotation_state.resize(expected_len, 0.0);
                    }
                    first
                })
                .unwrap_or(false);
            let unkeyed_state = self.unkeyed_state;

            let mut rotate_index = 0usize;
            let mut total_alpha = 0.0f32;
            for (i, kind) in kinds.into_iter().enumerate() {
                let mode = timeline_mode.get(i).copied().unwrap_or(TimelineMode::First);

                let (timeline_blend, alpha) = match mode {
                    TimelineMode::Subsequent => (from_blend, alpha_mix),
                    TimelineMode::First => (MixBlend::Setup, alpha_mix),
                    TimelineMode::HoldFirst => (MixBlend::Setup, alpha_hold),
                    TimelineMode::HoldSubsequent => (from_blend, alpha_hold),
                    TimelineMode::HoldMix => {
                        let hold_mix = timeline_hold_mix.get(i).copied().flatten();
                        if let Some(hold_mix) = hold_mix {
                            let factor = self
                                .entry(hold_mix)
                                .map(|e| {
                                    if e.mix_duration > 0.0 {
                                        (1.0 - e.mix_time / e.mix_duration).max(0.0)
                                    } else {
                                        0.0
                                    }
                                })
                                .unwrap_or(0.0);
                            (MixBlend::Setup, alpha_hold * factor)
                        } else {
                            (MixBlend::Setup, alpha_hold)
                        }
                    }
                };
                total_alpha += alpha;

                match kind {
                    TimelineKind::SlotAttachment(ti) => {
                        let timeline = &from_animation.slot_attachment_timelines[ti];
                        let apply =
                            attachments && alpha + TIME_EPSILON >= alpha_attachment_threshold;
                        apply_attachment(
                            timeline,
                            skeleton,
                            from_apply_time,
                            timeline_blend,
                            apply,
                            unkeyed_state,
                        );
                    }
                    TimelineKind::Deform(ti) => {
                        let timeline = &from_animation.deform_timelines[ti];
                        apply_deform(timeline, skeleton, from_apply_time, alpha, timeline_blend);
                    }
                    TimelineKind::Sequence(ti) => {
                        let timeline = &from_animation.sequence_timelines[ti];
                        apply_sequence_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            timeline_blend,
                            MixDirection::Out,
                        );
                    }
                    TimelineKind::Bone(ti) => match &from_animation.bone_timelines[ti] {
                        crate::BoneTimeline::Rotate(tl) => {
                            if !from_shortest_rotation && alpha < 1.0 && from_blend != MixBlend::Add
                            {
                                if let Some(entry) = self.entry_mut(from) {
                                    apply_rotate_mixed(
                                        tl,
                                        skeleton,
                                        from_apply_time,
                                        alpha,
                                        timeline_blend,
                                        entry.rotation_state.as_mut_slice(),
                                        rotate_index,
                                        first_frame,
                                    );
                                }
                            } else {
                                apply_rotate(tl, skeleton, from_apply_time, alpha, timeline_blend);
                            }
                            rotate_index += 1;
                        }
                        crate::BoneTimeline::Translate(tl) => {
                            apply_translate(tl, skeleton, from_apply_time, alpha, timeline_blend);
                        }
                        crate::BoneTimeline::TranslateX(tl) => {
                            apply_translate_x(tl, skeleton, from_apply_time, alpha, timeline_blend);
                        }
                        crate::BoneTimeline::TranslateY(tl) => {
                            apply_translate_y(tl, skeleton, from_apply_time, alpha, timeline_blend);
                        }
                        crate::BoneTimeline::Scale(tl) => {
                            apply_scale(
                                tl,
                                skeleton,
                                from_apply_time,
                                alpha,
                                timeline_blend,
                                MixDirection::Out,
                            );
                        }
                        crate::BoneTimeline::ScaleX(tl) => {
                            apply_scale_x(
                                tl,
                                skeleton,
                                from_apply_time,
                                alpha,
                                timeline_blend,
                                MixDirection::Out,
                            );
                        }
                        crate::BoneTimeline::ScaleY(tl) => {
                            apply_scale_y(
                                tl,
                                skeleton,
                                from_apply_time,
                                alpha,
                                timeline_blend,
                                MixDirection::Out,
                            );
                        }
                        crate::BoneTimeline::Shear(tl) => {
                            apply_shear(tl, skeleton, from_apply_time, alpha, timeline_blend);
                        }
                        crate::BoneTimeline::ShearX(tl) => {
                            apply_shear_x(tl, skeleton, from_apply_time, alpha, timeline_blend);
                        }
                        crate::BoneTimeline::ShearY(tl) => {
                            apply_shear_y(tl, skeleton, from_apply_time, alpha, timeline_blend);
                        }
                        crate::BoneTimeline::Inherit(tl) => {
                            apply_inherit(
                                tl,
                                skeleton,
                                from_apply_time,
                                timeline_blend,
                                MixDirection::Out,
                            );
                        }
                    },
                    TimelineKind::SlotColor(ti) => {
                        let timeline = &from_animation.slot_color_timelines[ti];
                        apply_slot_color(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::SlotRgb(ti) => {
                        let timeline = &from_animation.slot_rgb_timelines[ti];
                        apply_slot_rgb(timeline, skeleton, from_apply_time, alpha, timeline_blend);
                    }
                    TimelineKind::SlotAlpha(ti) => {
                        let timeline = &from_animation.slot_alpha_timelines[ti];
                        apply_slot_alpha(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::SlotRgba2(ti) => {
                        let timeline = &from_animation.slot_rgba2_timelines[ti];
                        apply_slot_rgba2(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::SlotRgb2(ti) => {
                        let timeline = &from_animation.slot_rgb2_timelines[ti];
                        apply_slot_rgb2(timeline, skeleton, from_apply_time, alpha, timeline_blend);
                    }
                    TimelineKind::IkConstraint(ti) => {
                        let timeline = &from_animation.ik_constraint_timelines[ti];
                        apply_ik_constraint_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                            MixDirection::Out,
                        );
                    }
                    TimelineKind::TransformConstraint(ti) => {
                        let timeline = &from_animation.transform_constraint_timelines[ti];
                        apply_transform_constraint_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::PathConstraint(ti) => {
                        let timeline = &from_animation.path_constraint_timelines[ti];
                        apply_path_constraint_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::PhysicsConstraint(ti) => {
                        let timeline = &from_animation.physics_constraint_timelines[ti];
                        apply_physics_constraint_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::SliderTime(ti) => {
                        let timeline = &from_animation.slider_time_timelines[ti];
                        apply_slider_time_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::SliderMix(ti) => {
                        let timeline = &from_animation.slider_mix_timelines[ti];
                        apply_slider_mix_timeline(
                            timeline,
                            skeleton,
                            from_apply_time,
                            alpha,
                            timeline_blend,
                        );
                    }
                    TimelineKind::DrawOrder => {
                        if let Some(timeline) = from_animation.draw_order_timeline.as_ref() {
                            let direction = if draw_order && timeline_blend == MixBlend::Setup {
                                MixDirection::In
                            } else {
                                MixDirection::Out
                            };
                            apply_draw_order(
                                timeline,
                                skeleton,
                                from_apply_time,
                                timeline_blend,
                                direction,
                            );
                        }
                    }
                }
            }
            if let Some(from_entry) = self.entry_mut(from) {
                from_entry.total_alpha = total_alpha;
            }
        }

        if mix_duration > 0.0 {
            self.apply_entry_events_and_complete(
                from,
                Some((mix_time, mix_duration)),
                !from_reverse,
                out,
            );
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
            let animation_index = if entry.animation_index == EMPTY_ANIMATION_INDEX {
                -1
            } else {
                i32::try_from(entry.animation_index).unwrap_or(i32::MAX)
            };
            TrackEntrySnapshot {
                track_index: entry.track_index,
                animation_index,
                animation_name: entry.animation.name.clone(),
                track_time: entry.track_time,
            }
        } else {
            TrackEntrySnapshot {
                track_index: 0,
                animation_index: -2,
                animation_name: "<disposed>".to_string(),
                track_time: 0.0,
            }
        }
    }

    fn take_entry_listener(&mut self, id: EntryId) -> Option<Box<dyn TrackEntryListener>> {
        self.entry_mut(id).and_then(|entry| entry.listener.take())
    }

    fn restore_entry_listener(&mut self, id: EntryId, listener: Box<dyn TrackEntryListener>) {
        if let Some(entry) = self.entry_mut(id) {
            if entry.listener.is_none() {
                entry.listener = Some(listener);
            }
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
        if to_next_track_last >= 0.0 && to_mix_time + TIME_EPSILON >= to_mix_duration {
            let from_total_alpha = self.entry(from).map(|e| e.total_alpha).unwrap_or(0.0);
            if to_mix_duration <= 0.0 || from_total_alpha.abs() <= TIME_EPSILON {
                let next_from = self.entry(from).and_then(|from_ref| from_ref.mixing_from);
                let from_interrupt_alpha =
                    self.entry(from).map(|e| e.interrupt_alpha).unwrap_or(1.0);
                if let Some(to_entry) = self.entry_mut(to) {
                    to_entry.mixing_from = next_from;
                    to_entry.interrupt_alpha = from_interrupt_alpha;
                }
                if let Some(next_from) = next_from {
                    if let Some(entry) = self.entry_mut(next_from) {
                        entry.mixing_to = Some(to);
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
        mix: Option<(f32, f32)>,
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

        let can_issue_events = match mix {
            None => true,
            Some((mix_time, mix_duration)) => {
                if mix_duration <= 0.0 {
                    false
                } else {
                    let mut percent = mix_time / mix_duration;
                    if percent > 1.0 {
                        percent = 1.0;
                    }
                    percent + TIME_EPSILON < entry.event_threshold
                }
            }
        };

        let mut events = Vec::new();
        if events_enabled && can_issue_events {
            if let Some(timeline) = &entry.animation.event_timeline {
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
            push_event(&mut self.event_queue, entry, AnimationStateEvent::Dispose);
        }
    }

    fn drain_event_queue(&mut self) {
        if self.draining_events {
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
                self.free_entry(entry_id);
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
