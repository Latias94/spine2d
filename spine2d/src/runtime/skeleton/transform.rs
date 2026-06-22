use super::{Skeleton, atan2_degrees};

#[derive(Clone, Debug)]
pub struct TransformConstraint {
    pub(crate) data_index: usize,
    pub(crate) bones: Vec<usize>,
    pub(crate) source: usize,
    pub(crate) mix_rotate: f32,
    pub(crate) mix_x: f32,
    pub(crate) mix_y: f32,
    pub(crate) mix_scale_x: f32,
    pub(crate) mix_scale_y: f32,
    pub(crate) mix_shear_y: f32,
    pub(crate) active: bool,
}

impl TransformConstraint {
    pub fn data_index(&self) -> usize {
        self.data_index
    }

    pub fn bones(&self) -> &[usize] {
        &self.bones
    }

    pub fn bones_mut(&mut self) -> &mut Vec<usize> {
        &mut self.bones
    }

    pub fn source(&self) -> usize {
        self.source
    }

    pub fn set_source(&mut self, source: usize) {
        self.source = source;
    }

    pub fn mix_rotate(&self) -> f32 {
        self.mix_rotate
    }

    pub fn set_mix_rotate(&mut self, mix_rotate: f32) {
        self.mix_rotate = mix_rotate;
    }

    pub fn mix_x(&self) -> f32 {
        self.mix_x
    }

    pub fn set_mix_x(&mut self, mix_x: f32) {
        self.mix_x = mix_x;
    }

    pub fn mix_y(&self) -> f32 {
        self.mix_y
    }

    pub fn set_mix_y(&mut self, mix_y: f32) {
        self.mix_y = mix_y;
    }

    pub fn mix_scale_x(&self) -> f32 {
        self.mix_scale_x
    }

    pub fn set_mix_scale_x(&mut self, mix_scale_x: f32) {
        self.mix_scale_x = mix_scale_x;
    }

    pub fn mix_scale_y(&self) -> f32 {
        self.mix_scale_y
    }

    pub fn set_mix_scale_y(&mut self, mix_scale_y: f32) {
        self.mix_scale_y = mix_scale_y;
    }

    pub fn mix_shear_y(&self) -> f32 {
        self.mix_shear_y
    }

