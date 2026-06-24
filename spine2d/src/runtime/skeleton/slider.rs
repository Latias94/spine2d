use super::{Skeleton, atan2_degrees};

#[derive(Clone, Debug)]
pub struct SliderConstraint {
    pub(crate) data_index: usize,
    pub(crate) time: f32,
    pub(crate) mix: f32,
    pub(crate) active: bool,
    pub(super) animation_bones: Vec<usize>,
}

impl SliderConstraint {
    pub fn data_index(&self) -> usize {
        self.data_index
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn mix(&self) -> f32 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

pub(super) fn apply(skeleton: &mut Skeleton, constraint_index: usize) -> bool {
    if constraint_index >= skeleton.slider_constraints.len() {
        return false;
    }

    let (data_index, mix, pose_time) = {
        let c = &skeleton.slider_constraints[constraint_index];
        (c.data_index, c.mix, c.time)
    };
    if mix == 0.0 {
        return false;
    }

    let (looped, additive, local, bone, property, property_from, to, scale, animation_index) = {
        let Some(data) = skeleton.data.slider_constraints.get(data_index) else {
            return false;
        };
        let Some(animation_index) = data.animation else {
            return false;
        };
        (
            data.looped,
            data.additive,
            data.local,
            data.bone,
            data.property,
            data.property_from,
            data.to,
            data.scale,
            animation_index,
        )
    };

    // Avoid borrowing `skeleton.data` across `&mut skeleton` calls during constraint evaluation.
    let data = std::sync::Arc::clone(&skeleton.data);
    let Some(animation) = data.animations.get(animation_index) else {
        return false;
    };
    let animation_duration = animation.duration;

    let mut time_to_apply = pose_time;
    if let (Some(bone_index), Some(property)) = (bone, property) {
        let Some(bone) = skeleton.bones.get(bone_index) else {
            return false;
        };
        if !bone.active {
            return false;
        }

        if local {
            // Match upstream: `validateLocalTransform` on the applied pose before reading local
            // properties (local values may be stale after world-space constraints).
            if bone.local_epoch == skeleton.update_epoch {
                skeleton.update_applied_transform(bone_index);
                skeleton.bones[bone_index].local_epoch = 0;
            }
        }

        let property_value = match property {
            crate::TransformProperty::Rotate => {
                if local {
                    skeleton
                        .bones
                        .get(bone_index)
                        .map(|b| b.arotation)
                        .unwrap_or(0.0)
                } else {
                    let (a, b, c, d) = {
                        let bone = &skeleton.bones[bone_index];
                        (bone.a, bone.b, bone.c, bone.d)
                    };
                    let sx = skeleton.scale_x;
                    let sy = skeleton.scale_y;
                    let mut value = atan2_degrees(c / sy, a / sx);
                    if value < 0.0 {
                        value += 360.0;
                    }
                    // Offsets are always zero in Slider (matches spine-cpp's static `_offsets`).
                    let _ = (a * d - b * c) * sx * sy;
                    value
                }
            }
            crate::TransformProperty::X => {
                if local {
                    skeleton.bones.get(bone_index).map(|b| b.ax).unwrap_or(0.0)
                } else {
                    skeleton
                        .bones
                        .get(bone_index)
                        .map(|b| b.world_x / skeleton.scale_x)
                        .unwrap_or(0.0)
                }
            }
            crate::TransformProperty::Y => {
                if local {
                    skeleton.bones.get(bone_index).map(|b| b.ay).unwrap_or(0.0)
                } else {
                    skeleton
                        .bones
                        .get(bone_index)
                        .map(|b| b.world_y / skeleton.scale_y)
                        .unwrap_or(0.0)
                }
            }
            crate::TransformProperty::ScaleX => {
                if local {
                    skeleton
                        .bones
                        .get(bone_index)
                        .map(|b| b.ascale_x)
                        .unwrap_or(0.0)
                } else {
                    let (a, c) = {
                        let bone = &skeleton.bones[bone_index];
                        (bone.a / skeleton.scale_x, bone.c / skeleton.scale_y)
                    };
                    (a * a + c * c).sqrt()
                }
            }
            crate::TransformProperty::ScaleY => {
                if local {
                    skeleton
                        .bones
                        .get(bone_index)
                        .map(|b| b.ascale_y)
                        .unwrap_or(0.0)
                } else {
                    let (b, d) = {
                        let bone = &skeleton.bones[bone_index];
                        (bone.b / skeleton.scale_x, bone.d / skeleton.scale_y)
                    };
                    (b * b + d * d).sqrt()
                }
            }
            crate::TransformProperty::ShearY => {
                if local {
                    skeleton
                        .bones
                        .get(bone_index)
                        .map(|b| b.ashear_y)
                        .unwrap_or(0.0)
                } else {
                    let (a, b, c, d) = {
                        let bone = &skeleton.bones[bone_index];
                        (bone.a, bone.b, bone.c, bone.d)
                    };
                    let sx = skeleton.scale_x;
                    let sy = skeleton.scale_y;
                    ((d / sy).atan2(b / sx) - (c / sy).atan2(a / sx)).to_degrees() - 90.0
                }
            }
        };

        time_to_apply = to + (property_value - property_from) * scale;
        if looped {
            if animation_duration > 0.0 {
                time_to_apply = animation_duration + time_to_apply.rem_euclid(animation_duration);
            }
        } else if time_to_apply < 0.0 {
            time_to_apply = 0.0;
        }
    }

    let animation_bones =
        std::mem::take(&mut skeleton.slider_constraints[constraint_index].animation_bones);
    for &bone_index in &animation_bones {
        skeleton.bone_modify_local(bone_index);
    }

    crate::runtime::apply_animation_applied(
        animation,
        skeleton,
        time_to_apply,
        looped,
        mix,
        if additive {
            crate::runtime::MixBlend::Add
        } else {
            crate::runtime::MixBlend::Replace
        },
    );

    skeleton.slider_constraints[constraint_index].animation_bones = animation_bones;
    true
}
