use crate::{
    AlphaTimeline, Animation, AttachmentTimeline, BoneTimeline, ColorTimeline, Curve,
    DeformTimeline, DrawOrderFolderTimeline, DrawOrderTimeline, IkConstraintTimeline,
    InheritTimeline, PathConstraintTimeline, PhysicsConstraintResetTimeline,
    PhysicsConstraintTimeline, Rgb2Timeline, RgbTimeline, Rgba2Timeline, RotateFrame,
    RotateTimeline, ScaleTimeline, ScaleXTimeline, ScaleYTimeline, ShearTimeline, ShearXTimeline,
    ShearYTimeline, Skeleton, SliderConstraintTimeline, TimelineKind, TransformConstraintTimeline,
    TranslateTimeline, TranslateXTimeline, TranslateYTimeline, Vec2Frame,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MixBlend {
    Setup,
    First,
    Replace,
    Add,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum MixFrom {
    Current,
    Setup,
    First,
}

impl MixFrom {
    fn from_blend(blend: MixBlend) -> Self {
        match blend {
            MixBlend::Setup => Self::Setup,
            MixBlend::First => Self::First,
            MixBlend::Replace | MixBlend::Add => Self::Current,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MixDirection {
    In,
    Out,
}

pub(crate) const ANIMATION_STATE_CURRENT: i32 = 2;
pub(crate) const ANIMATION_STATE_SETUP: i32 = 1;

pub fn apply_animation(
    animation: &Animation,
    skeleton: &mut Skeleton,
    time: f32,
    looped: bool,
    alpha: f32,
    blend: MixBlend,
) {
    if animation.bone_timelines.is_empty()
        && animation.deform_timelines.is_empty()
        && animation.sequence_timelines.is_empty()
        && animation.slot_attachment_timelines.is_empty()
        && animation.slot_color_timelines.is_empty()
        && animation.slot_rgb_timelines.is_empty()
        && animation.slot_alpha_timelines.is_empty()
        && animation.slot_rgba2_timelines.is_empty()
        && animation.slot_rgb2_timelines.is_empty()
        && animation.ik_constraint_timelines.is_empty()
        && animation.transform_constraint_timelines.is_empty()
        && animation.path_constraint_timelines.is_empty()
        && animation.physics_constraint_timelines.is_empty()
        && animation.physics_reset_timelines.is_empty()
        && animation.slider_time_timelines.is_empty()
        && animation.slider_mix_timelines.is_empty()
        && animation.draw_order_timeline.is_none()
        && animation.draw_order_folder_timelines.is_empty()
    {
        return;
    }

    let mut time = time;
    if looped && animation.duration > 0.0 {
        time = time.rem_euclid(animation.duration);
    }

    // Plain animation apply does not model AnimationState's attachmentState gating. Use the legacy
    // behaviour: always apply attachments.
    for kind in animation.timeline_order.iter().copied() {
        apply_animation_timeline(
            animation,
            kind,
            skeleton,
            RuntimeTimelineApply {
                time,
                alpha,
                blend,
                direction: MixDirection::In,
                applied_pose: false,
                attachments: true,
                unkeyed_state: 0,
                physics_last_time: time,
                physics_time: time,
            },
        );
    }
}

/// Applies an animation to the skeleton's *applied* pose (bone `a*` fields).
///
/// This matches Spine's `Animation::apply(..., appliedPose=true)` usage, which is required by
/// constraints such as Slider (Spine 4.3) that drive pose changes during `updateWorldTransform`.
pub(crate) fn apply_animation_applied(
    animation: &Animation,
    skeleton: &mut Skeleton,
    time: f32,
    looped: bool,
    alpha: f32,
    blend: MixBlend,
) {
    if animation.bone_timelines.is_empty()
        && animation.deform_timelines.is_empty()
        && animation.sequence_timelines.is_empty()
        && animation.slot_attachment_timelines.is_empty()
        && animation.slot_color_timelines.is_empty()
        && animation.slot_rgb_timelines.is_empty()
        && animation.slot_alpha_timelines.is_empty()
        && animation.slot_rgba2_timelines.is_empty()
        && animation.slot_rgb2_timelines.is_empty()
        && animation.ik_constraint_timelines.is_empty()
        && animation.transform_constraint_timelines.is_empty()
        && animation.path_constraint_timelines.is_empty()
        && animation.physics_constraint_timelines.is_empty()
        && animation.physics_reset_timelines.is_empty()
        && animation.slider_time_timelines.is_empty()
        && animation.slider_mix_timelines.is_empty()
        && animation.draw_order_timeline.is_none()
        && animation.draw_order_folder_timelines.is_empty()
    {
        return;
    }

    let mut time = time;
    if looped && animation.duration > 0.0 {
        time = time.rem_euclid(animation.duration);
    }

    for kind in animation.timeline_order.iter().copied() {
        apply_animation_timeline(
            animation,
            kind,
            skeleton,
            RuntimeTimelineApply {
                time,
                alpha,
                blend,
                direction: MixDirection::In,
                applied_pose: true,
                attachments: true,
                unkeyed_state: 0,
                physics_last_time: time,
                physics_time: time,
            },
        );
    }
}

#[derive(Copy, Clone)]
struct RuntimeTimelineApply {
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
    applied_pose: bool,
    attachments: bool,
    unkeyed_state: i32,
    physics_last_time: f32,
    physics_time: f32,
}

pub(crate) struct RotateTimelineState<'a> {
    pub state: &'a mut [f32],
    pub index: usize,
    pub first_frame: bool,
}

pub(crate) struct StateTimelineApply<'a> {
    pub time: f32,
    pub alpha: f32,
    pub timeline_blend: MixBlend,
    pub additive_blend: MixBlend,
    pub from: MixFrom,
    pub additive: bool,
    pub transform_additive: bool,
    pub direction: MixDirection,
    pub attachments: bool,
    pub unkeyed_state: i32,
    pub draw_order_from_current: bool,
    pub draw_order_out: Option<bool>,
    pub draw_order_folder_out: bool,
    pub physics_last_time: f32,
    pub physics_time: f32,
    pub rotate: Option<RotateTimelineState<'a>>,
}

fn apply_animation_timeline(
    animation: &Animation,
    kind: TimelineKind,
    skeleton: &mut Skeleton,
    ctx: RuntimeTimelineApply,
) {
    match kind {
        TimelineKind::SlotAttachment(i) => {
            apply_attachment(
                &animation.slot_attachment_timelines[i],
                skeleton,
                ctx.time,
                ctx.blend,
                ctx.attachments,
                ctx.unkeyed_state,
            );
        }
        TimelineKind::Deform(i) => {
            apply_deform(
                &animation.deform_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::Sequence(i) => {
            apply_sequence_timeline(
                &animation.sequence_timelines[i],
                skeleton,
                ctx.time,
                ctx.blend,
                ctx.direction,
            );
        }
        TimelineKind::Bone(i) => match &animation.bone_timelines[i] {
            BoneTimeline::Rotate(t) => {
                if ctx.applied_pose {
                    apply_rotate_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_rotate(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::Translate(t) => {
                if ctx.applied_pose {
                    apply_translate_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_translate(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::TranslateX(t) => {
                if ctx.applied_pose {
                    apply_translate_x_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_translate_x(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::TranslateY(t) => {
                if ctx.applied_pose {
                    apply_translate_y_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_translate_y(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::Scale(t) => {
                if ctx.applied_pose {
                    apply_scale_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend, ctx.direction);
                } else {
                    apply_scale(t, skeleton, ctx.time, ctx.alpha, ctx.blend, ctx.direction);
                }
            }
            BoneTimeline::ScaleX(t) => {
                if ctx.applied_pose {
                    apply_scale_x_applied(
                        t,
                        skeleton,
                        ctx.time,
                        ctx.alpha,
                        ctx.blend,
                        ctx.direction,
                    );
                } else {
                    apply_scale_x(t, skeleton, ctx.time, ctx.alpha, ctx.blend, ctx.direction);
                }
            }
            BoneTimeline::ScaleY(t) => {
                if ctx.applied_pose {
                    apply_scale_y_applied(
                        t,
                        skeleton,
                        ctx.time,
                        ctx.alpha,
                        ctx.blend,
                        ctx.direction,
                    );
                } else {
                    apply_scale_y(t, skeleton, ctx.time, ctx.alpha, ctx.blend, ctx.direction);
                }
            }
            BoneTimeline::Shear(t) => {
                if ctx.applied_pose {
                    apply_shear_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_shear(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::ShearX(t) => {
                if ctx.applied_pose {
                    apply_shear_x_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_shear_x(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::ShearY(t) => {
                if ctx.applied_pose {
                    apply_shear_y_applied(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                } else {
                    apply_shear_y(t, skeleton, ctx.time, ctx.alpha, ctx.blend);
                }
            }
            BoneTimeline::Inherit(t) => {
                apply_inherit(t, skeleton, ctx.time, ctx.blend, ctx.direction);
            }
        },
        TimelineKind::SlotColor(i) => {
            apply_slot_color(
                &animation.slot_color_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::SlotRgb(i) => {
            apply_slot_rgb(
                &animation.slot_rgb_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::SlotAlpha(i) => {
            apply_slot_alpha(
                &animation.slot_alpha_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::SlotRgba2(i) => {
            apply_slot_rgba2(
                &animation.slot_rgba2_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::SlotRgb2(i) => {
            apply_slot_rgb2(
                &animation.slot_rgb2_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::IkConstraint(i) => {
            apply_ik_constraint_timeline(
                &animation.ik_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
                ctx.direction,
            );
        }
        TimelineKind::TransformConstraint(i) => {
            apply_transform_constraint_timeline(
                &animation.transform_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::PathConstraint(i) => {
            apply_path_constraint_timeline(
                &animation.path_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::PhysicsConstraint(i) => {
            apply_physics_constraint_timeline(
                &animation.physics_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::PhysicsReset(i) => {
            apply_physics_reset_timeline(
                &animation.physics_reset_timelines[i],
                skeleton,
                ctx.physics_last_time,
                ctx.physics_time,
            );
        }
        TimelineKind::SliderTime(i) => {
            apply_slider_time_timeline(
                &animation.slider_time_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::SliderMix(i) => {
            apply_slider_mix_timeline(
                &animation.slider_mix_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.blend,
            );
        }
        TimelineKind::DrawOrder => {
            if let Some(timeline) = animation.draw_order_timeline.as_ref() {
                apply_draw_order(
                    timeline,
                    skeleton,
                    ctx.time,
                    mix_blend_is_current(ctx.blend),
                    false,
                );
            }
        }
        TimelineKind::DrawOrderFolder(i) => {
            apply_draw_order_folder(
                &animation.draw_order_folder_timelines[i],
                skeleton,
                ctx.time,
                mix_blend_is_current(ctx.blend),
                false,
            );
        }
    }
}

pub(crate) fn apply_state_timeline(
    animation: &Animation,
    kind: TimelineKind,
    skeleton: &mut Skeleton,
    ctx: StateTimelineApply<'_>,
) {
    match kind {
        TimelineKind::SlotAttachment(i) => {
            apply_attachment(
                &animation.slot_attachment_timelines[i],
                skeleton,
                ctx.time,
                ctx.timeline_blend,
                ctx.attachments,
                ctx.unkeyed_state,
            );
        }
        TimelineKind::Deform(i) => {
            apply_deform(
                &animation.deform_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.additive_blend,
            );
        }
        TimelineKind::Sequence(i) => {
            apply_sequence_timeline(
                &animation.sequence_timelines[i],
                skeleton,
                ctx.time,
                ctx.timeline_blend,
                ctx.direction,
            );
        }
        TimelineKind::Bone(i) => match &animation.bone_timelines[i] {
            BoneTimeline::Rotate(t) => {
                if let Some(rotate) = ctx.rotate {
                    apply_rotate_mixed(
                        t,
                        skeleton,
                        ctx.time,
                        ctx.alpha,
                        ctx.from,
                        rotate.state,
                        rotate.index,
                        rotate.first_frame,
                    );
                } else {
                    apply_rotate_with(
                        t,
                        skeleton,
                        ctx.time,
                        ctx.alpha,
                        ctx.from,
                        ctx.additive_blend == MixBlend::Add,
                    );
                }
            }
            BoneTimeline::Translate(t) => {
                apply_translate(t, skeleton, ctx.time, ctx.alpha, ctx.additive_blend);
            }
            BoneTimeline::TranslateX(t) => {
                apply_translate_x(t, skeleton, ctx.time, ctx.alpha, ctx.additive_blend);
            }
            BoneTimeline::TranslateY(t) => {
                apply_translate_y(t, skeleton, ctx.time, ctx.alpha, ctx.additive_blend);
            }
            BoneTimeline::Scale(t) => {
                apply_scale_with(
                    t,
                    skeleton,
                    ctx.time,
                    ctx.alpha,
                    ctx.from,
                    ctx.additive,
                    ctx.direction,
                );
            }
            BoneTimeline::ScaleX(t) => {
                apply_scale_x_with(
                    t,
                    skeleton,
                    ctx.time,
                    ctx.alpha,
                    ctx.from,
                    ctx.additive,
                    ctx.direction,
                );
            }
            BoneTimeline::ScaleY(t) => {
                apply_scale_y_with(
                    t,
                    skeleton,
                    ctx.time,
                    ctx.alpha,
                    ctx.from,
                    ctx.additive,
                    ctx.direction,
                );
            }
            BoneTimeline::Shear(t) => {
                apply_shear(t, skeleton, ctx.time, ctx.alpha, ctx.additive_blend);
            }
            BoneTimeline::ShearX(t) => {
                apply_shear_x(t, skeleton, ctx.time, ctx.alpha, ctx.additive_blend);
            }
            BoneTimeline::ShearY(t) => {
                apply_shear_y(t, skeleton, ctx.time, ctx.alpha, ctx.additive_blend);
            }
            BoneTimeline::Inherit(t) => {
                apply_inherit(t, skeleton, ctx.time, ctx.timeline_blend, ctx.direction);
            }
        },
        TimelineKind::SlotColor(i) => {
            apply_slot_color(
                &animation.slot_color_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.timeline_blend,
            );
        }
        TimelineKind::SlotRgb(i) => {
            apply_slot_rgb(
                &animation.slot_rgb_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.timeline_blend,
            );
        }
        TimelineKind::SlotAlpha(i) => {
            apply_slot_alpha(
                &animation.slot_alpha_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.timeline_blend,
            );
        }
        TimelineKind::SlotRgba2(i) => {
            apply_slot_rgba2(
                &animation.slot_rgba2_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.timeline_blend,
            );
        }
        TimelineKind::SlotRgb2(i) => {
            apply_slot_rgb2(
                &animation.slot_rgb2_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.timeline_blend,
            );
        }
        TimelineKind::IkConstraint(i) => {
            apply_ik_constraint_timeline(
                &animation.ik_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.timeline_blend,
                ctx.direction,
            );
        }
        TimelineKind::TransformConstraint(i) => {
            apply_transform_constraint_timeline_with(
                &animation.transform_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.from,
                ctx.transform_additive,
            );
        }
        TimelineKind::PathConstraint(i) => {
            apply_path_constraint_timeline_with(
                &animation.path_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.from,
                ctx.additive,
            );
        }
        TimelineKind::PhysicsConstraint(i) => {
            apply_physics_constraint_timeline_with(
                &animation.physics_constraint_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.from,
                ctx.additive,
            );
        }
        TimelineKind::SliderTime(i) => {
            apply_slider_time_timeline_with(
                &animation.slider_time_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.from,
                ctx.additive,
            );
        }
        TimelineKind::SliderMix(i) => {
            apply_slider_mix_timeline_with(
                &animation.slider_mix_timelines[i],
                skeleton,
                ctx.time,
                ctx.alpha,
                ctx.from,
                ctx.additive,
            );
        }
        TimelineKind::DrawOrder => {
            if let Some(timeline) = animation.draw_order_timeline.as_ref()
                && let Some(out) = ctx.draw_order_out
            {
                apply_draw_order(
                    timeline,
                    skeleton,
                    ctx.time,
                    ctx.draw_order_from_current,
                    out,
                );
            }
        }
        TimelineKind::DrawOrderFolder(i) => {
            apply_draw_order_folder(
                &animation.draw_order_folder_timelines[i],
                skeleton,
                ctx.time,
                ctx.draw_order_from_current,
                ctx.draw_order_folder_out,
            );
        }
        TimelineKind::PhysicsReset(i) => {
            apply_physics_reset_timeline(
                &animation.physics_reset_timelines[i],
                skeleton,
                ctx.physics_last_time,
                ctx.physics_time,
            );
        }
    }
}

pub(crate) fn apply_rotate_applied(
    timeline: &RotateTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.rotation)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.arotation = setup,
            MixBlend::First => bone.arotation += (setup - bone.arotation) * alpha,
            _ => {}
        }
        return;
    }

    let value = sample_rotate(&timeline.frames, time);
    match blend {
        MixBlend::Setup => bone.arotation = setup + value * alpha,
        MixBlend::First | MixBlend::Replace => {
            bone.arotation += (value + setup - bone.arotation) * alpha;
        }
        MixBlend::Add => bone.arotation += value * alpha,
    };
}

pub(crate) fn apply_translate_applied(
    timeline: &TranslateTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.x, b.y))
        .unwrap_or((0.0, 0.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                bone.ax = setup.0;
                bone.ay = setup.1;
            }
            MixBlend::First => {
                bone.ax += (setup.0 - bone.ax) * alpha;
                bone.ay += (setup.1 - bone.ay) * alpha;
            }
            _ => {}
        }
        return;
    }

    let offset = sample_vec2(&timeline.frames, time);
    let target_x = setup.0 + offset.0;
    let target_y = setup.1 + offset.1;

    match blend {
        MixBlend::Setup => {
            bone.ax = setup.0 + offset.0 * alpha;
            bone.ay = setup.1 + offset.1 * alpha;
        }
        MixBlend::First | MixBlend::Replace => {
            bone.ax += (target_x - bone.ax) * alpha;
            bone.ay += (target_y - bone.ay) * alpha;
        }
        MixBlend::Add => {
            bone.ax += offset.0 * alpha;
            bone.ay += offset.1 * alpha;
        }
    };
}

pub(crate) fn apply_translate_x_applied(
    timeline: &TranslateXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.x)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.ax = setup,
            MixBlend::First => bone.ax += (setup - bone.ax) * alpha,
            _ => {}
        }
        return;
    }

    let offset = sample_float(&timeline.frames, time);
    match blend {
        MixBlend::Setup => bone.ax = setup + offset * alpha,
        MixBlend::First | MixBlend::Replace => bone.ax += (setup + offset - bone.ax) * alpha,
        MixBlend::Add => bone.ax += offset * alpha,
    }
}

pub(crate) fn apply_translate_y_applied(
    timeline: &TranslateYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.y)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.ay = setup,
            MixBlend::First => bone.ay += (setup - bone.ay) * alpha,
            _ => {}
        }
        return;
    }

    let offset = sample_float(&timeline.frames, time);
    match blend {
        MixBlend::Setup => bone.ay = setup + offset * alpha,
        MixBlend::First | MixBlend::Replace => bone.ay += (setup + offset - bone.ay) * alpha,
        MixBlend::Add => bone.ay += offset * alpha,
    }
}

pub(crate) fn apply_scale_applied(
    timeline: &ScaleTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.scale_x, b.scale_y))
        .unwrap_or((1.0, 1.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                bone.ascale_x = setup.0;
                bone.ascale_y = setup.1;
            }
            MixBlend::First => {
                bone.ascale_x += (setup.0 - bone.ascale_x) * alpha;
                bone.ascale_y += (setup.1 - bone.ascale_y) * alpha;
            }
            _ => {}
        }
        return;
    }

    let mult = sample_vec2(&timeline.frames, time);
    let x = setup.0 * mult.0;
    let y = setup.1 * mult.1;

    if alpha >= 1.0 {
        match blend {
            MixBlend::Add => {
                bone.ascale_x += x - setup.0;
                bone.ascale_y += y - setup.1;
            }
            _ => {
                bone.ascale_x = x;
                bone.ascale_y = y;
            }
        }
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    match direction {
        MixDirection::Out => match blend {
            MixBlend::Setup => {
                let bx = setup.0;
                let by = setup.1;
                bone.ascale_x = bx + (x.abs() * signum(bx) - bx) * alpha;
                bone.ascale_y = by + (y.abs() * signum(by) - by) * alpha;
            }
            MixBlend::First | MixBlend::Replace => {
                let bx = bone.ascale_x;
                let by = bone.ascale_y;
                bone.ascale_x = bx + (x.abs() * signum(bx) - bx) * alpha;
                bone.ascale_y = by + (y.abs() * signum(by) - by) * alpha;
            }
            MixBlend::Add => {
                bone.ascale_x += (x - setup.0) * alpha;
                bone.ascale_y += (y - setup.1) * alpha;
            }
        },
        MixDirection::In => match blend {
            MixBlend::Setup => {
                let bx = setup.0.abs() * signum(x);
                let by = setup.1.abs() * signum(y);
                bone.ascale_x = bx + (x - bx) * alpha;
                bone.ascale_y = by + (y - by) * alpha;
            }
            MixBlend::First | MixBlend::Replace => {
                let bx = bone.ascale_x.abs() * signum(x);
                let by = bone.ascale_y.abs() * signum(y);
                bone.ascale_x = bx + (x - bx) * alpha;
                bone.ascale_y = by + (y - by) * alpha;
            }
            MixBlend::Add => {
                bone.ascale_x += (x - setup.0) * alpha;
                bone.ascale_y += (y - setup.1) * alpha;
            }
        },
    }
}

pub(crate) fn apply_scale_x_applied(
    timeline: &ScaleXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.scale_x)
        .unwrap_or(1.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.ascale_x = setup,
            MixBlend::First => bone.ascale_x += (setup - bone.ascale_x) * alpha,
            _ => {}
        }
        return;
    }

    let value = sample_float(&timeline.frames, time) * setup;
    if alpha >= 1.0 {
        match blend {
            MixBlend::Add => bone.ascale_x = bone.ascale_x + value - setup,
            _ => bone.ascale_x = value,
        }
        return;
    }

    if blend == MixBlend::Add {
        bone.ascale_x += (value - setup) * alpha;
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    match direction {
        MixDirection::Out => match blend {
            MixBlend::Setup => {
                let bx = setup;
                bone.ascale_x = bx + (value.abs() * signum(bx) - bx) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let bx = bone.ascale_x;
                bone.ascale_x = bx + (value.abs() * signum(bx) - bx) * alpha;
                return;
            }
            _ => {}
        },
        MixDirection::In => match blend {
            MixBlend::Setup => {
                let s = setup.abs() * signum(value);
                bone.ascale_x = s + (value - s) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let s = bone.ascale_x.abs() * signum(value);
                bone.ascale_x = s + (value - s) * alpha;
                return;
            }
            _ => {}
        },
    }

    bone.ascale_x += (value - bone.ascale_x) * alpha;
}

pub(crate) fn apply_scale_y_applied(
    timeline: &ScaleYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.scale_y)
        .unwrap_or(1.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.ascale_y = setup,
            MixBlend::First => bone.ascale_y += (setup - bone.ascale_y) * alpha,
            _ => {}
        }
        return;
    }

    let value = sample_float(&timeline.frames, time) * setup;
    if alpha >= 1.0 {
        match blend {
            MixBlend::Add => bone.ascale_y = bone.ascale_y + value - setup,
            _ => bone.ascale_y = value,
        }
        return;
    }

    if blend == MixBlend::Add {
        bone.ascale_y += (value - setup) * alpha;
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    match direction {
        MixDirection::Out => match blend {
            MixBlend::Setup => {
                let by = setup;
                bone.ascale_y = by + (value.abs() * signum(by) - by) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let by = bone.ascale_y;
                bone.ascale_y = by + (value.abs() * signum(by) - by) * alpha;
                return;
            }
            _ => {}
        },
        MixDirection::In => match blend {
            MixBlend::Setup => {
                let s = setup.abs() * signum(value);
                bone.ascale_y = s + (value - s) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let s = bone.ascale_y.abs() * signum(value);
                bone.ascale_y = s + (value - s) * alpha;
                return;
            }
            _ => {}
        },
    }

    bone.ascale_y += (value - bone.ascale_y) * alpha;
}

pub(crate) fn apply_shear_applied(
    timeline: &ShearTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.shear_x, b.shear_y))
        .unwrap_or((0.0, 0.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                bone.ashear_x = setup.0;
                bone.ashear_y = setup.1;
            }
            MixBlend::First => {
                bone.ashear_x += (setup.0 - bone.ashear_x) * alpha;
                bone.ashear_y += (setup.1 - bone.ashear_y) * alpha;
            }
            _ => {}
        }
        return;
    }

    let (x, y) = sample_vec2(&timeline.frames, time);
    match blend {
        MixBlend::Setup => {
            bone.ashear_x = setup.0 + x * alpha;
            bone.ashear_y = setup.1 + y * alpha;
        }
        MixBlend::First | MixBlend::Replace => {
            bone.ashear_x += (setup.0 + x - bone.ashear_x) * alpha;
            bone.ashear_y += (setup.1 + y - bone.ashear_y) * alpha;
        }
        MixBlend::Add => {
            bone.ashear_x += x * alpha;
            bone.ashear_y += y * alpha;
        }
    }
}

pub(crate) fn apply_shear_x_applied(
    timeline: &ShearXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.shear_x)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.ashear_x = setup,
            MixBlend::First => bone.ashear_x += (setup - bone.ashear_x) * alpha,
            _ => {}
        }
        return;
    }

    let value = sample_float(&timeline.frames, time);
    match blend {
        MixBlend::Setup => bone.ashear_x = setup + value * alpha,
        MixBlend::First | MixBlend::Replace => {
            bone.ashear_x += (setup + value - bone.ashear_x) * alpha
        }
        MixBlend::Add => bone.ashear_x += value * alpha,
    }
}

pub(crate) fn apply_shear_y_applied(
    timeline: &ShearYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.shear_y)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.ashear_y = setup,
            MixBlend::First => bone.ashear_y += (setup - bone.ashear_y) * alpha,
            _ => {}
        }
        return;
    }

    let value = sample_float(&timeline.frames, time);
    match blend {
        MixBlend::Setup => bone.ashear_y = setup + value * alpha,
        MixBlend::First | MixBlend::Replace => {
            bone.ashear_y += (setup + value - bone.ashear_y) * alpha
        }
        MixBlend::Add => bone.ashear_y += value * alpha,
    }
}

pub(crate) fn apply_slider_time_timeline(
    timeline: &SliderConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    apply_slider_time_timeline_with(
        timeline,
        skeleton,
        time,
        alpha,
        MixFrom::from_blend(blend),
        blend == MixBlend::Add,
    );
}

pub(crate) fn apply_slider_time_timeline_with(
    timeline: &SliderConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
) {
    if alpha <= 0.0 || timeline.frames.is_empty() {
        return;
    }
    let Some(constraint) = skeleton
        .slider_constraints
        .get_mut(timeline.constraint_index)
    else {
        return;
    };
    let Some(data) = skeleton.data.slider_constraints.get(constraint.data_index) else {
        return;
    };

    let first_time = timeline.frames[0].time;
    if time < first_time {
        constraint.time = before_first_absolute(from, alpha, constraint.time, data.setup_time);
        return;
    }

    let sampled = sample_float(&timeline.frames, time);
    constraint.time =
        apply_absolute_value(from, add, alpha, constraint.time, data.setup_time, sampled);
}

pub(crate) fn apply_slider_mix_timeline(
    timeline: &SliderConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    apply_slider_mix_timeline_with(
        timeline,
        skeleton,
        time,
        alpha,
        MixFrom::from_blend(blend),
        blend == MixBlend::Add,
    );
}

pub(crate) fn apply_slider_mix_timeline_with(
    timeline: &SliderConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
) {
    if alpha <= 0.0 || timeline.frames.is_empty() {
        return;
    }
    let Some(constraint) = skeleton
        .slider_constraints
        .get_mut(timeline.constraint_index)
    else {
        return;
    };
    let Some(data) = skeleton.data.slider_constraints.get(constraint.data_index) else {
        return;
    };

    let first_time = timeline.frames[0].time;
    if time < first_time {
        constraint.mix = before_first_absolute(from, alpha, constraint.mix, data.setup_mix);
        return;
    }

    let sampled = sample_float(&timeline.frames, time);
    constraint.mix =
        apply_absolute_value(from, add, alpha, constraint.mix, data.setup_mix, sampled);
}

fn before_first_absolute(from: MixFrom, alpha: f32, current: f32, setup: f32) -> f32 {
    match from {
        MixFrom::Setup => setup,
        MixFrom::First => current + (setup - current) * alpha,
        MixFrom::Current => current,
    }
}

fn apply_absolute_value(
    from: MixFrom,
    add: bool,
    alpha: f32,
    current: f32,
    setup: f32,
    value: f32,
) -> f32 {
    if from == MixFrom::Setup {
        let delta = if add { value } else { value - setup };
        setup + delta * alpha
    } else {
        let delta = if add { value } else { value - current };
        current + delta * alpha
    }
}

pub(crate) fn apply_physics_reset_timeline(
    timeline: &PhysicsConstraintResetTimeline,
    skeleton: &mut Skeleton,
    mut last_time: f32,
    time: f32,
) {
    if timeline.frames.is_empty() {
        return;
    }

    let mut constraint_opt = None;
    if timeline.constraint_index != -1 {
        let idx = timeline.constraint_index as usize;
        if idx >= skeleton.physics_constraints.len() {
            return;
        }
        constraint_opt = Some(idx);
    }

    // Apply after lastTime for looped animations.
    if last_time > time {
        apply_physics_reset_timeline(timeline, skeleton, last_time, f32::INFINITY);
        last_time = -1.0;
    } else if last_time >= *timeline.frames.last().unwrap_or(&0.0) {
        return;
    }
    if time < timeline.frames[0] {
        return;
    }

    let crossed = if last_time < timeline.frames[0] {
        true
    } else {
        let idx = timeline
            .frames
            .partition_point(|t| *t <= last_time)
            .saturating_sub(1);
        let next = timeline
            .frames
            .get(idx + 1)
            .copied()
            .unwrap_or(f32::INFINITY);
        time >= next
    };

    if crossed {
        let now = skeleton.time();
        if let Some(idx) = constraint_opt {
            skeleton.physics_constraints[idx].reset_with_time(now);
        } else {
            for c in &mut skeleton.physics_constraints {
                c.reset_with_time(now);
            }
        }
    }
}

pub(crate) fn apply_physics_constraint_timeline(
    timeline: &PhysicsConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    apply_physics_constraint_timeline_with(
        timeline,
        skeleton,
        time,
        alpha,
        MixFrom::from_blend(blend),
        blend == MixBlend::Add,
    );
}

pub(crate) fn apply_physics_constraint_timeline_with(
    timeline: &PhysicsConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    entry_add: bool,
) {
    if alpha <= 0.0 {
        return;
    }

    type GetSetup = fn(&crate::PhysicsConstraintData) -> f32;
    type GetCurrent = fn(&crate::PhysicsConstraint) -> f32;
    type SetCurrent = fn(&mut crate::PhysicsConstraint, f32);
    type IsGlobal = fn(&crate::PhysicsConstraintData) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn apply_value_timeline(
        skeleton: &mut Skeleton,
        constraint_index: i32,
        frames: &[crate::FloatFrame],
        time: f32,
        alpha: f32,
        from: MixFrom,
        add: bool,
        get_setup: GetSetup,
        get_current: GetCurrent,
        set_current: SetCurrent,
        is_global: IsGlobal,
    ) {
        if frames.is_empty() {
            return;
        }

        let first_time = frames[0].time;
        let sampled = if time >= first_time {
            sample_float(frames, time)
        } else {
            0.0
        };

        let apply_one = |constraint: &mut crate::PhysicsConstraint, setup: f32, value: f32| {
            if time < first_time {
                match from {
                    MixFrom::Setup => set_current(constraint, setup),
                    MixFrom::First => {
                        let current = get_current(constraint);
                        set_current(constraint, current + (setup - current) * alpha);
                    }
                    MixFrom::Current => {}
                }
                return;
            }
            match from {
                MixFrom::Setup => {
                    let delta = if add { value } else { value - setup };
                    set_current(constraint, setup + delta * alpha);
                }
                MixFrom::First | MixFrom::Current => {
                    let current = get_current(constraint);
                    let delta = if add { value } else { value - current };
                    set_current(constraint, current + delta * alpha);
                }
            }
        };

        if constraint_index == -1 {
            for c in &mut skeleton.physics_constraints {
                let Some(data) = skeleton.data.physics_constraints.get(c.data_index()) else {
                    continue;
                };
                if !is_global(data) {
                    continue;
                }
                let setup = get_setup(data);
                apply_one(c, setup, sampled);
            }
            return;
        }

        let idx = constraint_index as usize;
        let Some(c) = skeleton.physics_constraints.get_mut(idx) else {
            return;
        };
        let setup = skeleton
            .data
            .physics_constraints
            .get(c.data_index())
            .map(get_setup)
            .unwrap_or(0.0);
        apply_one(c, setup, sampled);
    }

    match timeline {
        PhysicsConstraintTimeline::Inertia(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            false,
            |d| d.inertia,
            |c| c.inertia,
            |c, v| c.inertia = v,
            |d| d.inertia_global,
        ),
        PhysicsConstraintTimeline::Strength(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            false,
            |d| d.strength,
            |c| c.strength,
            |c, v| c.strength = v,
            |d| d.strength_global,
        ),
        PhysicsConstraintTimeline::Damping(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            false,
            |d| d.damping,
            |c| c.damping,
            |c, v| c.damping = v,
            |d| d.damping_global,
        ),
        PhysicsConstraintTimeline::Mass(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            false,
            |d| 1.0 / d.mass_inverse,
            |c| 1.0 / c.mass_inverse,
            |c, v| c.mass_inverse = 1.0 / v,
            |d| d.mass_global,
        ),
        PhysicsConstraintTimeline::Wind(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            entry_add,
            |d| d.wind,
            |c| c.wind,
            |c, v| c.wind = v,
            |d| d.wind_global,
        ),
        PhysicsConstraintTimeline::Gravity(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            entry_add,
            |d| d.gravity,
            |c| c.gravity,
            |c, v| c.gravity = v,
            |d| d.gravity_global,
        ),
        PhysicsConstraintTimeline::Mix(tl) => apply_value_timeline(
            skeleton,
            tl.constraint_index,
            &tl.frames,
            time,
            alpha,
            from,
            false,
            |d| d.mix,
            |c| c.mix,
            |c, v| c.mix = v,
            |d| d.mix_global,
        ),
    }
}

pub(crate) fn apply_path_constraint_timeline(
    timeline: &PathConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    apply_path_constraint_timeline_with(
        timeline,
        skeleton,
        time,
        alpha,
        MixFrom::from_blend(blend),
        blend == MixBlend::Add,
    );
}

pub(crate) fn apply_path_constraint_timeline_with(
    timeline: &PathConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    entry_add: bool,
) {
    match timeline {
        PathConstraintTimeline::Position(t) => {
            apply_path_position_timeline(t, skeleton, time, alpha, from, entry_add)
        }
        PathConstraintTimeline::Spacing(t) => {
            apply_path_spacing_timeline(t, skeleton, time, alpha, from)
        }
        PathConstraintTimeline::Mix(t) => {
            apply_path_mix_timeline(t, skeleton, time, alpha, from, entry_add)
        }
    }
}

fn apply_path_position_timeline(
    timeline: &crate::PathConstraintPositionTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
) {
    if alpha <= 0.0 || timeline.frames.is_empty() {
        return;
    }
    let Some(constraint) = skeleton.path_constraints.get_mut(timeline.constraint_index) else {
        return;
    };

    let setup = skeleton
        .data
        .path_constraints
        .get(timeline.constraint_index)
        .map(|c| c.position)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match from {
            MixFrom::Setup => constraint.position = setup,
            MixFrom::First => constraint.position += (setup - constraint.position) * alpha,
            MixFrom::Current => {}
        }
        return;
    }

    let sampled = sample_float(&timeline.frames, time);
    constraint.position =
        apply_absolute_value(from, add, alpha, constraint.position, setup, sampled);
}

fn apply_path_spacing_timeline(
    timeline: &crate::PathConstraintSpacingTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
) {
    if alpha <= 0.0 || timeline.frames.is_empty() {
        return;
    }
    let Some(constraint) = skeleton.path_constraints.get_mut(timeline.constraint_index) else {
        return;
    };

    let setup = skeleton
        .data
        .path_constraints
        .get(timeline.constraint_index)
        .map(|c| c.spacing)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match from {
            MixFrom::Setup => constraint.spacing = setup,
            MixFrom::First => constraint.spacing += (setup - constraint.spacing) * alpha,
            MixFrom::Current => {}
        }
        return;
    }

    let sampled = sample_float(&timeline.frames, time);
    constraint.spacing =
        apply_absolute_value(from, false, alpha, constraint.spacing, setup, sampled);
}

fn apply_path_mix_timeline(
    timeline: &crate::PathConstraintMixTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
) {
    if alpha <= 0.0 || timeline.frames.is_empty() {
        return;
    }
    let Some(constraint) = skeleton.path_constraints.get_mut(timeline.constraint_index) else {
        return;
    };

    let setup = skeleton
        .data
        .path_constraints
        .get(timeline.constraint_index)
        .map(|c| (c.mix_rotate, c.mix_x, c.mix_y))
        .unwrap_or((1.0, 1.0, 1.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match from {
            MixFrom::Setup => {
                constraint.mix_rotate = setup.0;
                constraint.mix_x = setup.1;
                constraint.mix_y = setup.2;
            }
            MixFrom::First => {
                constraint.mix_rotate += (setup.0 - constraint.mix_rotate) * alpha;
                constraint.mix_x += (setup.1 - constraint.mix_x) * alpha;
                constraint.mix_y += (setup.2 - constraint.mix_y) * alpha;
            }
            MixFrom::Current => {}
        }
        return;
    }

    let sampled = sample_path_mix(&timeline.frames, time);
    constraint.mix_rotate =
        apply_absolute_value(from, add, alpha, constraint.mix_rotate, setup.0, sampled.0);
    constraint.mix_x = apply_absolute_value(from, add, alpha, constraint.mix_x, setup.1, sampled.1);
    constraint.mix_y = apply_absolute_value(from, add, alpha, constraint.mix_y, setup.2, sampled.2);
}

fn sample_float(frames: &[crate::FloatFrame], time: f32) -> f32 {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        return frames[0].value;
    }
    if index >= frames.len() {
        return frames[frames.len() - 1].value;
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return next.value;
    }
    curve_value(
        prev.curve, time, prev.time, prev.value, next.time, next.value,
    )
}

fn sample_path_mix(frames: &[crate::PathMixFrame], time: f32) -> (f32, f32, f32) {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        let f = &frames[0];
        return (f.mix_rotate, f.mix_x, f.mix_y);
    }
    if index >= frames.len() {
        let f = &frames[frames.len() - 1];
        return (f.mix_rotate, f.mix_x, f.mix_y);
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return (next.mix_rotate, next.mix_x, next.mix_y);
    }
    (
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.mix_rotate,
            next.time,
            next.mix_rotate,
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.mix_x,
            next.time,
            next.mix_x,
        ),
        curve_value(
            prev.curve[2],
            time,
            prev.time,
            prev.mix_y,
            next.time,
            next.mix_y,
        ),
    )
}

pub(crate) fn apply_transform_constraint_timeline(
    timeline: &TransformConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    apply_transform_constraint_timeline_with(
        timeline,
        skeleton,
        time,
        alpha,
        MixFrom::from_blend(blend),
        blend == MixBlend::Add,
    );
}

pub(crate) fn apply_transform_constraint_timeline_with(
    timeline: &TransformConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
) {
    if alpha <= 0.0 {
        return;
    }
    let Some(constraint) = skeleton
        .transform_constraints
        .get_mut(timeline.constraint_index)
    else {
        return;
    };
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .transform_constraints
        .get(timeline.constraint_index)
        .map(|c| {
            (
                c.mix_rotate,
                c.mix_x,
                c.mix_y,
                c.mix_scale_x,
                c.mix_scale_y,
                c.mix_shear_y,
            )
        })
        .unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0, 0.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match from {
            MixFrom::Setup => {
                constraint.mix_rotate = setup.0;
                constraint.mix_x = setup.1;
                constraint.mix_y = setup.2;
                constraint.mix_scale_x = setup.3;
                constraint.mix_scale_y = setup.4;
                constraint.mix_shear_y = setup.5;
            }
            MixFrom::First => {
                constraint.mix_rotate += (setup.0 - constraint.mix_rotate) * alpha;
                constraint.mix_x += (setup.1 - constraint.mix_x) * alpha;
                constraint.mix_y += (setup.2 - constraint.mix_y) * alpha;
                constraint.mix_scale_x += (setup.3 - constraint.mix_scale_x) * alpha;
                constraint.mix_scale_y += (setup.4 - constraint.mix_scale_y) * alpha;
                constraint.mix_shear_y += (setup.5 - constraint.mix_shear_y) * alpha;
            }
            MixFrom::Current => {}
        }
        return;
    }

    let sampled = sample_transform_mix(&timeline.frames, time);
    let base = if from == MixFrom::Setup {
        setup
    } else {
        (
            constraint.mix_rotate,
            constraint.mix_x,
            constraint.mix_y,
            constraint.mix_scale_x,
            constraint.mix_scale_y,
            constraint.mix_shear_y,
        )
    };

    if add {
        constraint.mix_rotate = base.0 + sampled.0 * alpha;
        constraint.mix_x = base.1 + sampled.1 * alpha;
        constraint.mix_y = base.2 + sampled.2 * alpha;
        constraint.mix_scale_x = base.3 + sampled.3 * alpha;
        constraint.mix_scale_y = base.4 + sampled.4 * alpha;
        constraint.mix_shear_y = base.5 + sampled.5 * alpha;
    } else {
        constraint.mix_rotate = base.0 + (sampled.0 - base.0) * alpha;
        constraint.mix_x = base.1 + (sampled.1 - base.1) * alpha;
        constraint.mix_y = base.2 + (sampled.2 - base.2) * alpha;
        constraint.mix_scale_x = base.3 + (sampled.3 - base.3) * alpha;
        constraint.mix_scale_y = base.4 + (sampled.4 - base.4) * alpha;
        constraint.mix_shear_y = base.5 + (sampled.5 - base.5) * alpha;
    }
}

fn sample_transform_mix(
    frames: &[crate::TransformFrame],
    time: f32,
) -> (f32, f32, f32, f32, f32, f32) {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        let f = &frames[0];
        return (
            f.mix_rotate,
            f.mix_x,
            f.mix_y,
            f.mix_scale_x,
            f.mix_scale_y,
            f.mix_shear_y,
        );
    }
    if index >= frames.len() {
        let f = &frames[frames.len() - 1];
        return (
            f.mix_rotate,
            f.mix_x,
            f.mix_y,
            f.mix_scale_x,
            f.mix_scale_y,
            f.mix_shear_y,
        );
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return (
            next.mix_rotate,
            next.mix_x,
            next.mix_y,
            next.mix_scale_x,
            next.mix_scale_y,
            next.mix_shear_y,
        );
    }
    (
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.mix_rotate,
            next.time,
            next.mix_rotate,
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.mix_x,
            next.time,
            next.mix_x,
        ),
        curve_value(
            prev.curve[2],
            time,
            prev.time,
            prev.mix_y,
            next.time,
            next.mix_y,
        ),
        curve_value(
            prev.curve[3],
            time,
            prev.time,
            prev.mix_scale_x,
            next.time,
            next.mix_scale_x,
        ),
        curve_value(
            prev.curve[4],
            time,
            prev.time,
            prev.mix_scale_y,
            next.time,
            next.mix_scale_y,
        ),
        curve_value(
            prev.curve[5],
            time,
            prev.time,
            prev.mix_shear_y,
            next.time,
            next.mix_shear_y,
        ),
    )
}

pub(crate) fn apply_ik_constraint_timeline(
    timeline: &IkConstraintTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    if alpha <= 0.0 {
        return;
    }
    let Some(constraint) = skeleton.ik_constraints.get_mut(timeline.constraint_index) else {
        return;
    };
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .ik_constraints
        .get(timeline.constraint_index)
        .map(|c| (c.mix, c.softness, c.bend_direction, c.compress, c.stretch))
        .unwrap_or((1.0, 0.0, 1, false, false));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                constraint.mix = setup.0;
                constraint.softness = setup.1;
                constraint.bend_direction = setup.2;
                constraint.compress = setup.3;
                constraint.stretch = setup.4;
            }
            MixBlend::First => {
                constraint.mix += (setup.0 - constraint.mix) * alpha;
                constraint.softness += (setup.1 - constraint.softness) * alpha;
                constraint.bend_direction = setup.2;
                constraint.compress = setup.3;
                constraint.stretch = setup.4;
            }
            _ => {}
        }
        return;
    }

    let (mix_value, softness_value) = sample_ik(&timeline.frames, time);
    match blend {
        MixBlend::Setup => {
            constraint.mix = setup.0 + (mix_value - setup.0) * alpha;
            constraint.softness = setup.1 + (softness_value - setup.1) * alpha;
            if direction == MixDirection::Out {
                constraint.bend_direction = setup.2;
                constraint.compress = setup.3;
                constraint.stretch = setup.4;
                return;
            }
        }
        MixBlend::First | MixBlend::Replace => {
            constraint.mix += (mix_value - constraint.mix) * alpha;
            constraint.softness += (softness_value - constraint.softness) * alpha;
            if direction == MixDirection::Out {
                return;
            }
        }
        MixBlend::Add => {
            constraint.mix += mix_value * alpha;
            constraint.softness += softness_value * alpha;
            if direction == MixDirection::Out {
                return;
            }
        }
    }

    let frame_index = timeline
        .frames
        .partition_point(|f| f.time <= time)
        .saturating_sub(1);
    let f = &timeline.frames[frame_index];
    constraint.bend_direction = f.bend_direction;
    constraint.compress = f.compress;
    constraint.stretch = f.stretch;
}

fn sample_ik(frames: &[crate::IkFrame], time: f32) -> (f32, f32) {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        let f = &frames[0];
        return (f.mix, f.softness);
    }
    if index >= frames.len() {
        let f = &frames[frames.len() - 1];
        return (f.mix, f.softness);
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return (next.mix, next.softness);
    }
    (
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.mix,
            next.time,
            next.mix,
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.softness,
            next.time,
            next.softness,
        ),
    )
}

pub(crate) fn apply_slot_color(
    timeline: &ColorTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(slot) = skeleton.slots.get_mut(timeline.slot_index) else {
        return;
    };
    let bone_active = skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false);
    if !bone_active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .slots
        .get(timeline.slot_index)
        .map(|s| s.color)
        .unwrap_or([1.0, 1.0, 1.0, 1.0]);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => slot.color = setup,
            MixBlend::First => {
                slot.color = lerp_color(slot.color, setup, alpha);
            }
            _ => {}
        }
        return;
    }

    let target = sample_color(&timeline.frames, time);
    slot.color = match blend {
        MixBlend::Setup => lerp_color(setup, target, alpha),
        MixBlend::First | MixBlend::Replace | MixBlend::Add => {
            lerp_color(slot.color, target, alpha)
        }
    };
}

pub(crate) fn apply_slot_rgb(
    timeline: &RgbTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(slot) = skeleton.slots.get_mut(timeline.slot_index) else {
        return;
    };
    let bone_active = skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false);
    if !bone_active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .slots
        .get(timeline.slot_index)
        .map(|s| s.color)
        .unwrap_or([1.0, 1.0, 1.0, 1.0]);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => slot.color = setup,
            MixBlend::First => {
                slot.color = lerp_color(slot.color, setup, alpha);
            }
            _ => {}
        }
        return;
    }

    let target = sample_rgb(&timeline.frames, time);
    if alpha == 1.0 {
        slot.color[0] = target[0];
        slot.color[1] = target[1];
        slot.color[2] = target[2];
        return;
    }

    if blend == MixBlend::Setup {
        slot.color[0] = setup[0];
        slot.color[1] = setup[1];
        slot.color[2] = setup[2];
    }

    slot.color[0] += (target[0] - slot.color[0]) * alpha;
    slot.color[1] += (target[1] - slot.color[1]) * alpha;
    slot.color[2] += (target[2] - slot.color[2]) * alpha;
}