    pub fn set_mix_shear_y(&mut self, mix_shear_y: f32) {
        self.mix_shear_y = mix_shear_y;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

pub(super) fn apply(skeleton: &mut Skeleton, constraint_index: usize) -> bool {
    const PI: f32 = std::f32::consts::PI;
    const PI2: f32 = 2.0 * std::f32::consts::PI;
    const DEG_RAD: f32 = std::f32::consts::PI / 180.0;

    let Some(constraint) = skeleton
        .transform_constraints
        .get(constraint_index)
        .cloned()
    else {
        return false;
    };
    let data_index = constraint.data_index;
    let (local_source, local_target, additive, clamp, offsets) = {
        let Some(data) = skeleton.data.transform_constraints.get(data_index) else {
            return false;
        };
        if data.properties.is_empty() {
            return false;
        }
        (
            data.local_source,
            data.local_target,
            data.additive,
            data.clamp,
            data.offsets,
        )
    };

    if constraint.mix_rotate == 0.0
        && constraint.mix_x == 0.0
        && constraint.mix_y == 0.0
        && constraint.mix_scale_x == 0.0
        && constraint.mix_scale_y == 0.0
        && constraint.mix_shear_y == 0.0
    {
        return false;
    }

    if constraint.source >= skeleton.bones.len() {
        return false;
    }

    if local_source && skeleton.bones[constraint.source].local_epoch == skeleton.update_epoch {
        skeleton.update_applied_transform(constraint.source);
        skeleton.bones[constraint.source].local_epoch = 0;
    }

    let (source_ax, source_ay, source_rot, source_scale_x, source_scale_y, source_shear_y) = {
        let b = &skeleton.bones[constraint.source];
        (b.ax, b.ay, b.arotation, b.ascale_x, b.ascale_y, b.ashear_y)
    };
    let (source_a, source_b, source_c, source_d, source_wx, source_wy) = {
        let b = &skeleton.bones[constraint.source];
        (b.a, b.b, b.c, b.d, b.world_x, b.world_y)
    };

    let sx = skeleton.scale_x;
    let sy = skeleton.scale_y;

    if local_target {
        for &bone_index in &constraint.bones {
            if bone_index >= skeleton.bones.len() {
                continue;
            }
            if !skeleton.bones[bone_index].active {
                continue;
            }
            skeleton.bone_modify_local(bone_index);
        }
    }

    let properties = skeleton
        .data
        .transform_constraints
        .get(data_index)
        .map(|d| d.properties.clone())
        .unwrap_or_default();

    let mut applied = false;
    for &bone_index in &constraint.bones {
        if bone_index >= skeleton.bones.len() {
            continue;
        }
        if !skeleton.bones[bone_index].active {
            continue;
        }
        if !local_target {
            skeleton.bone_modify_world(bone_index);
        }

        for from in &properties {
            let from_value = match from.property {
                crate::TransformProperty::Rotate => {
                    if local_source {
                        source_rot + offsets[crate::TransformProperty::Rotate.index()]
                    } else {
                        let value = atan2_degrees(source_c / sy, source_a / sx);
                        let det = source_a * source_d - source_b * source_c;
                        let sign = if det * sx * sy > 0.0 { 1.0 } else { -1.0 };
                        let mut v =
                            value + offsets[crate::TransformProperty::Rotate.index()] * sign;
                        if v < 0.0 {
                            v += 360.0;
                        }
                        v
                    }
                }
                crate::TransformProperty::X => {
                    if local_source {
                        source_ax + offsets[crate::TransformProperty::X.index()]
                    } else {
                        (offsets[crate::TransformProperty::X.index()] * source_a
                            + offsets[crate::TransformProperty::Y.index()] * source_b
                            + source_wx)
                            / sx
                    }
                }
                crate::TransformProperty::Y => {
                    if local_source {
                        source_ay + offsets[crate::TransformProperty::Y.index()]
                    } else {
                        (offsets[crate::TransformProperty::X.index()] * source_c
                            + offsets[crate::TransformProperty::Y.index()] * source_d
                            + source_wy)
                            / sy
                    }
                }
                crate::TransformProperty::ScaleX => {
                    if local_source {
                        source_scale_x + offsets[crate::TransformProperty::ScaleX.index()]
                    } else {
                        let a = source_a / sx;
                        let c0 = source_c / sy;
                        (a * a + c0 * c0).sqrt() + offsets[crate::TransformProperty::ScaleX.index()]
                    }
                }
                crate::TransformProperty::ScaleY => {
                    if local_source {
                        source_scale_y + offsets[crate::TransformProperty::ScaleY.index()]
                    } else {
                        let b = source_b / sx;
                        let d = source_d / sy;
                        (b * b + d * d).sqrt() + offsets[crate::TransformProperty::ScaleY.index()]
                    }
                }
                crate::TransformProperty::ShearY => {
                    if local_source {
                        source_shear_y + offsets[crate::TransformProperty::ShearY.index()]
                    } else {
                        let ix = 1.0 / sx;
                        let iy = 1.0 / sy;
                        ((source_d * iy).atan2(source_b * ix)
                            - (source_c * iy).atan2(source_a * ix))
                        .to_degrees()
                            - 90.0
                            + offsets[crate::TransformProperty::ShearY.index()]
                    }
                }
            } - from.offset;

            for to in &from.to {
                let mix = match to.property {
                    crate::TransformProperty::Rotate => constraint.mix_rotate,
                    crate::TransformProperty::X => constraint.mix_x,
                    crate::TransformProperty::Y => constraint.mix_y,
                    crate::TransformProperty::ScaleX => constraint.mix_scale_x,
                    crate::TransformProperty::ScaleY => constraint.mix_scale_y,
                    crate::TransformProperty::ShearY => constraint.mix_shear_y,
                };
                if mix == 0.0 {
                    continue;
                }

                let mut value = to.offset + from_value * to.scale;
                if clamp {
                    value = clamp_value(value, to.offset, to.max);
                }

                if local_target {
                    let bone = &mut skeleton.bones[bone_index];
                    match to.property {
                        crate::TransformProperty::Rotate => {
                            bone.arotation += (if additive {
                                value
                            } else {
                                value - bone.arotation
                            }) * mix;
                        }
                        crate::TransformProperty::X => {
                            bone.ax += (if additive { value } else { value - bone.ax }) * mix;
                        }
                        crate::TransformProperty::Y => {
                            bone.ay += (if additive { value } else { value - bone.ay }) * mix;
                        }
                        crate::TransformProperty::ScaleX => {
                            if additive {
                                bone.ascale_x *= 1.0 + (value - 1.0) * mix;
                            } else if bone.ascale_x != 0.0 {
                                bone.ascale_x += (value - bone.ascale_x) * mix;
                            }
                        }
                        crate::TransformProperty::ScaleY => {
                            if additive {
                                bone.ascale_y *= 1.0 + (value - 1.0) * mix;
                            } else if bone.ascale_y != 0.0 {
                                bone.ascale_y += (value - bone.ascale_y) * mix;
                            }
                        }
                        crate::TransformProperty::ShearY => {
                            if !additive {
                                value -= bone.ashear_y;
                            }
                            bone.ashear_y += value * mix;
                        }
                    }
                } else {
                    let bone = &mut skeleton.bones[bone_index];
                    match to.property {
                        crate::TransformProperty::Rotate => {
                            let ix = 1.0 / sx;
                            let iy = 1.0 / sy;
                            let a = bone.a * ix;
                            let b = bone.b * ix;
                            let c0 = bone.c * iy;
                            let d = bone.d * iy;
                            let mut r = value * DEG_RAD;
                            if !additive {
                                r -= c0.atan2(a);
                            }
                            if r > PI {
                                r -= PI2;
                            } else if r < -PI {
                                r += PI2;
                            }
                            r *= mix;
                            let cos = r.cos();
                            let sin = r.sin();
                            bone.a = (cos * a - sin * c0) * sx;
                            bone.b = (cos * b - sin * d) * sx;
                            bone.c = (sin * a + cos * c0) * sy;
                            bone.d = (sin * b + cos * d) * sy;
                        }
                        crate::TransformProperty::X => {
                            if !additive {
                                value -= bone.world_x / sx;
                            }
                            bone.world_x += value * mix * sx;
                        }
                        crate::TransformProperty::Y => {
                            if !additive {
                                value -= bone.world_y / sy;
                            }
                            bone.world_y += value * mix * sy;
                        }
                        crate::TransformProperty::ScaleX => {
                            if additive {
                                let s = 1.0 + (value - 1.0) * mix;
                                bone.a *= s;
                                bone.c *= s;
                            } else {
                                let a = bone.a / sx;
                                let c0 = bone.c / sy;
                                let s = (a * a + c0 * c0).sqrt();
                                if s != 0.0 {
                                    let s = 1.0 + (value - s) * mix / s;
                                    bone.a *= s;
                                    bone.c *= s;
                                }
                            }
                        }
                        crate::TransformProperty::ScaleY => {
                            if additive {
                                let s = 1.0 + (value - 1.0) * mix;
                                bone.b *= s;
                                bone.d *= s;
                            } else {
                                let b = bone.b / sx;
                                let d = bone.d / sy;
                                let s = (b * b + d * d).sqrt();
                                if s != 0.0 {
                                    let s = 1.0 + (value - s) * mix / s;
                                    bone.b *= s;
                                    bone.d *= s;
                                }
                            }
                        }
                        crate::TransformProperty::ShearY => {
                            let b0 = bone.b / sx;
                            let d0 = bone.d / sy;
                            let by = d0.atan2(b0);
                            let mut r = (value + 90.0) * DEG_RAD;
                            if additive {
                                r -= PI / 2.0;
                            } else {
                                r -= by - (bone.c / sy).atan2(bone.a / sx);
                                if r > PI {
                                    r -= PI2;
                                } else if r < -PI {
                                    r += PI2;
                                }
                            }
                            r = by + r * mix;
                            let s = (b0 * b0 + d0 * d0).sqrt();
                            bone.b = r.cos() * s * sx;
                            bone.d = r.sin() * s * sy;
                        }
                    }
                }
                applied = true;
            }
        }
    }

    applied
}

fn clamp_value(v: f32, a: f32, b: f32) -> f32 {
    let (min, max) = if a <= b { (a, b) } else { (b, a) };
    v.clamp(min, max)
}
