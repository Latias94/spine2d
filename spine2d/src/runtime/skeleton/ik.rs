use super::{Skeleton, acos_f32, atan2_degrees, atan2_radians, cos_f32, sin_f32, sqrt_f32};

#[derive(Clone, Debug)]
pub struct IkConstraint {
    pub(crate) data_index: usize,
    pub(crate) bones: Vec<usize>,
    pub(crate) target: usize,
    pub(crate) scale_y_mode: crate::ScaleYMode,
    pub(crate) mix: f32,
    pub(crate) softness: f32,
    pub(crate) compress: bool,
    pub(crate) stretch: bool,
    pub(crate) bend_direction: i32,
    pub(crate) active: bool,
}

impl IkConstraint {
    pub fn get_bones(&self) -> &[usize] {
        &self.bones
    }

    pub fn get_bones_mut(&mut self) -> &mut Vec<usize> {
        &mut self.bones
    }

    pub fn get_target(&self) -> usize {
        self.target
    }

    pub fn set_target(&mut self, target: usize) {
        self.target = target;
    }

    pub fn get_scale_y_mode(&self) -> crate::ScaleYMode {
        self.scale_y_mode
    }

    pub fn set_scale_y_mode(&mut self, scale_y_mode: crate::ScaleYMode) {
        self.scale_y_mode = scale_y_mode;
    }

    pub fn get_mix(&self) -> f32 {
        self.mix
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix;
    }

    pub fn get_softness(&self) -> f32 {
        self.softness
    }

    pub fn set_softness(&mut self, softness: f32) {
        self.softness = softness;
    }

    pub fn get_compress(&self) -> bool {
        self.compress
    }

    pub fn set_compress(&mut self, compress: bool) {
        self.compress = compress;
    }

    pub fn get_stretch(&self) -> bool {
        self.stretch
    }

    pub fn set_stretch(&mut self, stretch: bool) {
        self.stretch = stretch;
    }

    pub fn get_bend_direction(&self) -> i32 {
        self.bend_direction
    }