pub(crate) fn apply_slot_alpha(
    timeline: &AlphaTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(slot) = skeleton.slots.get_mut(timeline.slot_index) else {
        return;
    };
    let bone_active = skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false);
    if !bone_active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup_alpha = skeleton
        .data
        .slots
        .get(timeline.slot_index)
        .map(|s| s.color[3])
        .unwrap_or(1.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => slot.color[3] = setup_alpha,
            MixBlend::First => {
                slot.color[3] += (setup_alpha - slot.color[3]) * alpha;
            }
            _ => {}
        }
        return;
    }

    let target = sample_alpha(&timeline.frames, time);
    if alpha == 1.0 {
        slot.color[3] = target;
        return;
    }

    if blend == MixBlend::Setup {
        slot.color[3] = setup_alpha;
    }
    slot.color[3] += (target - slot.color[3]) * alpha;
}

pub(crate) fn apply_slot_rgba2(
    timeline: &Rgba2Timeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(slot) = skeleton.slots.get_mut(timeline.slot_index) else {
        return;
    };
    let bone_active = skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false);
    if !bone_active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let (setup_light, setup_has_dark, setup_dark) = skeleton
        .data
        .slots
        .get(timeline.slot_index)
        .map(|s| (s.color, s.has_dark, s.dark_color))
        .unwrap_or(([1.0, 1.0, 1.0, 1.0], false, [0.0, 0.0, 0.0]));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                slot.color = setup_light;
                slot.has_dark = setup_has_dark;
                slot.dark_color = setup_dark;
            }
            MixBlend::First => {
                slot.color = lerp_color(slot.color, setup_light, alpha);
                slot.dark_color = lerp3(slot.dark_color, setup_dark, alpha);
            }
            _ => {}
        }
        return;
    }

    let (target_light, target_dark) = sample_rgba2(&timeline.frames, time);
    if alpha == 1.0 {
        slot.color = target_light;
        slot.has_dark = true;
        slot.dark_color = target_dark;
        return;
    }

    if blend == MixBlend::Setup {
        slot.color = setup_light;
        slot.has_dark = setup_has_dark;
        slot.dark_color = setup_dark;
    }

    slot.color = lerp_color(slot.color, target_light, alpha);
    slot.dark_color = lerp3(slot.dark_color, target_dark, alpha);
    slot.has_dark = true;
}

