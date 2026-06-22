use super::{Bone, atan2_degrees, cos_f32, sin_f32, sqrt_f32};

impl Bone {
    /// Transforms a point from world coordinates into this bone's local coordinates.
    ///
    /// Mirrors the official runtimes' `BonePose.worldToLocal`.
    pub fn world_to_local(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        let det = self.a * self.d - self.b * self.c;
        let x = world_x - self.world_x;
        let y = world_y - self.world_y;
        (
            (x * self.d - y * self.b) / det,
            (y * self.a - x * self.c) / det,
        )
    }

    /// Transforms a point from this bone's local coordinates into world coordinates.
    ///
    /// Mirrors the official runtimes' `BonePose.localToWorld`.
    pub fn local_to_world(&self, local_x: f32, local_y: f32) -> (f32, f32) {
        (
            local_x * self.a + local_y * self.b + self.world_x,
            local_x * self.c + local_y * self.d + self.world_y,
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub(super) struct ParentTransform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    world_x: f32,
    world_y: f32,
}

impl ParentTransform {
    pub(super) fn from_bone(bone: &Bone) -> Self {
        Self {
            a: bone.a,
            b: bone.b,
            c: bone.c,
            d: bone.d,
            world_x: bone.world_x,
            world_y: bone.world_y,
        }
    }
}

pub(super) fn update_world_transform_root(
    bone: &mut Bone,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
) {
    let rotation_x = (bone.arotation + bone.ashear_x).to_radians();
    let rotation_y = (bone.arotation + 90.0 + bone.ashear_y).to_radians();
    let la = cos_f32(rotation_x) * bone.ascale_x;
    let lb = cos_f32(rotation_y) * bone.ascale_y;
    let lc = sin_f32(rotation_x) * bone.ascale_x;
    let ld = sin_f32(rotation_y) * bone.ascale_y;

    bone.a = la * scale_x;
    bone.b = lb * scale_x;
    bone.c = lc * scale_y;
    bone.d = ld * scale_y;
    bone.world_x = bone.ax * scale_x + x;
    bone.world_y = bone.ay * scale_y + y;
}

pub(super) fn update_world_transform_child(
    bone: &mut Bone,
    skeleton_scale_x: f32,
    skeleton_scale_y: f32,
    parent: &ParentTransform,
) {
    let mut pa = parent.a;
    let mut pb = parent.b;
    let mut pc = parent.c;
    let mut pd = parent.d;

    bone.world_x = pa * bone.ax + pb * bone.ay + parent.world_x;
    bone.world_y = pc * bone.ax + pd * bone.ay + parent.world_y;

    match bone.inherit {
        crate::Inherit::Normal => {
            let rotation_x = (bone.arotation + bone.ashear_x).to_radians();
            let rotation_y = (bone.arotation + 90.0 + bone.ashear_y).to_radians();
            let la = cos_f32(rotation_x) * bone.ascale_x;
            let lb = cos_f32(rotation_y) * bone.ascale_y;
            let lc = sin_f32(rotation_x) * bone.ascale_x;
            let ld = sin_f32(rotation_y) * bone.ascale_y;

            bone.a = pa * la + pb * lc;
            bone.b = pa * lb + pb * ld;
            bone.c = pc * la + pd * lc;
            bone.d = pc * lb + pd * ld;
        }
        crate::Inherit::OnlyTranslation => {
            let rotation_x = (bone.arotation + bone.ashear_x).to_radians();
            let rotation_y = (bone.arotation + 90.0 + bone.ashear_y).to_radians();
            bone.a = cos_f32(rotation_x) * bone.ascale_x;
            bone.b = cos_f32(rotation_y) * bone.ascale_y;
            bone.c = sin_f32(rotation_x) * bone.ascale_x;
            bone.d = sin_f32(rotation_y) * bone.ascale_y;

            bone.a *= skeleton_scale_x;
            bone.b *= skeleton_scale_x;
            bone.c *= skeleton_scale_y;
            bone.d *= skeleton_scale_y;
        }
        crate::Inherit::NoRotationOrReflection => {
            let sx = if skeleton_scale_x.abs() > 1.0e-12 {
                1.0 / skeleton_scale_x
            } else {
                0.0
            };
            let sy = if skeleton_scale_y.abs() > 1.0e-12 {
                1.0 / skeleton_scale_y
            } else {
                0.0
            };
            pa *= sx;
            pc *= sy;

            let mut s = pa * pa + pc * pc;
            let prx;
            if s > 1.0e-10 {
                s = (pa * pd * sy - pb * sx * pc).abs() / s;
                pb = pc * s;
                pd = pa * s;
                prx = atan2_degrees(pc, pa);
            } else {
                pa = 0.0;
                pc = 0.0;
                prx = 90.0 - atan2_degrees(pd, pb);
            }

            let rotation_x = (bone.arotation + bone.ashear_x - prx).to_radians();
            let rotation_y = (bone.arotation + bone.ashear_y - prx + 90.0).to_radians();
            let la = cos_f32(rotation_x) * bone.ascale_x;
            let lb = cos_f32(rotation_y) * bone.ascale_y;
            let lc = sin_f32(rotation_x) * bone.ascale_x;
            let ld = sin_f32(rotation_y) * bone.ascale_y;

            bone.a = pa * la - pb * lc;
            bone.b = pa * lb - pb * ld;
            bone.c = pc * la + pd * lc;
            bone.d = pc * lb + pd * ld;

            bone.a *= skeleton_scale_x;
            bone.b *= skeleton_scale_x;
            bone.c *= skeleton_scale_y;
            bone.d *= skeleton_scale_y;
        }
        crate::Inherit::NoScale | crate::Inherit::NoScaleOrReflection => {
            let rotation = bone.arotation.to_radians();
            let cos = cos_f32(rotation);
            let sin = sin_f32(rotation);

            let za = (pa * cos + pb * sin) / skeleton_scale_x;
            let zc = (pc * cos + pd * sin) / skeleton_scale_y;
            let s = 1.0 / sqrt_f32(za * za + zc * zc);
            let za = za * s;
            let zc = zc * s;

            let mut zb = -zc;
            let mut zd = za;
            if matches!(bone.inherit, crate::Inherit::NoScale) {
                let det = pa * pd - pb * pc;
                let flip = (det < 0.0) != ((skeleton_scale_x < 0.0) != (skeleton_scale_y < 0.0));
                if flip {
                    zb = -zb;
                    zd = -zd;
                }
            }

            let shear_x = bone.ashear_x.to_radians();
            let shear_y = (90.0 + bone.ashear_y).to_radians();
            let la = cos_f32(shear_x) * bone.ascale_x;
            let lb = cos_f32(shear_y) * bone.ascale_y;
            let lc = sin_f32(shear_x) * bone.ascale_x;
            let ld = sin_f32(shear_y) * bone.ascale_y;

            bone.a = za * la + zb * lc;
            bone.b = za * lb + zb * ld;
            bone.c = zc * la + zd * lc;
            bone.d = zc * lb + zd * ld;

            bone.a *= skeleton_scale_x;
            bone.b *= skeleton_scale_x;
            bone.c *= skeleton_scale_y;
            bone.d *= skeleton_scale_y;
        }
    }
}