    pub fn set_bend_direction(&mut self, bend_direction: i32) {
        self.bend_direction = bend_direction;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

pub(super) fn apply(skeleton: &mut Skeleton, constraint_index: usize) -> bool {
    let Some(ik) = skeleton.ik_constraints.get(constraint_index).cloned() else {
        return false;
    };
    // spine-cpp does not clamp the IK mix; Add blending can intentionally push it beyond 1.
    // Keep behavior identical for strict runtime parity.
    let mix = ik.mix;
    if mix == 0.0 {
        return false;
    }

    let Some(target) = skeleton.bones.get(ik.target) else {
        return false;
    };
    let target_x = target.world_x;
    let target_y = target.world_y;
    match ik.bones.as_slice() {
        [bone] => {
            skeleton.bone_modify_local(*bone);
            apply_one(
                skeleton,
                *bone,
                target_x,
                target_y,
                ik.compress,
                ik.stretch,
                ik.scale_y_mode,
                mix,
            );
            true
        }
        [parent, child] => {
            skeleton.bone_modify_local(*parent);
            skeleton.bone_modify_local(*child);
            apply_two(
                skeleton,
                *parent,
                *child,
                target_x,
                target_y,
                ik.bend_direction,
                ik.softness,
                ik.stretch,
                ik.scale_y_mode,
                mix,
            );
            true
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_one(
    skeleton: &mut Skeleton,
    bone_index: usize,
    target_x: f32,
    target_y: f32,
    compress: bool,
    stretch: bool,
    scale_y_mode: crate::ScaleYMode,
    alpha: f32,
) {
    if bone_index >= skeleton.bones.len() {
        return;
    }
    let Some(parent_index) = skeleton.bones[bone_index].parent else {
        return;
    };
    if parent_index >= skeleton.bones.len() {
        return;
    }

    let (pa, mut pb, pc, mut pd, pwx, pwy) = {
        let p = &skeleton.bones[parent_index];
        (p.a, p.b, p.c, p.d, p.world_x, p.world_y)
    };

    let (inherit, world_x, world_y, ax, ay, arotation, mut sx, mut sy, ashear_x, ashear_y) = {
        let b = &skeleton.bones[bone_index];
        (
            b.inherit,
            b.world_x,
            b.world_y,
            b.ax,
            b.ay,
            b.arotation,
            b.ascale_x,
            b.ascale_y,
            b.ashear_x,
            b.ashear_y,
        )
    };

    let mut rotation_ik = -ashear_x - arotation;
    let (mut tx, mut ty) = match inherit {
        crate::Inherit::OnlyTranslation => (
            (target_x - world_x) * signum(skeleton.scale_x),
            (target_y - world_y) * signum(skeleton.scale_y),
        ),
        crate::Inherit::NoRotationOrReflection => {
            let denom = (pa * pa + pc * pc).max(1.0e-5);
            let s = (pa * pd - pb * pc).abs() / denom;
            let sa = pa / skeleton.scale_x;
            let sc = pc / skeleton.scale_y;
            pb = -sc * s * skeleton.scale_x;
            pd = sa * s * skeleton.scale_y;
            rotation_ik += atan2_degrees(sc, sa);
            // fallthrough to default branch with adjusted pb/pd.
            let x = target_x - pwx;
            let y = target_y - pwy;
            let det = pa * pd - pb * pc;
            if det.abs() <= 1.0e-5 {
                (0.0, 0.0)
            } else {
                ((x * pd - y * pb) / det - ax, (y * pa - x * pc) / det - ay)
            }
        }
        _ => {
            let x = target_x - pwx;
            let y = target_y - pwy;
            let det = pa * pd - pb * pc;
            if det.abs() <= 1.0e-5 {
                (0.0, 0.0)
            } else {
                ((x * pd - y * pb) / det - ax, (y * pa - x * pc) / det - ay)
            }
        }
    };

    rotation_ik += atan2_degrees(ty, tx);
    if sx < 0.0 {
        rotation_ik += 180.0;
    }
    rotation_ik = shortest_rotation(rotation_ik);

    if compress || stretch {
        if matches!(
            inherit,
            crate::Inherit::NoScale | crate::Inherit::NoScaleOrReflection
        ) {
            tx = target_x - world_x;
            ty = target_y - world_y;
        }
        let length = skeleton
            .data
            .bones
            .get(bone_index)
            .map(|d| d.length)
            .unwrap_or(0.0);
        let b = length * sx;
        if b > 1.0e-5 {
            let dd = tx * tx + ty * ty;
            if (compress && dd < b * b) || (stretch && dd > b * b) {
                let s = (dd.sqrt() / b - 1.0) * alpha + 1.0;
                sx *= s;
                match scale_y_mode {
                    crate::ScaleYMode::Uniform => {
                        sy *= s;
                    }
                    crate::ScaleYMode::Volume => {
                        sy /= if s < 0.7 { 0.25 + 0.642857 * s } else { s };
                    }
                    crate::ScaleYMode::None => {}
                }
            }
        }
    }

    let bone = &mut skeleton.bones[bone_index];
    bone.ax = ax;
    bone.ay = ay;
    bone.arotation = arotation + rotation_ik * alpha;
    bone.ascale_x = sx;
    bone.ascale_y = sy;
    bone.ashear_x = ashear_x;
    bone.ashear_y = ashear_y;
}

#[allow(clippy::too_many_arguments)]
fn apply_two(
    skeleton: &mut Skeleton,
    parent_index: usize,
    child_index: usize,
    target_x: f32,
    target_y: f32,
    bend_direction: i32,
    softness: f32,
    stretch: bool,
    scale_y_mode: crate::ScaleYMode,
    alpha: f32,
) {
    const EPSILON: f32 = 1.0e-5;
    const PI: f32 = std::f32::consts::PI;
    const RAD_DEG: f32 = 180.0 / PI;

    if parent_index >= skeleton.bones.len() || child_index >= skeleton.bones.len() {
        return;
    }
    if skeleton.bones[parent_index].inherit != crate::Inherit::Normal
        || skeleton.bones[child_index].inherit != crate::Inherit::Normal
    {
        return;
    }

    let Some(pp_index) = skeleton.bones[parent_index].parent else {
        return;
    };
    if pp_index >= skeleton.bones.len() {
        return;
    }

    let (px, py, parent_rotation, psx0, psy0) = {
        let p = &skeleton.bones[parent_index];
        (p.ax, p.ay, p.arotation, p.ascale_x, p.ascale_y)
    };
    let mut psx = psx0;
    let mut psy = psy0;
    let mut os1 = 0.0f32;
    let mut s2 = 1.0f32;
    if psx < 0.0 {
        psx = -psx;
        os1 = 180.0;
        s2 = -1.0;
    }
    if psy < 0.0 {
        psy = -psy;
        s2 = -s2;
    }

    let (cx, child_ay, child_rotation, csx0, csy0, child_shear_x, child_shear_y) = {
        let c = &skeleton.bones[child_index];
        (
            c.ax,
            c.ay,
            c.arotation,
            c.ascale_x,
            c.ascale_y,
            c.ashear_x,
            c.ashear_y,
        )
    };
    let mut csx = csx0;
    let mut os2 = 0.0f32;
    if csx < 0.0 {
        csx = -csx;
        os2 = 180.0;
    }

    let (pa, pb, pc, pd, pwx, pwy) = {
        let p = &skeleton.bones[parent_index];
        (p.a, p.b, p.c, p.d, p.world_x, p.world_y)
    };

    let u = (psx - psy).abs() <= EPSILON;
    let (cy, cwx, cwy) = if !u || stretch {
        (0.0f32, pa * cx + pwx, pc * cx + pwy)
    } else {
        (
            child_ay,
            pa * cx + pb * child_ay + pwx,
            pc * cx + pd * child_ay + pwy,
        )
    };

    let (pp_a, pp_b, pp_c, pp_d, pp_wx, pp_wy) = {
        let pp = &skeleton.bones[pp_index];
        (pp.a, pp.b, pp.c, pp.d, pp.world_x, pp.world_y)
    };

    let mut id = pp_a * pp_d - pp_b * pp_c;
    let x = cwx - pp_wx;
    let y = cwy - pp_wy;
    id = if id.abs() <= EPSILON { 0.0 } else { 1.0 / id };
    let dx = (x * pp_d - y * pp_b) * id - px;
    let dy = (y * pp_a - x * pp_c) * id - py;

    let l1 = sqrt_f32(dx * dx + dy * dy);
    if l1 < EPSILON {
        apply_one(
            skeleton,
            parent_index,
            target_x,
            target_y,
            false,
            stretch,
            crate::ScaleYMode::None,
            alpha,
        );
        let child = &mut skeleton.bones[child_index];
        child.ax = cx;
        child.ay = cy;
        child.arotation = 0.0;
        child.ascale_x = csx0;
        child.ascale_y = csy0;
        child.ashear_x = child_shear_x;
        child.ashear_y = child_shear_y;
        return;
    }

    let l2 = skeleton
        .data
        .bones
        .get(child_index)
        .map(|d| d.length)
        .unwrap_or(0.0)
        * csx;

    let x = target_x - pp_wx;
    let y = target_y - pp_wy;
    let mut tx = (x * pp_d - y * pp_b) * id - px;
    let mut ty = (y * pp_a - x * pp_c) * id - py;
    let mut dd = tx * tx + ty * ty;

    if softness != 0.0 {
        let softness = softness * psx * (csx + 1.0) * 0.5;
        let td = sqrt_f32(dd);
        let sd = td - l1 - l2 * psx + softness;
        if sd > 0.0 {
            let mut p = (sd / (softness * 2.0)).min(1.0) - 1.0;
            p = (sd - softness * (1.0 - p * p)) / td;
            tx -= p * tx;
            ty -= p * ty;
            dd = tx * tx + ty * ty;
        }
    }

    let bend_dir = if bend_direction >= 0 { 1.0 } else { -1.0 };
    let mut a1 = 0.0f32;
    let mut a2 = 0.0f32;
    let mut solved = false;

    if u {
        let l2u = l2 * psx;
        let mut cos = (dd - l1 * l1 - l2u * l2u) / (2.0 * l1 * l2u);
        if cos < -1.0 {
            cos = -1.0;
            a2 = PI * bend_dir;
        } else if cos > 1.0 {
            if cos <= 1.0 + f32::EPSILON {
                cos = 1.0 - f32::EPSILON;
                a2 = acos_f32(cos) * bend_dir;
            } else {
                cos = 1.0;
                a2 = 0.0;
                if stretch {
                    let s = (sqrt_f32(dd) / (l1 + l2u) - 1.0) * alpha + 1.0;
                    {
                        let parent = &mut skeleton.bones[parent_index];
                        parent.ascale_x *= s;
                        match scale_y_mode {
                            crate::ScaleYMode::Uniform => {
                                parent.ascale_y *= s;
                            }
                            crate::ScaleYMode::Volume => {
                                parent.ascale_y /= if s < 0.7 { 0.25 + 0.642857 * s } else { s };
                            }
                            crate::ScaleYMode::None => {}
                        }
                    }
                }
            }
        } else {
            a2 = acos_f32(cos) * bend_dir;
        }
        let aa = l1 + l2u * cos;
        let bb = l2u * sin_f32(a2);
        a1 = atan2_radians(ty * aa - tx * bb, tx * aa + ty * bb);
    } else {
        let a = psx * l2;
        let b = psy * l2;
        let aa = a * a;
        let bb = b * b;
        let ta = atan2_radians(ty, tx);
        let mut c = bb * l1 * l1 + aa * dd - aa * bb;
        let c1 = -2.0 * bb * l1;
        let c2 = bb - aa;
        let disc = c1 * c1 - 4.0 * c2 * c;

        if disc >= 0.0 {
            let mut q = sqrt_f32(disc);
            if c1 < 0.0 {
                q = -q;
            }
            q = -(c1 + q) * 0.5;
            let r0 = q / c2;
            let r1 = c / q;
            let r = if r0.abs() < r1.abs() { r0 } else { r1 };
            let r0 = dd - r * r;
            if r0 >= 0.0 {
                let y = sqrt_f32(r0) * bend_dir;
                a1 = ta - atan2_radians(y, r);
                a2 = atan2_radians(y / psy, (r - l1) / psx);
                solved = true;
            }
        }

        if !solved {
            let mut min_angle = PI;
            let mut min_x = l1 - a;
            let mut min_dist = min_x * min_x;
            let mut min_y = 0.0f32;
            let mut max_angle = 0.0f32;
            let mut max_x = l1 + a;
            let mut max_dist = max_x * max_x;
            let mut max_y = 0.0f32;
            c = -a * l1 / (aa - bb);
            if (-1.0..=1.0).contains(&c) {
                let c = acos_f32(c);
                let x = a * cos_f32(c) + l1;
                let y = b * sin_f32(c);
                let d = x * x + y * y;
                if d < min_dist {
                    min_angle = c;
                    min_dist = d;
                    min_x = x;
                    min_y = y;
                }
                if d > max_dist {
                    max_angle = c;
                    max_dist = d;
                    max_x = x;
                    max_y = y;
                }
            }
            if dd <= (min_dist + max_dist) * 0.5 {
                a1 = ta - atan2_radians(min_y * bend_dir, min_x);
                a2 = min_angle * bend_dir;
            } else {
                a1 = ta - atan2_radians(max_y * bend_dir, max_x);
                a2 = max_angle * bend_dir;
            }
        }
    }

    let os = atan2_radians(cy, cx) * s2;

    a1 = (a1 - os) * RAD_DEG + os1 - parent_rotation;
    if a1 > 180.0 {
        a1 -= 360.0;
    } else if a1 <= -180.0 {
        a1 += 360.0;
    }

    a2 = ((a2 + os) * RAD_DEG - child_shear_x) * s2 + os2 - child_rotation;
    if a2 > 180.0 {
        a2 -= 360.0;
    } else if a2 <= -180.0 {
        a2 += 360.0;
    }

    let parent = &mut skeleton.bones[parent_index];
    parent.ax = px;
    parent.ay = py;
    parent.arotation = parent_rotation + a1 * alpha;

    let child = &mut skeleton.bones[child_index];
    child.ax = cx;
    child.ay = cy;
    child.arotation = child_rotation + a2 * alpha;
    child.ascale_x = csx0;
    child.ascale_y = csy0;
    child.ashear_x = child_shear_x;
    child.ashear_y = child_shear_y;
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

fn shortest_rotation(mut degrees: f32) -> f32 {
    if degrees > 180.0 {
        degrees -= 360.0;
    } else if degrees <= -180.0 {
        degrees += 360.0;
    }
    degrees
}