pub(crate) fn apply_slot_rgb2(
    timeline: &Rgb2Timeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(slot) = skeleton.slots.get_mut(timeline.slot_index) else {
        return;
    };
    let bone_active = skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false);
    if !bone_active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let (setup_light, setup_has_dark, setup_dark) = skeleton
        .data
        .slots
        .get(timeline.slot_index)
        .map(|s| (s.color, s.has_dark, s.dark_color))
        .unwrap_or(([1.0, 1.0, 1.0, 1.0], false, [0.0, 0.0, 0.0]));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                slot.color[0] = setup_light[0];
                slot.color[1] = setup_light[1];
                slot.color[2] = setup_light[2];
                slot.has_dark = setup_has_dark;
                slot.dark_color = setup_dark;
            }
            MixBlend::First => {
                let target = [setup_light[0], setup_light[1], setup_light[2]];
                let current = [slot.color[0], slot.color[1], slot.color[2]];
                let out = lerp3(current, target, alpha);
                slot.color[0] = out[0];
                slot.color[1] = out[1];
                slot.color[2] = out[2];
                slot.dark_color = lerp3(slot.dark_color, setup_dark, alpha);
            }
            _ => {}
        }
        return;
    }

    let (target_light, target_dark) = sample_rgb2(&timeline.frames, time);
    if alpha == 1.0 {
        slot.color[0] = target_light[0];
        slot.color[1] = target_light[1];
        slot.color[2] = target_light[2];
        slot.has_dark = true;
        slot.dark_color = target_dark;
        return;
    }

    if blend == MixBlend::Setup {
        slot.color[0] = setup_light[0];
        slot.color[1] = setup_light[1];
        slot.color[2] = setup_light[2];
        slot.has_dark = setup_has_dark;
        slot.dark_color = setup_dark;
    }

    let current = [slot.color[0], slot.color[1], slot.color[2]];
    let out = lerp3(current, target_light, alpha);
    slot.color[0] = out[0];
    slot.color[1] = out[1];
    slot.color[2] = out[2];
    slot.dark_color = lerp3(slot.dark_color, target_dark, alpha);
    slot.has_dark = true;
}

