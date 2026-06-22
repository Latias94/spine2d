use super::{Physics, Skeleton};

pub(super) fn apply(skeleton: &mut Skeleton, constraint_index: usize, physics: Physics) -> bool {
    const PI_2: f32 = std::f32::consts::PI * 2.0;
    const INV_PI_2: f32 = 1.0 / PI_2;

    let Some(constraint) = skeleton.physics_constraints.get_mut(constraint_index) else {
        return false;
    };
    if !constraint.active {
        return false;
    }
    let mix = constraint.mix;
    if mix == 0.0 {
        return false;
    }

    let Some(data) = skeleton.data.physics_constraints.get(constraint.data_index) else {
        return false;
    };
    let bone_index = constraint.bone;
    if bone_index >= skeleton.bones.len() {
        return false;
    }

    let x = data.x > 0.0;
    let y = data.y > 0.0;
    let rotate_or_shear_x = data.rotate > 0.0 || data.shear_x > 0.0;
    let scale_x = data.scale_x > 0.0;

    let l = skeleton
        .data
        .bones
        .get(bone_index)
        .map(|b| b.length)
        .unwrap_or(0.0);

    let mut z = 0.0f32;

    let mut physics_mode = physics;
    if matches!(physics_mode, Physics::Reset) {
        constraint.reset_with_time(skeleton.time);
        physics_mode = Physics::Update;
    }

    match physics_mode {
        Physics::None => return false,
        Physics::Update => {
            let delta = (skeleton.time - constraint.last_time).max(0.0);
            let aa = constraint.remaining;
            constraint.remaining += delta;
            constraint.last_time = skeleton.time;

            let (mut bx, mut by) = {
                let bone = &skeleton.bones[bone_index];
                (bone.world_x, bone.world_y)
            };

            if constraint.reset {
                constraint.reset = false;
                constraint.ux = bx;
                constraint.uy = by;
            } else {
                let remaining0 = constraint.remaining;
                let inertia = constraint.inertia;
                let step = data.step;
                let reference_scale = skeleton.data.reference_scale;

                let mut qx = data.limit * delta;
                let qy = qx * skeleton.scale_y.abs();
                qx *= skeleton.scale_x.abs();

                let mut d = -1.0f32;
                let mut m = 0.0f32;
                let mut e = 0.0f32;

                // X/Y translation.
                let mut a = remaining0;
                if x || y {
                    if x {
                        let u = (constraint.ux - bx) * inertia;
                        constraint.x_offset += if u > qx {
                            qx
                        } else if u < -qx {
                            -qx
                        } else {
                            u
                        };
                        constraint.ux = bx;
                    }
                    if y {
                        let u = (constraint.uy - by) * inertia;
                        constraint.y_offset += if u > qy {
                            qy
                        } else if u < -qy {
                            -qy
                        } else {
                            u
                        };
                        constraint.uy = by;
                    }

                    if a >= step {
                        let xs = constraint.x_offset;
                        let ys = constraint.y_offset;

                        d = constraint.damping.powf(60.0 * step);
                        m = step * constraint.mass_inverse;
                        e = constraint.strength;

                        let w = reference_scale * constraint.wind;
                        let g = reference_scale * constraint.gravity;
                        let ax = (w * skeleton.wind_x + g * skeleton.gravity_x) * skeleton.scale_x;
                        let ay = (w * skeleton.wind_y + g * skeleton.gravity_y) * skeleton.scale_y;

                        while a >= step {
                            if x {
                                constraint.x_velocity += (ax - constraint.x_offset * e) * m;
                                constraint.x_offset += constraint.x_velocity * step;
                                constraint.x_velocity *= d;
                            }
                            if y {
                                constraint.y_velocity -= (ay + constraint.y_offset * e) * m;
                                constraint.y_offset += constraint.y_velocity * step;
                                constraint.y_velocity *= d;
                            }
                            a -= step;
                        }

                        constraint.x_lag = constraint.x_offset - xs;
                        constraint.y_lag = constraint.y_offset - ys;
                    }

                    if x {
                        z = (1.0 - a / step).max(0.0);
                        bx += (constraint.x_offset - constraint.x_lag * z) * mix * data.x;
                    }
                    if y {
                        z = (1.0 - a / step).max(0.0);
                        by += (constraint.y_offset - constraint.y_lag * z) * mix * data.y;
                    }
                }

                // Rotation/shear/scale.
                if rotate_or_shear_x || scale_x {
                    let (bone_a, bone_c) = {
                        let bone = &skeleton.bones[bone_index];
                        (bone.a, bone.c)
                    };
                    let ca = bone_c.atan2(bone_a);

                    let mut ccos;
                    let mut ssin;
                    let mut mr = 0.0f32;

                    let mut dx = constraint.cx - bx;
                    let mut dy = constraint.cy - by;
                    if dx > qx {
                        dx = qx;
                    } else if dx < -qx {
                        dx = -qx;
                    }
                    if dy > qy {
                        dy = qy;
                    } else if dy < -qy {
                        dy = -qy;
                    }

                    if rotate_or_shear_x {
                        mr = (data.rotate + data.shear_x) * mix;
                        let z0 = constraint.rotate_lag * (1.0 - aa / step).max(0.0);
                        let r = (dy + constraint.ty).atan2(dx + constraint.tx)
                            - ca
                            - (constraint.rotate_offset - z0) * mr;
                        constraint.rotate_offset +=
                            (r - ((r * INV_PI_2 - 0.5).ceil()) * PI_2) * inertia;
                        let r = (constraint.rotate_offset - z0) * mr + ca;
                        ccos = r.cos();
                        ssin = r.sin();
                        if scale_x {
                            let world_scale_x = (bone_a * bone_a + bone_c * bone_c).sqrt();
                            let r = l * world_scale_x;
                            if r > 0.0 {
                                constraint.scale_offset += (dx * ccos + dy * ssin) * inertia / r;
                            }
                        }
                    } else {
                        ccos = ca.cos();
                        ssin = ca.sin();
                        let world_scale_x = (bone_a * bone_a + bone_c * bone_c).sqrt();
                        let r =
                            l * world_scale_x - constraint.scale_lag * (1.0 - aa / step).max(0.0);
                        if r > 0.0 {
                            constraint.scale_offset += (dx * ccos + dy * ssin) * inertia / r;
                        }
                    }

                    let mut a = remaining0;
                    if a >= step {
                        if d < 0.0 {
                            d = constraint.damping.powf(60.0 * step);
                            m = step * constraint.mass_inverse;
                            e = constraint.strength;
                        }

                        let ax = constraint.wind * skeleton.wind_x
                            + constraint.gravity * skeleton.gravity_x;
                        let ay = constraint.wind * skeleton.wind_y
                            + constraint.gravity * skeleton.gravity_y;
                        let h = if reference_scale.abs() > 1.0e-12 {
                            l / reference_scale
                        } else {
                            0.0
                        };
                        let rs = constraint.rotate_offset;
                        let ss = constraint.scale_offset;
                        loop {
                            a -= step;
                            if scale_x {
                                constraint.scale_velocity +=
                                    (ax * ccos - ay * ssin - constraint.scale_offset * e) * m;
                                constraint.scale_offset += constraint.scale_velocity * step;
                                constraint.scale_velocity *= d;
                            }
                            if rotate_or_shear_x {
                                constraint.rotate_velocity -= ((ax * ssin + ay * ccos) * h
                                    + constraint.rotate_offset * e)
                                    * m;
                                constraint.rotate_offset += constraint.rotate_velocity * step;
                                constraint.rotate_velocity *= d;
                                if a < step {
                                    break;
                                }
                                let r = constraint.rotate_offset * mr + ca;
                                ccos = r.cos();
                                ssin = r.sin();
                            } else if a < step {
                                break;
                            }
                        }

                        constraint.rotate_lag = constraint.rotate_offset - rs;
                        constraint.scale_lag = constraint.scale_offset - ss;
                    }

                    z = (1.0 - a / step).max(0.0);
                    constraint.remaining = a;
                } else {
                    constraint.remaining = a;
                }

                {
                    let bone = &mut skeleton.bones[bone_index];
                    bone.world_x = bx;
                    bone.world_y = by;
                }
            }

            constraint.cx = skeleton.bones[bone_index].world_x;
            constraint.cy = skeleton.bones[bone_index].world_y;
        }
        Physics::Pose => {
            z = (1.0 - constraint.remaining / data.step).max(0.0);
            if x {
                skeleton.bones[bone_index].world_x +=
                    (constraint.x_offset - constraint.x_lag * z) * mix * data.x;
            }
            if y {
                skeleton.bones[bone_index].world_y +=
                    (constraint.y_offset - constraint.y_lag * z) * mix * data.y;
            }
        }
        Physics::Reset => unreachable!(),
    }

    if rotate_or_shear_x {
        let mut o = (constraint.rotate_offset - constraint.rotate_lag * z) * mix;
        if data.shear_x > 0.0 {
            let mut r = 0.0;
            if data.rotate > 0.0 {
                r = o * data.rotate;
                let s = r.sin();
                let c = r.cos();
                let b = skeleton.bones[bone_index].b;
                let d = skeleton.bones[bone_index].d;
                skeleton.bones[bone_index].b = c * b - s * d;
                skeleton.bones[bone_index].d = s * b + c * d;
            }
            r += o * data.shear_x;
            let s = r.sin();
            let c = r.cos();
            let a = skeleton.bones[bone_index].a;
            let c0 = skeleton.bones[bone_index].c;
            skeleton.bones[bone_index].a = c * a - s * c0;
            skeleton.bones[bone_index].c = s * a + c * c0;
        } else {
            o *= data.rotate;
            let s = o.sin();
            let c = o.cos();
            let a = skeleton.bones[bone_index].a;
            let c0 = skeleton.bones[bone_index].c;
            skeleton.bones[bone_index].a = c * a - s * c0;
            skeleton.bones[bone_index].c = s * a + c * c0;
            let b = skeleton.bones[bone_index].b;
            let d = skeleton.bones[bone_index].d;
            skeleton.bones[bone_index].b = c * b - s * d;
            skeleton.bones[bone_index].d = s * b + c * d;
        }
    }

    if scale_x {
        let s = 1.0 + (constraint.scale_offset - constraint.scale_lag * z) * mix * data.scale_x;
        skeleton.bones[bone_index].a *= s;
        skeleton.bones[bone_index].c *= s;
        match constraint.scale_y_mode {
            crate::ScaleYMode::Uniform => {
                skeleton.bones[bone_index].b *= s;
                skeleton.bones[bone_index].d *= s;
            }
            crate::ScaleYMode::Volume => {
                let sy = s.abs();
                let sy = if sy >= 0.7 {
                    1.0 / sy
                } else {
                    4.0 - 3.67347 * sy
                };
                skeleton.bones[bone_index].b *= sy;
                skeleton.bones[bone_index].d *= sy;
            }
            crate::ScaleYMode::None => {}
        }
    }

    if !matches!(physics_mode, Physics::Pose) {
        constraint.tx = l * skeleton.bones[bone_index].a;
        constraint.ty = l * skeleton.bones[bone_index].c;
    }

    skeleton.bone_modify_world(bone_index);
    true
}