fn lerp_color(from: [f32; 4], to: [f32; 4], alpha: f32) -> [f32; 4] {
    let a = alpha.clamp(0.0, 1.0);
    [
        from[0] + (to[0] - from[0]) * a,
        from[1] + (to[1] - from[1]) * a,
        from[2] + (to[2] - from[2]) * a,
        from[3] + (to[3] - from[3]) * a,
    ]
}

fn lerp3(from: [f32; 3], to: [f32; 3], alpha: f32) -> [f32; 3] {
    let a = alpha.clamp(0.0, 1.0);
    [
        from[0] + (to[0] - from[0]) * a,
        from[1] + (to[1] - from[1]) * a,
        from[2] + (to[2] - from[2]) * a,
    ]
}

pub(crate) fn apply_attachment(
    timeline: &AttachmentTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    blend: MixBlend,
    attachments: bool,
    unkeyed_state: i32,
) {
    let Some(bone_index) = skeleton.slots.get(timeline.slot_index).map(|s| s.bone) else {
        return;
    };
    let bone_active = skeleton
        .bones
        .get(bone_index)
        .map(|b| b.active)
        .unwrap_or(false);
    if !bone_active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }
    if !attachments && blend == MixBlend::Replace {
        return;
    }

    if time < timeline.frames[0].time {
        if matches!(blend, MixBlend::Setup | MixBlend::First) {
            let setup_attachment = skeleton
                .data
                .slots
                .get(timeline.slot_index)
                .and_then(|s| s.attachment.as_deref())
                .map(|s| s.to_string());
            if setup_attachment.is_some() {
                set_attachment(
                    skeleton,
                    timeline.slot_index,
                    setup_attachment.as_deref(),
                    attachments,
                    unkeyed_state,
                );
            } else {
                set_attachment(
                    skeleton,
                    timeline.slot_index,
                    None,
                    attachments,
                    unkeyed_state,
                );
            }
        }
        if let Some(slot) = skeleton.slots.get_mut(timeline.slot_index)
            && slot.attachment_state <= unkeyed_state
        {
            slot.attachment_state = unkeyed_state + ANIMATION_STATE_SETUP;
        }
        return;
    }

    let frame_index = timeline
        .frames
        .partition_point(|f| f.time <= time)
        .saturating_sub(1);

    set_attachment(
        skeleton,
        timeline.slot_index,
        timeline.frames[frame_index].name.as_deref(),
        attachments,
        unkeyed_state,
    );
    if let Some(slot) = skeleton.slots.get_mut(timeline.slot_index)
        && slot.attachment_state <= unkeyed_state
    {
        slot.attachment_state = unkeyed_state + ANIMATION_STATE_SETUP;
    }
}

fn resolve_attachment_source_skin<'a>(
    skeleton: &'a Skeleton,
    slot_index: usize,
    name: &str,
) -> Option<&'a str> {
    let skin_name = skeleton.skin.as_deref();
    if let Some(skin_name) = skin_name {
        if let Some(skin) = skeleton.data.skin(skin_name)
            && skin.attachment(slot_index, name).is_some()
        {
            return Some(skin_name);
        }
        if skin_name != "default"
            && let Some(default_skin) = skeleton.data.skin("default")
            && default_skin.attachment(slot_index, name).is_some()
        {
            return Some("default");
        }
    } else if let Some(default_skin) = skeleton.data.skin("default")
        && default_skin.attachment(slot_index, name).is_some()
    {
        return Some("default");
    }
    None
}

fn set_attachment(
    skeleton: &mut Skeleton,
    slot_index: usize,
    name: Option<&str>,
    attachments: bool,
    unkeyed_state: i32,
) {
    fn attachment_timeline_key(
        skeleton: &Skeleton,
        slot_index: usize,
        source_skin: &str,
        key: &str,
    ) -> Option<(bool, String, String)> {
        let skin = skeleton.data.skin(source_skin)?;
        let att = skin.attachment(slot_index, key)?;
        match att {
            crate::AttachmentData::Mesh(m) => {
                Some((true, m.timeline_skin.clone(), m.timeline_attachment.clone()))
            }
            crate::AttachmentData::Path(_)
            | crate::AttachmentData::BoundingBox(_)
            | crate::AttachmentData::Clipping(_) => {
                Some((true, source_skin.to_string(), key.to_string()))
            }
            crate::AttachmentData::Region(_) | crate::AttachmentData::Point(_) => {
                Some((false, source_skin.to_string(), key.to_string()))
            }
        }
    }

    let (old_key, old_skin) = skeleton
        .slots
        .get(slot_index)
        .map(|slot| (slot.attachment.clone(), slot.attachment_skin.clone()))
        .unwrap_or((None, None));

    let (new_key, new_skin) = if let Some(name) = name {
        if let Some(source_skin) = resolve_attachment_source_skin(skeleton, slot_index, name) {
            (Some(name.to_string()), Some(source_skin.to_string()))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let clear_deform = if old_key == new_key && old_skin == new_skin {
        false
    } else {
        match (
            old_key
                .as_deref()
                .zip(old_skin.as_deref())
                .and_then(|(k, s)| attachment_timeline_key(skeleton, slot_index, s, k)),
            new_key
                .as_deref()
                .zip(new_skin.as_deref())
                .and_then(|(k, s)| attachment_timeline_key(skeleton, slot_index, s, k)),
        ) {
            (Some((old_vertex, old_skin, old_key)), Some((new_vertex, new_skin, new_key))) => {
                !(old_vertex && new_vertex && old_skin == new_skin && old_key == new_key)
            }
            _ => true,
        }
    };

    let Some(slot) = skeleton.slots.get_mut(slot_index) else {
        return;
    };

    if old_key == new_key && old_skin == new_skin {
        if attachments {
            slot.attachment_state = unkeyed_state + ANIMATION_STATE_CURRENT;
        }
        return;
    }

    if clear_deform {
        slot.deform.clear();
    }

    slot.attachment = new_key;
    slot.attachment_skin = new_skin;
    slot.sequence_index = -1;
    if attachments {
        slot.attachment_state = unkeyed_state + ANIMATION_STATE_CURRENT;
    }
}

pub(crate) fn apply_draw_order(
    timeline: &DrawOrderTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    from_current: bool,
    out: bool,
) {
    if out {
        if !from_current {
            skeleton.draw_order = (0..skeleton.slots.len()).collect::<Vec<_>>();
        }
        return;
    }

    if timeline.frames.is_empty() {
        return;
    }

    if time < timeline.frames[0].time {
        if !from_current {
            skeleton.draw_order = (0..skeleton.slots.len()).collect::<Vec<_>>();
        }
        return;
    }

    let frame_index = timeline
        .frames
        .partition_point(|f| f.time <= time)
        .saturating_sub(1);
    if let Some(order) = timeline.frames[frame_index]
        .draw_order_to_setup_index
        .as_ref()
    {
        if order.len() == skeleton.slots.len() {
            skeleton.draw_order.clone_from(order);
        }
    } else {
        skeleton.draw_order = (0..skeleton.slots.len()).collect::<Vec<_>>();
    }
}

pub(crate) fn apply_draw_order_folder(
    timeline: &DrawOrderFolderTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    from_current: bool,
    out: bool,
) {
    if timeline.slots.is_empty() || timeline.frames.is_empty() {
        return;
    }

    if out || time < timeline.frames[0].time {
        if !from_current {
            setup_draw_order_folder(timeline, skeleton);
        }
        return;
    }

    let frame_index = timeline
        .frames
        .partition_point(|f| f.time <= time)
        .saturating_sub(1);
    if let Some(draw_order) = timeline.frames[frame_index].folder_draw_order.as_ref() {
        apply_draw_order_folder_frame(timeline, skeleton, draw_order);
    } else {
        setup_draw_order_folder(timeline, skeleton);
    }
}

fn mix_blend_is_current(blend: MixBlend) -> bool {
    matches!(blend, MixBlend::Replace | MixBlend::Add)
}

fn setup_draw_order_folder(timeline: &DrawOrderFolderTimeline, skeleton: &mut Skeleton) {
    let mut found = 0usize;
    let done = timeline.slots.len();
    let in_folder = draw_order_folder_mask(timeline, skeleton.slots.len());
    for slot_index in &mut skeleton.draw_order {
        if in_folder.get(*slot_index).copied().unwrap_or(false) {
            *slot_index = timeline.slots[found];
            found += 1;
            if found == done {
                break;
            }
        }
    }
}

fn apply_draw_order_folder_frame(
    timeline: &DrawOrderFolderTimeline,
    skeleton: &mut Skeleton,
    draw_order: &[usize],
) {
    if draw_order.len() != timeline.slots.len() {
        return;
    }

    let mut found = 0usize;
    let done = timeline.slots.len();
    let in_folder = draw_order_folder_mask(timeline, skeleton.slots.len());
    for slot_index in &mut skeleton.draw_order {
        if in_folder.get(*slot_index).copied().unwrap_or(false) {
            let Some(&folder_index) = draw_order.get(found) else {
                return;
            };
            let Some(&setup_slot_index) = timeline.slots.get(folder_index) else {
                return;
            };
            *slot_index = setup_slot_index;
            found += 1;
            if found == done {
                break;
            }
        }
    }
}

fn draw_order_folder_mask(timeline: &DrawOrderFolderTimeline, slot_count: usize) -> Vec<bool> {
    let mut in_folder = vec![false; slot_count];
    for &slot in &timeline.slots {
        if let Some(v) = in_folder.get_mut(slot) {
            *v = true;
        }
    }
    in_folder
}

pub(crate) fn apply_deform(
    timeline: &DeformTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    if alpha <= 0.0 {
        return;
    }
    if timeline.frames.is_empty() || timeline.vertex_count == 0 {
        return;
    }

    apply_deform_to_slot(timeline, skeleton, timeline.slot_index, time, alpha, blend);
    for slot_index in deform_timeline_slots(timeline, skeleton) {
        apply_deform_to_slot(timeline, skeleton, slot_index, time, alpha, blend);
    }
}

fn deform_timeline_slots(timeline: &DeformTimeline, skeleton: &Skeleton) -> Vec<usize> {
    skeleton
        .data
        .skin(timeline.skin.as_str())
        .and_then(|skin| skin.attachment(timeline.slot_index, timeline.attachment.as_str()))
        .and_then(|attachment| match attachment {
            crate::AttachmentData::Mesh(mesh) => Some(mesh.timeline_slots.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn slot_matches_deform_timeline(
    timeline: &DeformTimeline,
    skeleton: &Skeleton,
    slot_index: usize,
) -> bool {
    let Some(slot) = skeleton.slots.get(slot_index) else {
        return false;
    };
    if !skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false)
    {
        return false;
    }
    let Some(slot_key) = slot.attachment.as_deref() else {
        return false;
    };
    let Some(slot_skin) = slot.attachment_skin.as_deref() else {
        return false;
    };

    skeleton
        .slot_attachment_data(slot_index)
        .map(|attachment| match attachment {
            crate::AttachmentData::Mesh(mesh) => {
                mesh.timeline_skin.as_str() == timeline.skin.as_str()
                    && mesh.timeline_attachment.as_str() == timeline.attachment.as_str()
            }
            _ => slot_skin == timeline.skin.as_str() && slot_key == timeline.attachment.as_str(),
        })
        .unwrap_or(false)
}

fn apply_deform_to_slot(
    timeline: &DeformTimeline,
    skeleton: &mut Skeleton,
    slot_index: usize,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    if !slot_matches_deform_timeline(timeline, skeleton, slot_index) {
        return;
    }

    let Some(slot) = skeleton.slots.get_mut(slot_index) else {
        return;
    };
    let mut blend = blend;
    if slot.deform.is_empty() {
        blend = MixBlend::Setup;
    }

    if time < timeline.frames[0].time {
        match blend {
            MixBlend::Setup => {
                slot.deform.clear();
            }
            MixBlend::First => {
                if alpha >= 1.0 {
                    slot.deform.clear();
                    return;
                }
                ensure_len_with_zeros(&mut slot.deform, timeline.vertex_count);
                if let Some(setup) = timeline.setup_vertices.as_ref() {
                    for (d, s) in slot.deform.iter_mut().zip(setup) {
                        *d += (*s - *d) * alpha;
                    }
                } else {
                    let m = 1.0 - alpha;
                    for d in &mut slot.deform {
                        *d *= m;
                    }
                }
            }
            MixBlend::Replace | MixBlend::Add => {}
        }
        return;
    }

    ensure_len_with_zeros(&mut slot.deform, timeline.vertex_count);

    let last_index = timeline.frames.len() - 1;
    if time >= timeline.frames[last_index].time {
        let last_vertices = &timeline.frames[last_index].vertices;
        apply_deform_vertices(
            &mut slot.deform,
            timeline.setup_vertices.as_deref(),
            last_vertices,
            alpha,
            blend,
        );
        return;
    }

    let frame_index = timeline
        .frames
        .partition_point(|f| f.time <= time)
        .saturating_sub(1);
    let prev = &timeline.frames[frame_index];
    let next = &timeline.frames[frame_index + 1];
    let denom = next.time - prev.time;
    let percent = if denom.abs() <= 1.0e-12 {
        0.0
    } else {
        curve_value(prev.curve, time, prev.time, 0.0, next.time, 1.0)
    };

    let mut mixed = vec![0.0f32; timeline.vertex_count];
    for (i, out) in mixed.iter_mut().enumerate().take(timeline.vertex_count) {
        let pv = prev.vertices.get(i).copied().unwrap_or(0.0);
        let nv = next.vertices.get(i).copied().unwrap_or(pv);
        *out = pv + (nv - pv) * percent;
    }

    apply_deform_vertices(
        &mut slot.deform,
        timeline.setup_vertices.as_deref(),
        &mixed,
        alpha,
        blend,
    );
}

pub(crate) fn apply_sequence_timeline(
    timeline: &crate::SequenceTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    if timeline.frames.is_empty() {
        return;
    }

    let timeline_slots = sequence_timeline_slots(timeline, skeleton);
    if !sequence_timeline_active(timeline, skeleton, &timeline_slots) {
        return;
    }

    if direction == MixDirection::Out {
        if matches!(blend, MixBlend::Setup | MixBlend::First) {
            setup_sequence_timeline_slots(timeline, skeleton, &timeline_slots);
        }
        return;
    }

    if time < timeline.frames[0].time {
        if matches!(blend, MixBlend::Setup | MixBlend::First) {
            setup_sequence_timeline_slots(timeline, skeleton, &timeline_slots);
        }
        return;
    }

    let Some(index) = sequence_timeline_index(timeline, skeleton, time) else {
        return;
    };
    apply_sequence_index_to_slot(timeline, skeleton, timeline.slot_index, index);
    for slot_index in timeline_slots {
        apply_sequence_index_to_slot(timeline, skeleton, slot_index, index);
    }
}

fn sequence_timeline_slots(timeline: &crate::SequenceTimeline, skeleton: &Skeleton) -> Vec<usize> {
    skeleton
        .data
        .skin(timeline.skin.as_str())
        .and_then(|skin| skin.attachment(timeline.slot_index, timeline.attachment.as_str()))
        .and_then(|attachment| match attachment {
            crate::AttachmentData::Mesh(mesh) => Some(mesh.timeline_slots.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn sequence_timeline_active(
    timeline: &crate::SequenceTimeline,
    skeleton: &Skeleton,
    timeline_slots: &[usize],
) -> bool {
    slot_matches_sequence_timeline(timeline, skeleton, timeline.slot_index)
        || timeline_slots
            .iter()
            .any(|&slot_index| slot_matches_sequence_timeline(timeline, skeleton, slot_index))
}

fn slot_matches_sequence_timeline(
    timeline: &crate::SequenceTimeline,
    skeleton: &Skeleton,
    slot_index: usize,
) -> bool {
    let Some(slot) = skeleton.slots.get(slot_index) else {
        return false;
    };
    if !skeleton
        .bones
        .get(slot.bone)
        .map(|b| b.active)
        .unwrap_or(false)
    {
        return false;
    }
    let Some(slot_key) = slot.attachment.as_deref() else {
        return false;
    };
    let Some(slot_skin) = slot.attachment_skin.as_deref() else {
        return false;
    };

    skeleton
        .slot_attachment_data(slot_index)
        .map(|attachment| match attachment {
            crate::AttachmentData::Mesh(mesh) => {
                mesh.timeline_skin.as_str() == timeline.skin.as_str()
                    && mesh.timeline_attachment.as_str() == timeline.attachment.as_str()
            }
            _ => slot_skin == timeline.skin.as_str() && slot_key == timeline.attachment.as_str(),
        })
        .unwrap_or(false)
}

fn setup_sequence_timeline_slots(
    timeline: &crate::SequenceTimeline,
    skeleton: &mut Skeleton,
    timeline_slots: &[usize],
) {
    setup_sequence_slot(timeline, skeleton, timeline.slot_index);
    for &slot_index in timeline_slots {
        setup_sequence_slot(timeline, skeleton, slot_index);
    }
}

fn setup_sequence_slot(
    timeline: &crate::SequenceTimeline,
    skeleton: &mut Skeleton,
    slot_index: usize,
) {
    if !slot_matches_sequence_timeline(timeline, skeleton, slot_index) {
        return;
    }
    if let Some(slot) = skeleton.slots.get_mut(slot_index) {
        slot.sequence_index = -1;
    }
}

fn sequence_timeline_index(
    timeline: &crate::SequenceTimeline,
    skeleton: &Skeleton,
    time: f32,
) -> Option<i32> {
    let sequence = skeleton
        .data
        .skin(timeline.skin.as_str())
        .and_then(|s| s.attachment(timeline.slot_index, timeline.attachment.as_str()))
        .and_then(|a| match a {
            crate::AttachmentData::Region(r) => r.sequence.as_ref(),
            crate::AttachmentData::Mesh(m) => m.sequence.as_ref(),
            _ => None,
        })?;

    let frame_index = timeline
        .frames
        .partition_point(|f| f.time <= time)
        .saturating_sub(1);
    let before = timeline.frames[frame_index].time;
    let mode = timeline.frames[frame_index].mode;
    let delay = timeline.frames[frame_index].delay;
    let mut index = timeline.frames[frame_index].index;

    let count = sequence.count as i32;
    if count <= 0 {
        return None;
    }

    if mode != crate::SequenceMode::Hold {
        let step = if delay > 0.0 {
            ((time - before) / delay + 0.0001) as i32
        } else {
            0
        };
        index = index.saturating_add(step);

        match mode {
            crate::SequenceMode::Hold => {}
            crate::SequenceMode::Once => {
                index = index.min(count - 1);
            }
            crate::SequenceMode::Loop => {
                index %= count;
            }
            crate::SequenceMode::PingPong => {
                let n = (count << 1) - 2;
                index = if n == 0 { 0 } else { index % n };
                if index >= count {
                    index = n - index;
                }
            }
            crate::SequenceMode::OnceReverse => {
                index = (count - 1 - index).max(0);
            }
            crate::SequenceMode::LoopReverse => {
                index = count - 1 - (index % count);
            }
            crate::SequenceMode::PingPongReverse => {
                let n = (count << 1) - 2;
                index = if n == 0 { 0 } else { (index + count - 1) % n };
                if index >= count {
                    index = n - index;
                }
            }
        }
    }

    Some(index)
}

fn apply_sequence_index_to_slot(
    timeline: &crate::SequenceTimeline,
    skeleton: &mut Skeleton,
    slot_index: usize,
    index: i32,
) {
    if !slot_matches_sequence_timeline(timeline, skeleton, slot_index) {
        return;
    }
    if let Some(slot) = skeleton.slots.get_mut(slot_index) {
        slot.sequence_index = index;
    }
}

fn ensure_len_with_zeros(buf: &mut Vec<f32>, len: usize) {
    if buf.len() != len {
        buf.clear();
        buf.resize(len, 0.0);
    }
}

fn apply_deform_vertices(
    deform: &mut [f32],
    setup: Option<&[f32]>,
    value: &[f32],
    alpha: f32,
    blend: MixBlend,
) {
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha >= 1.0 {
        match blend {
            MixBlend::Add => {
                if let Some(setup) = setup {
                    for (i, d) in deform.iter_mut().enumerate() {
                        let v = value.get(i).copied().unwrap_or(0.0);
                        let s = setup.get(i).copied().unwrap_or(0.0);
                        *d += v - s;
                    }
                } else {
                    for (i, d) in deform.iter_mut().enumerate() {
                        *d += value.get(i).copied().unwrap_or(0.0);
                    }
                }
            }
            _ => {
                for (i, d) in deform.iter_mut().enumerate() {
                    *d = value.get(i).copied().unwrap_or(0.0);
                }
            }
        }
        return;
    }

    match blend {
        MixBlend::Setup => {
            if let Some(setup) = setup {
                for (i, d) in deform.iter_mut().enumerate() {
                    let s = setup.get(i).copied().unwrap_or(0.0);
                    let v = value.get(i).copied().unwrap_or(0.0);
                    *d = s + (v - s) * alpha;
                }
            } else {
                for (i, d) in deform.iter_mut().enumerate() {
                    *d = value.get(i).copied().unwrap_or(0.0) * alpha;
                }
            }
        }
        MixBlend::First | MixBlend::Replace => {
            for (i, d) in deform.iter_mut().enumerate() {
                let v = value.get(i).copied().unwrap_or(0.0);
                *d += (v - *d) * alpha;
            }
        }
        MixBlend::Add => {
            if let Some(setup) = setup {
                for (i, d) in deform.iter_mut().enumerate() {
                    let v = value.get(i).copied().unwrap_or(0.0);
                    let s = setup.get(i).copied().unwrap_or(0.0);
                    *d += (v - s) * alpha;
                }
            } else {
                for (i, d) in deform.iter_mut().enumerate() {
                    *d += value.get(i).copied().unwrap_or(0.0) * alpha;
                }
            }
        }
    }
}

fn sign(value: f32) -> f32 {
    if value < 0.0 {
        -1.0
    } else if value > 0.0 {
        1.0
    } else {
        0.0
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_rotate_mixed(
    timeline: &RotateTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    state: &mut [f32],
    rotate_timeline_index: usize,
    first_frame: bool,
) {
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.rotation)
        .unwrap_or(0.0);

    let base = rotate_timeline_index * 2;
    if base + 1 >= state.len() {
        apply_rotate_with(timeline, skeleton, time, alpha, from, false);
        return;
    }

    if first_frame {
        state[base] = 0.0;
    }
    if alpha >= 1.0 {
        apply_rotate_with(timeline, skeleton, time, 1.0, from, false);
        return;
    }

    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }

    let (r1, r2) = if time < timeline.frames[0].time {
        match from {
            MixFrom::Setup => {
                bone.rotation = setup;
                return;
            }
            MixFrom::Current => return,
            MixFrom::First => (bone.rotation, setup),
        }
    } else {
        let r1 = if from == MixFrom::Setup {
            setup
        } else {
            bone.rotation
        };
        let r2 = setup + sample_rotate(&timeline.frames, time);
        (r1, r2)
    };

    let mut total;
    let mut diff = r2 - r1;
    diff -= ((diff / 360.0 - 0.5).ceil()) * 360.0;

    if diff == 0.0 {
        total = state[base];
    } else {
        let (last_total, last_diff) = if first_frame {
            (0.0, diff)
        } else {
            (state[base], state[base + 1])
        };

        let loops = last_total - (last_total % 360.0);
        total = diff + loops;

        let current = diff >= 0.0;
        let mut dir = last_total >= 0.0;

        if last_diff.abs() <= 90.0 && sign(last_diff) != sign(diff) {
            if (last_total - loops).abs() > 180.0 {
                total += 360.0 * sign(last_total);
                dir = current;
            } else if loops != 0.0 {
                total -= 360.0 * sign(last_total);
            } else {
                dir = current;
            }
        }

        if dir != current {
            total += 360.0 * sign(last_total);
        }
    }

    state[base] = total;
    state[base + 1] = diff;

    bone.rotation = r1 + total * alpha;
}

pub(crate) fn apply_rotate(
    timeline: &RotateTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    apply_rotate_with(
        timeline,
        skeleton,
        time,
        alpha,
        MixFrom::from_blend(blend),
        blend == MixBlend::Add,
    );
}

pub(crate) fn apply_rotate_with(
    timeline: &RotateTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
) {
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.rotation)
        .unwrap_or(0.0);

    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match from {
            MixFrom::Setup => bone.rotation = setup,
            MixFrom::First => {
                bone.rotation += (setup - bone.rotation) * alpha;
            }
            MixFrom::Current => {}
        }
        return;
    }

    let value = sample_rotate(&timeline.frames, time);
    if from == MixFrom::Setup {
        bone.rotation = setup + value * alpha;
    } else if add {
        bone.rotation += value * alpha;
    } else {
        bone.rotation += (value + setup - bone.rotation) * alpha;
    }
}

pub(crate) fn apply_translate(
    timeline: &TranslateTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.x, b.y))
        .unwrap_or((0.0, 0.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                bone.x = setup.0;
                bone.y = setup.1;
            }
            MixBlend::First => {
                bone.x += (setup.0 - bone.x) * alpha;
                bone.y += (setup.1 - bone.y) * alpha;
            }
            _ => {}
        }
        return;
    }

    let offset = sample_vec2(&timeline.frames, time);
    let target_x = setup.0 + offset.0;
    let target_y = setup.1 + offset.1;

    match blend {
        MixBlend::Setup => {
            bone.x = setup.0 + offset.0 * alpha;
            bone.y = setup.1 + offset.1 * alpha;
        }
        MixBlend::First | MixBlend::Replace => {
            if alpha >= 1.0 {
                bone.x = target_x;
                bone.y = target_y;
            } else if alpha > 0.0 {
                bone.x += (target_x - bone.x) * alpha;
                bone.y += (target_y - bone.y) * alpha;
            }
        }
        MixBlend::Add => {
            if alpha > 0.0 {
                bone.x += offset.0 * alpha;
                bone.y += offset.1 * alpha;
            }
        }
    }
}

pub(crate) fn apply_translate_x(
    timeline: &TranslateXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.x)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.x = setup,
            MixBlend::First => bone.x += (setup - bone.x) * alpha,
            _ => {}
        }
        return;
    }

    let offset = sample_float(&timeline.frames, time);
    let target = setup + offset;

    match blend {
        MixBlend::Setup => bone.x = setup + offset * alpha,
        MixBlend::First | MixBlend::Replace => {
            if alpha >= 1.0 {
                bone.x = target;
            } else if alpha > 0.0 {
                bone.x += (target - bone.x) * alpha;
            }
        }
        MixBlend::Add => {
            if alpha > 0.0 {
                bone.x += offset * alpha;
            }
        }
    }
}

pub(crate) fn apply_translate_y(
    timeline: &TranslateYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.y)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.y = setup,
            MixBlend::First => bone.y += (setup - bone.y) * alpha,
            _ => {}
        }
        return;
    }

    let offset = sample_float(&timeline.frames, time);
    let target = setup + offset;

    match blend {
        MixBlend::Setup => bone.y = setup + offset * alpha,
        MixBlend::First | MixBlend::Replace => {
            if alpha >= 1.0 {
                bone.y = target;
            } else if alpha > 0.0 {
                bone.y += (target - bone.y) * alpha;
            }
        }
        MixBlend::Add => {
            if alpha > 0.0 {
                bone.y += offset * alpha;
            }
        }
    }
}

pub(crate) fn apply_scale(
    timeline: &ScaleTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.scale_x, b.scale_y))
        .unwrap_or((1.0, 1.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                bone.scale_x = setup.0;
                bone.scale_y = setup.1;
            }
            MixBlend::First => {
                bone.scale_x += (setup.0 - bone.scale_x) * alpha;
                bone.scale_y += (setup.1 - bone.scale_y) * alpha;
            }
            _ => {}
        }
        return;
    }

    let mult = sample_vec2(&timeline.frames, time);
    let x = setup.0 * mult.0;
    let y = setup.1 * mult.1;

    if alpha >= 1.0 {
        match blend {
            MixBlend::Add => {
                bone.scale_x += x - setup.0;
                bone.scale_y += y - setup.1;
            }
            _ => {
                bone.scale_x = x;
                bone.scale_y = y;
            }
        }
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    match direction {
        MixDirection::Out => match blend {
            MixBlend::Setup => {
                let bx = setup.0;
                let by = setup.1;
                bone.scale_x = bx + (x.abs() * signum(bx) - bx) * alpha;
                bone.scale_y = by + (y.abs() * signum(by) - by) * alpha;
            }
            MixBlend::First | MixBlend::Replace => {
                let bx = bone.scale_x;
                let by = bone.scale_y;
                bone.scale_x = bx + (x.abs() * signum(bx) - bx) * alpha;
                bone.scale_y = by + (y.abs() * signum(by) - by) * alpha;
            }
            MixBlend::Add => {
                bone.scale_x += (x - setup.0) * alpha;
                bone.scale_y += (y - setup.1) * alpha;
            }
        },
        MixDirection::In => match blend {
            MixBlend::Setup => {
                let bx = setup.0.abs() * signum(x);
                let by = setup.1.abs() * signum(y);
                bone.scale_x = bx + (x - bx) * alpha;
                bone.scale_y = by + (y - by) * alpha;
            }
            MixBlend::First | MixBlend::Replace => {
                let bx = bone.scale_x.abs() * signum(x);
                let by = bone.scale_y.abs() * signum(y);
                bone.scale_x = bx + (x - bx) * alpha;
                bone.scale_y = by + (y - by) * alpha;
            }
            MixBlend::Add => {
                bone.scale_x += (x - setup.0) * alpha;
                bone.scale_y += (y - setup.1) * alpha;
            }
        },
    }
}

pub(crate) fn apply_scale_with(
    timeline: &ScaleTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.scale_x, b.scale_y))
        .unwrap_or((1.0, 1.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match from {
            MixFrom::Setup => {
                bone.scale_x = setup.0;
                bone.scale_y = setup.1;
            }
            MixFrom::First => {
                bone.scale_x += (setup.0 - bone.scale_x) * alpha;
                bone.scale_y += (setup.1 - bone.scale_y) * alpha;
            }
            MixFrom::Current => {}
        }
        return;
    }

    let mult = sample_vec2(&timeline.frames, time);
    let x = setup.0 * mult.0;
    let y = setup.1 * mult.1;

    if alpha >= 1.0 && !add {
        bone.scale_x = x;
        bone.scale_y = y;
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    let (mut bx, mut by) = match from {
        MixFrom::Setup => setup,
        MixFrom::Current | MixFrom::First => (bone.scale_x, bone.scale_y),
    };
    if add {
        bone.scale_x = bx + (x - setup.0) * alpha;
        bone.scale_y = by + (y - setup.1) * alpha;
    } else if direction == MixDirection::Out {
        bone.scale_x = bx + (x.abs() * signum(bx) - bx) * alpha;
        bone.scale_y = by + (y.abs() * signum(by) - by) * alpha;
    } else {
        bx = bx.abs() * signum(x);
        by = by.abs() * signum(y);
        bone.scale_x = bx + (x - bx) * alpha;
        bone.scale_y = by + (y - by) * alpha;
    }
}

pub(crate) fn apply_scale_x(
    timeline: &ScaleXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.scale_x)
        .unwrap_or(1.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.scale_x = setup,
            MixBlend::First => bone.scale_x += (setup - bone.scale_x) * alpha,
            _ => {}
        }
        return;
    }

    let value = sample_float(&timeline.frames, time) * setup;
    if alpha >= 1.0 {
        match blend {
            MixBlend::Add => bone.scale_x = bone.scale_x + value - setup,
            _ => bone.scale_x = value,
        }
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    match direction {
        MixDirection::Out => match blend {
            MixBlend::Setup => {
                let bx = setup;
                bone.scale_x = bx + (value.abs() * signum(bx) - bx) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let bx = bone.scale_x;
                bone.scale_x = bx + (value.abs() * signum(bx) - bx) * alpha;
                return;
            }
            _ => {}
        },
        MixDirection::In => match blend {
            MixBlend::Setup => {
                let s = setup.abs() * signum(value);
                bone.scale_x = s + (value - s) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let s = bone.scale_x.abs() * signum(value);
                bone.scale_x = s + (value - s) * alpha;
                return;
            }
            _ => {}
        },
    }

    bone.scale_x += (value - setup) * alpha;
}

pub(crate) fn apply_scale_x_with(
    timeline: &ScaleXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.scale_x)
        .unwrap_or(1.0);
    let current = bone.scale_x;
    bone.scale_x = scale_value(
        &timeline.frames,
        time,
        alpha,
        from,
        add,
        direction,
        (current, setup),
    );
}

pub(crate) fn apply_scale_y(
    timeline: &ScaleYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.scale_y)
        .unwrap_or(1.0);

    // NOTE: Matches spine-cpp: ScaleYTimeline uses `scaleX` as the "current" parameter for the
    // CurveTimeline1 scale helper.
    let current = bone.scale_x;

    let first_time = timeline.frames[0].time;
    if time < first_time {
        bone.scale_y = match blend {
            MixBlend::Setup => setup,
            MixBlend::First => current + (setup - current) * alpha,
            _ => current,
        };
        return;
    }

    let value = sample_float(&timeline.frames, time) * setup;
    if alpha >= 1.0 {
        bone.scale_y = match blend {
            MixBlend::Add => current + value - setup,
            _ => value,
        };
        return;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    match direction {
        MixDirection::Out => match blend {
            MixBlend::Setup => {
                let bx = setup;
                bone.scale_y = bx + (value.abs() * signum(bx) - bx) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                bone.scale_y = current + (value.abs() * signum(current) - current) * alpha;
                return;
            }
            _ => {}
        },
        MixDirection::In => match blend {
            MixBlend::Setup => {
                let s = setup.abs() * signum(value);
                bone.scale_y = s + (value - s) * alpha;
                return;
            }
            MixBlend::First | MixBlend::Replace => {
                let s = current.abs() * signum(value);
                bone.scale_y = s + (value - s) * alpha;
                return;
            }
            _ => {}
        },
    }

    bone.scale_y = current + (value - setup) * alpha;
}

pub(crate) fn apply_scale_y_with(
    timeline: &ScaleYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.scale_y)
        .unwrap_or(1.0);
    let current = bone.scale_y;
    bone.scale_y = scale_value(
        &timeline.frames,
        time,
        alpha,
        from,
        add,
        direction,
        (current, setup),
    );
}

fn scale_value(
    frames: &[crate::FloatFrame],
    time: f32,
    alpha: f32,
    from: MixFrom,
    add: bool,
    direction: MixDirection,
    current_and_setup: (f32, f32),
) -> f32 {
    let (current, setup) = current_and_setup;
    if time < frames[0].time {
        return before_first_absolute(from, alpha, current, setup);
    }

    let value = sample_float(frames, time) * setup;
    if alpha >= 1.0 && !add {
        return value;
    }

    fn signum(v: f32) -> f32 {
        if v > 0.0 {
            1.0
        } else if v < 0.0 {
            -1.0
        } else {
            0.0
        }
    }

    let mut base = match from {
        MixFrom::Setup => setup,
        MixFrom::Current | MixFrom::First => current,
    };
    if add {
        base + (value - setup) * alpha
    } else if direction == MixDirection::Out {
        base + (value.abs() * signum(base) - base) * alpha
    } else {
        base = base.abs() * signum(value);
        base + (value - base) * alpha
    }
}

pub(crate) fn apply_shear(
    timeline: &ShearTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| (b.shear_x, b.shear_y))
        .unwrap_or((0.0, 0.0));

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => {
                bone.shear_x = setup.0;
                bone.shear_y = setup.1;
            }
            MixBlend::First => {
                bone.shear_x += (setup.0 - bone.shear_x) * alpha;
                bone.shear_y += (setup.1 - bone.shear_y) * alpha;
            }
            _ => {}
        }
        return;
    }

    let offset = sample_vec2(&timeline.frames, time);
    let target_x = setup.0 + offset.0;
    let target_y = setup.1 + offset.1;

    match blend {
        MixBlend::Setup => {
            bone.shear_x = setup.0 + offset.0 * alpha;
            bone.shear_y = setup.1 + offset.1 * alpha;
        }
        MixBlend::First | MixBlend::Replace => {
            if alpha >= 1.0 {
                bone.shear_x = target_x;
                bone.shear_y = target_y;
            } else if alpha > 0.0 {
                bone.shear_x += (target_x - bone.shear_x) * alpha;
                bone.shear_y += (target_y - bone.shear_y) * alpha;
            }
        }
        MixBlend::Add => {
            if alpha > 0.0 {
                bone.shear_x += offset.0 * alpha;
                bone.shear_y += offset.1 * alpha;
            }
        }
    }
}

pub(crate) fn apply_shear_x(
    timeline: &ShearXTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.shear_x)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.shear_x = setup,
            MixBlend::First => bone.shear_x += (setup - bone.shear_x) * alpha,
            _ => {}
        }
        return;
    }

    let offset = sample_float(&timeline.frames, time);
    let target = setup + offset;

    match blend {
        MixBlend::Setup => bone.shear_x = setup + offset * alpha,
        MixBlend::First | MixBlend::Replace => {
            if alpha >= 1.0 {
                bone.shear_x = target;
            } else if alpha > 0.0 {
                bone.shear_x += (target - bone.shear_x) * alpha;
            }
        }
        MixBlend::Add => {
            if alpha > 0.0 {
                bone.shear_x += offset * alpha;
            }
        }
    }
}

pub(crate) fn apply_shear_y(
    timeline: &ShearYTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    alpha: f32,
    blend: MixBlend,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.shear_y)
        .unwrap_or(0.0);

    let first_time = timeline.frames[0].time;
    if time < first_time {
        match blend {
            MixBlend::Setup => bone.shear_y = setup,
            MixBlend::First => bone.shear_y += (setup - bone.shear_y) * alpha,
            _ => {}
        }
        return;
    }

    let offset = sample_float(&timeline.frames, time);
    let target = setup + offset;

    match blend {
        MixBlend::Setup => bone.shear_y = setup + offset * alpha,
        MixBlend::First | MixBlend::Replace => {
            if alpha >= 1.0 {
                bone.shear_y = target;
            } else if alpha > 0.0 {
                bone.shear_y += (target - bone.shear_y) * alpha;
            }
        }
        MixBlend::Add => {
            if alpha > 0.0 {
                bone.shear_y += offset * alpha;
            }
        }
    }
}

pub(crate) fn apply_inherit(
    timeline: &InheritTimeline,
    skeleton: &mut Skeleton,
    time: f32,
    blend: MixBlend,
    direction: MixDirection,
) {
    let Some(bone) = skeleton.bones.get_mut(timeline.bone_index) else {
        return;
    };
    if !bone.active {
        return;
    }
    if timeline.frames.is_empty() {
        return;
    }

    let setup = skeleton
        .data
        .bones
        .get(timeline.bone_index)
        .map(|b| b.inherit)
        .unwrap_or(crate::Inherit::Normal);

    if direction == MixDirection::Out {
        if matches!(blend, MixBlend::Setup | MixBlend::First) {
            bone.inherit = setup;
        }
        return;
    }

    let first_time = timeline.frames[0].time;
    if time < first_time {
        if matches!(blend, MixBlend::Setup | MixBlend::First) {
            bone.inherit = setup;
        }
        return;
    }

    let idx = timeline.frames.partition_point(|f| f.time <= time);
    let frame = timeline
        .frames
        .get(idx.saturating_sub(1))
        .unwrap_or(&timeline.frames[0]);
    bone.inherit = frame.inherit;
}

fn sample_rotate(frames: &[RotateFrame], time: f32) -> f32 {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        return frames[0].angle;
    }
    if index >= frames.len() {
        return frames[frames.len() - 1].angle;
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return next.angle;
    }
    curve_value(
        prev.curve, time, prev.time, prev.angle, next.time, next.angle,
    )
}

fn sample_color(frames: &[crate::ColorFrame], time: f32) -> [f32; 4] {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        return frames[0].color;
    }
    if index >= frames.len() {
        return frames[frames.len() - 1].color;
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return next.color;
    }
    [
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.color[0],
            next.time,
            next.color[0],
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.color[1],
            next.time,
            next.color[1],
        ),
        curve_value(
            prev.curve[2],
            time,
            prev.time,
            prev.color[2],
            next.time,
            next.color[2],
        ),
        curve_value(
            prev.curve[3],
            time,
            prev.time,
            prev.color[3],
            next.time,
            next.color[3],
        ),
    ]
}

fn sample_rgb(frames: &[crate::RgbFrame], time: f32) -> [f32; 3] {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        return frames[0].color;
    }
    if index >= frames.len() {
        return frames[frames.len() - 1].color;
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return next.color;
    }
    [
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.color[0],
            next.time,
            next.color[0],
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.color[1],
            next.time,
            next.color[1],
        ),
        curve_value(
            prev.curve[2],
            time,
            prev.time,
            prev.color[2],
            next.time,
            next.color[2],
        ),
    ]
}

fn sample_alpha(frames: &[crate::AlphaFrame], time: f32) -> f32 {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        return frames[0].alpha;
    }
    if index >= frames.len() {
        return frames[frames.len() - 1].alpha;
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return next.alpha;
    }
    curve_value(
        prev.curve, time, prev.time, prev.alpha, next.time, next.alpha,
    )
}

fn sample_rgba2(frames: &[crate::Rgba2Frame], time: f32) -> ([f32; 4], [f32; 3]) {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        let f = &frames[0];
        return (f.light, f.dark);
    }
    if index >= frames.len() {
        let f = &frames[frames.len() - 1];
        return (f.light, f.dark);
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return (next.light, next.dark);
    }

    let light = [
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.light[0],
            next.time,
            next.light[0],
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.light[1],
            next.time,
            next.light[1],
        ),
        curve_value(
            prev.curve[2],
            time,
            prev.time,
            prev.light[2],
            next.time,
            next.light[2],
        ),
        curve_value(
            prev.curve[3],
            time,
            prev.time,
            prev.light[3],
            next.time,
            next.light[3],
        ),
    ];
    let dark = [
        curve_value(
            prev.curve[4],
            time,
            prev.time,
            prev.dark[0],
            next.time,
            next.dark[0],
        ),
        curve_value(
            prev.curve[5],
            time,
            prev.time,
            prev.dark[1],
            next.time,
            next.dark[1],
        ),
        curve_value(
            prev.curve[6],
            time,
            prev.time,
            prev.dark[2],
            next.time,
            next.dark[2],
        ),
    ];
    (light, dark)
}

fn sample_rgb2(frames: &[crate::Rgb2Frame], time: f32) -> ([f32; 3], [f32; 3]) {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        let f = &frames[0];
        return (f.light, f.dark);
    }
    if index >= frames.len() {
        let f = &frames[frames.len() - 1];
        return (f.light, f.dark);
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return (next.light, next.dark);
    }

    let light = [
        curve_value(
            prev.curve[0],
            time,
            prev.time,
            prev.light[0],
            next.time,
            next.light[0],
        ),
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.light[1],
            next.time,
            next.light[1],
        ),
        curve_value(
            prev.curve[2],
            time,
            prev.time,
            prev.light[2],
            next.time,
            next.light[2],
        ),
    ];
    let dark = [
        curve_value(
            prev.curve[3],
            time,
            prev.time,
            prev.dark[0],
            next.time,
            next.dark[0],
        ),
        curve_value(
            prev.curve[4],
            time,
            prev.time,
            prev.dark[1],
            next.time,
            next.dark[1],
        ),
        curve_value(
            prev.curve[5],
            time,
            prev.time,
            prev.dark[2],
            next.time,
            next.dark[2],
        ),
    ];
    (light, dark)
}

fn sample_vec2(frames: &[Vec2Frame], time: f32) -> (f32, f32) {
    let index = frames.partition_point(|f| f.time <= time);
    if index == 0 {
        let f = &frames[0];
        return (f.x, f.y);
    }
    if index >= frames.len() {
        let f = &frames[frames.len() - 1];
        return (f.x, f.y);
    }
    let prev = &frames[index - 1];
    let next = &frames[index];
    let denom = next.time - prev.time;
    if denom.abs() <= 1.0e-12 {
        return (next.x, next.y);
    }
    (
        curve_value(prev.curve[0], time, prev.time, prev.x, next.time, next.x),
        curve_value(prev.curve[1], time, prev.time, prev.y, next.time, next.y),
    )
}

fn curve_value(curve: Curve, time: f32, time1: f32, value1: f32, time2: f32, value2: f32) -> f32 {
    let denom = time2 - time1;
    if denom.abs() <= 1.0e-12 {
        return value2;
    }

    match curve {
        Curve::Linear => {
            let t = (time - time1) / denom;
            value1 + (value2 - value1) * t
        }
        Curve::Stepped => value1,
        Curve::Bezier { cx1, cy1, cx2, cy2 } => {
            bezier_value(time, time1, value1, cx1, cy1, cx2, cy2, time2, value2)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bezier_value(
    time: f32,
    time1: f32,
    value1: f32,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
    time2: f32,
    value2: f32,
) -> f32 {
    const BEZIER_SIZE: usize = 18;

    let tmpx = (time1 - cx1 * 2.0 + cx2) * 0.03;
    let tmpy = (value1 - cy1 * 2.0 + cy2) * 0.03;
    let dddx = ((cx1 - cx2) * 3.0 - time1 + time2) * 0.006;
    let dddy = ((cy1 - cy2) * 3.0 - value1 + value2) * 0.006;
    let mut ddx = tmpx * 2.0 + dddx;
    let mut ddy = tmpy * 2.0 + dddy;
    let mut dx = (cx1 - time1) * 0.3 + tmpx + dddx * 0.16666667;
    let mut dy = (cy1 - value1) * 0.3 + tmpy + dddy * 0.16666667;

    let mut x = time1 + dx;
    let mut y = value1 + dy;

    let mut points = [0.0f32; BEZIER_SIZE];
    for i in (0..BEZIER_SIZE).step_by(2) {
        points[i] = x;
        points[i + 1] = y;
        dx += ddx;
        dy += ddy;
        ddx += dddx;
        ddy += dddy;
        x += dx;
        y += dy;
    }

    if points[0] > time {
        let x = time1;
        let y = value1;
        let denom = points[0] - x;
        if denom.abs() <= 1.0e-12 {
            return y;
        }
        return y + (time - x) / denom * (points[1] - y);
    }

    for i in (2..BEZIER_SIZE).step_by(2) {
        if points[i] >= time {
            let x = points[i - 2];
            let y = points[i - 1];
            let denom = points[i] - x;
            if denom.abs() <= 1.0e-12 {
                return y;
            }
            return y + (time - x) / denom * (points[i + 1] - y);
        }
    }

    let x = points[BEZIER_SIZE - 2];
    let y = points[BEZIER_SIZE - 1];
    let denom = time2 - x;
    if denom.abs() <= 1.0e-12 {
        return y;
    }
    y + (time - x) / denom * (value2 - y)
}
