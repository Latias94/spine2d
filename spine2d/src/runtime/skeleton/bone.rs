use super::{Skeleton, atan2_degrees, cos_f32, degrees_to_radians, sin_f32, sqrt_f32};
use std::sync::atomic::{AtomicBool, Ordering};

static Y_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
pub struct Bone {
    pub(super) data_index: usize,
    pub(super) parent: Option<usize>,

    pub(crate) inherit: crate::Inherit,
    pub(crate) active: bool,

    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) rotation: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    pub(crate) shear_x: f32,
    pub(crate) shear_y: f32,

    pub(crate) ax: f32,
    pub(crate) ay: f32,
    pub(crate) arotation: f32,
    pub(crate) ascale_x: f32,
    pub(crate) ascale_y: f32,
    pub(crate) ashear_x: f32,
    pub(crate) ashear_y: f32,

    pub(crate) a: f32,
    pub(crate) b: f32,
    pub(crate) c: f32,
    pub(crate) d: f32,
    pub(crate) world_x: f32,
    pub(crate) world_y: f32,

    pub(super) world_epoch: u32,
    pub(super) local_epoch: u32,
}

impl Bone {
    /// Whether skeleton Y scale is interpreted in a Y-down coordinate system.
    ///
    /// Mirrors the official runtimes' global `Bone.isYDown`. This crate keeps
    /// the default `false` to preserve its Y-up runtime and oracle baseline.
    pub fn is_y_down() -> bool {
        Y_DOWN.load(Ordering::Relaxed)
    }

    /// Sets whether skeleton Y scale is interpreted in a Y-down coordinate system.
    ///
    /// Mirrors the official runtimes' global `Bone.setYDown`.
    pub fn set_y_down(y_down: bool) {
        Y_DOWN.store(y_down, Ordering::Relaxed);
    }

    pub fn data_index(&self) -> usize {
        self.data_index
    }

    pub fn parent_index(&self) -> Option<usize> {
        self.parent
    }

    /// The immediate child bone indices for this bone in the given skeleton.
    ///
    /// Mirrors the official runtimes' `Bone.getChildren` while keeping Rust's
    /// index-based skeleton storage public surface.
    pub fn child_indices<'a>(&self, skeleton: &'a Skeleton) -> &'a [usize] {
        skeleton
            .bone_children
            .get(self.data_index)
            .map_or(&[], Vec::as_slice)
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn inherit(&self) -> crate::Inherit {
        self.inherit
    }

    pub fn set_inherit(&mut self, inherit: crate::Inherit) {
        self.inherit = inherit;
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }

    pub fn set_scale_x(&mut self, scale_x: f32) {
        self.scale_x = scale_x;
    }

    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }

    pub fn set_scale_y(&mut self, scale_y: f32) {
        self.scale_y = scale_y;
    }

    pub fn scale(&self) -> (f32, f32) {
        (self.scale_x, self.scale_y)
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
    }

    pub fn shear_x(&self) -> f32 {
        self.shear_x
    }

    pub fn set_shear_x(&mut self, shear_x: f32) {
        self.shear_x = shear_x;
    }

    pub fn shear_y(&self) -> f32 {
        self.shear_y
    }

    pub fn set_shear_y(&mut self, shear_y: f32) {
        self.shear_y = shear_y;
    }

    pub fn applied_x(&self) -> f32 {
        self.ax
    }

    pub fn set_applied_x(&mut self, x: f32) {
        self.ax = x;
    }

    pub fn applied_y(&self) -> f32 {
        self.ay
    }

    pub fn set_applied_y(&mut self, y: f32) {
        self.ay = y;
    }

    pub fn applied_position(&self) -> (f32, f32) {
        (self.ax, self.ay)
    }

    pub fn set_applied_position(&mut self, x: f32, y: f32) {
        self.ax = x;
        self.ay = y;
    }

    pub fn applied_rotation(&self) -> f32 {
        self.arotation
    }

    pub fn set_applied_rotation(&mut self, rotation: f32) {
        self.arotation = rotation;
    }

    pub fn applied_scale_x(&self) -> f32 {
        self.ascale_x
    }

    pub fn set_applied_scale_x(&mut self, scale_x: f32) {
        self.ascale_x = scale_x;
    }

    pub fn applied_scale_y(&self) -> f32 {
        self.ascale_y
    }

    pub fn set_applied_scale_y(&mut self, scale_y: f32) {
        self.ascale_y = scale_y;
    }

    pub fn applied_scale(&self) -> (f32, f32) {
        (self.ascale_x, self.ascale_y)
    }

    pub fn set_applied_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.ascale_x = scale_x;
        self.ascale_y = scale_y;
    }

    pub fn applied_shear_x(&self) -> f32 {
        self.ashear_x
    }

    pub fn set_applied_shear_x(&mut self, shear_x: f32) {
        self.ashear_x = shear_x;
    }

    pub fn applied_shear_y(&self) -> f32 {
        self.ashear_y
    }

    pub fn set_applied_shear_y(&mut self, shear_y: f32) {
        self.ashear_y = shear_y;
    }

    pub fn applied_shear(&self) -> (f32, f32) {
        (self.ashear_x, self.ashear_y)
    }

    pub fn set_applied_shear(&mut self, shear_x: f32, shear_y: f32) {
        self.ashear_x = shear_x;
        self.ashear_y = shear_y;
    }

    pub fn a(&self) -> f32 {
        self.a
    }

    pub fn set_a(&mut self, a: f32) {
        self.a = a;
    }

    pub fn b(&self) -> f32 {
        self.b
    }

    pub fn set_b(&mut self, b: f32) {
        self.b = b;
    }

    pub fn c(&self) -> f32 {
        self.c
    }

    pub fn set_c(&mut self, c: f32) {
        self.c = c;
    }

    pub fn d(&self) -> f32 {
        self.d
    }

    pub fn set_d(&mut self, d: f32) {
        self.d = d;
    }

    pub fn world_x(&self) -> f32 {
        self.world_x
    }

    pub fn set_world_x(&mut self, world_x: f32) {
        self.world_x = world_x;
    }

    pub fn world_y(&self) -> f32 {
        self.world_y
    }

    pub fn set_world_y(&mut self, world_y: f32) {
        self.world_y = world_y;
    }

    pub fn world_position(&self) -> (f32, f32) {
        (self.world_x, self.world_y)
    }

    pub fn set_world_position(&mut self, world_x: f32, world_y: f32) {
        self.world_x = world_x;
        self.world_y = world_y;
    }

    pub fn world_rotation_x(&self) -> f32 {
        atan2_degrees(self.c, self.a)
    }

    pub fn world_rotation_y(&self) -> f32 {
        atan2_degrees(self.d, self.b)
    }

    pub fn world_scale_x(&self) -> f32 {
        sqrt_f32(self.a * self.a + self.c * self.c)
    }

    pub fn world_scale_y(&self) -> f32 {
        sqrt_f32(self.b * self.b + self.d * self.d)
    }

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

    /// Transforms a point from world coordinates into the parent bone's local
    /// coordinates, or returns the point unchanged for a root bone.
    ///
    /// Mirrors the official runtimes' `BonePose.worldToParent`.
    pub fn world_to_parent(&self, skeleton: &Skeleton, world_x: f32, world_y: f32) -> (f32, f32) {
        match self.parent.and_then(|index| skeleton.bones.get(index)) {
            Some(parent) => parent.world_to_local(world_x, world_y),
            None => (world_x, world_y),
        }
    }

    /// Transforms a point from the parent bone's local coordinates into world
    /// coordinates, or returns the point unchanged for a root bone.
    ///
    /// Mirrors the official runtimes' `BonePose.parentToWorld`.
    pub fn parent_to_world(&self, skeleton: &Skeleton, parent_x: f32, parent_y: f32) -> (f32, f32) {
        match self.parent.and_then(|index| skeleton.bones.get(index)) {
            Some(parent) => parent.local_to_world(parent_x, parent_y),
            None => (parent_x, parent_y),
        }
    }

    /// Transforms a world rotation into this bone's applied local rotation.
    ///
    /// Mirrors the official runtimes' `BonePose.worldToLocalRotation`.
    pub fn world_to_local_rotation(&self, world_rotation: f32) -> f32 {
        let world_rotation = degrees_to_radians(world_rotation);
        let sin_rot = sin_f32(world_rotation);
        let cos_rot = cos_f32(world_rotation);
        atan2_degrees(
            self.a * sin_rot - self.c * cos_rot,
            self.d * cos_rot - self.b * sin_rot,
        ) + self.arotation
            - self.ashear_x
    }

    /// Transforms an applied local rotation into world rotation.
    ///
    /// Mirrors the official runtimes' `BonePose.localToWorldRotation`.
    pub fn local_to_world_rotation(&self, local_rotation: f32) -> f32 {
        let local_rotation = degrees_to_radians(local_rotation - self.arotation - self.ashear_x);
        let sin_rot = sin_f32(local_rotation);
        let cos_rot = cos_f32(local_rotation);
        atan2_degrees(
            cos_rot * self.c + sin_rot * self.d,
            cos_rot * self.a + sin_rot * self.b,
        )
    }

    /// Rotates this bone's world transform matrix by the specified degrees.
    ///
    /// Mirrors the official runtimes' `BonePose.rotateWorld`. Like the world
    /// matrix setters, this mutates only this bone's stored world transform.
    pub fn rotate_world(&mut self, degrees: f32) {
        let degrees = degrees_to_radians(degrees);
        let sin_rot = sin_f32(degrees);
        let cos_rot = cos_f32(degrees);
        let ra = self.a;
        let rb = self.b;
        self.a = cos_rot * ra - sin_rot * self.c;
        self.b = cos_rot * rb - sin_rot * self.d;
        self.c = sin_rot * ra + cos_rot * self.c;
        self.d = sin_rot * rb + cos_rot * self.d;
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
    let rotation_x = degrees_to_radians(bone.arotation + bone.ashear_x);
    let rotation_y = degrees_to_radians(bone.arotation + 90.0 + bone.ashear_y);
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

    bone.world_x = pa.mul_add(bone.ax, pb * bone.ay) + parent.world_x;
    bone.world_y = pc.mul_add(bone.ax, pd * bone.ay) + parent.world_y;

    match bone.inherit {
        crate::Inherit::Normal => {
            let rotation_x = degrees_to_radians(bone.arotation + bone.ashear_x);
            let rotation_y = degrees_to_radians(bone.arotation + 90.0 + bone.ashear_y);
            let la = cos_f32(rotation_x) * bone.ascale_x;
            let lb = cos_f32(rotation_y) * bone.ascale_y;
            let lc = sin_f32(rotation_x) * bone.ascale_x;
            let ld = sin_f32(rotation_y) * bone.ascale_y;

            bone.a = pa.mul_add(la, pb * lc);
            bone.b = pa.mul_add(lb, pb * ld);
            bone.c = pc.mul_add(la, pd * lc);
            bone.d = pc.mul_add(lb, pd * ld);
        }
        crate::Inherit::OnlyTranslation => {
            let rotation_x = degrees_to_radians(bone.arotation + bone.ashear_x);
            let rotation_y = degrees_to_radians(bone.arotation + 90.0 + bone.ashear_y);
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

            let rotation_x = degrees_to_radians(bone.arotation + bone.ashear_x - prx);
            let rotation_y = degrees_to_radians(bone.arotation + bone.ashear_y - prx + 90.0);
            let la = cos_f32(rotation_x) * bone.ascale_x;
            let lb = cos_f32(rotation_y) * bone.ascale_y;
            let lc = sin_f32(rotation_x) * bone.ascale_x;
            let ld = sin_f32(rotation_y) * bone.ascale_y;

            bone.a = pa.mul_add(la, -(pb * lc));
            bone.b = pa.mul_add(lb, -(pb * ld));
            bone.c = pc.mul_add(la, pd * lc);
            bone.d = pc.mul_add(lb, pd * ld);

            bone.a *= skeleton_scale_x;
            bone.b *= skeleton_scale_x;
            bone.c *= skeleton_scale_y;
            bone.d *= skeleton_scale_y;
        }
        crate::Inherit::NoScale | crate::Inherit::NoScaleOrReflection => {
            let rotation = degrees_to_radians(bone.arotation);
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

            let shear_x = degrees_to_radians(bone.ashear_x);
            let shear_y = degrees_to_radians(90.0 + bone.ashear_y);
            let la = cos_f32(shear_x) * bone.ascale_x;
            let lb = cos_f32(shear_y) * bone.ascale_y;
            let lc = sin_f32(shear_x) * bone.ascale_x;
            let ld = sin_f32(shear_y) * bone.ascale_y;

            bone.a = za.mul_add(la, zb * lc);
            bone.b = za.mul_add(lb, zb * ld);
            bone.c = zc.mul_add(la, zd * lc);
            bone.d = zc.mul_add(lb, zd * ld);

            bone.a *= skeleton_scale_x;
            bone.b *= skeleton_scale_x;
            bone.c *= skeleton_scale_y;
            bone.d *= skeleton_scale_y;
        }
    }
}

pub(super) fn modify_world(skeleton: &mut Skeleton, bone_index: usize) {
    if bone_index >= skeleton.bones.len() {
        return;
    }
    let epoch = skeleton.update_epoch;
    skeleton.bones[bone_index].world_epoch = epoch;
    skeleton.bones[bone_index].local_epoch = epoch;
    reset_world_children_if_updated(skeleton, bone_index, epoch);
}

pub(super) fn update_world_transform(skeleton: &mut Skeleton, bone_index: usize) {
    if bone_index >= skeleton.bones.len() {
        return;
    }
    if !skeleton.bones[bone_index].active {
        return;
    }
    if skeleton.bones[bone_index].world_epoch == skeleton.update_epoch {
        return;
    }
    if skeleton.bones[bone_index].local_epoch == skeleton.update_epoch {
        update_applied_transform(skeleton, bone_index);
        skeleton.bones[bone_index].local_epoch = 0;
    }

    let parent_index = skeleton.bones[bone_index].parent;
    let skeleton_scale_y = skeleton.effective_scale_y();
    if let Some(parent_index) = parent_index {
        if parent_index >= skeleton.bones.len() {
            return;
        }
        if !skeleton.bones[parent_index].active {
            return;
        }

        let parent = ParentTransform::from_bone(&skeleton.bones[parent_index]);
        update_world_transform_child(
            &mut skeleton.bones[bone_index],
            skeleton.scale_x,
            skeleton_scale_y,
            &parent,
        );
    } else {
        update_world_transform_root(
            &mut skeleton.bones[bone_index],
            skeleton.x,
            skeleton.y,
            skeleton.scale_x,
            skeleton_scale_y,
        );
    }

    skeleton.bones[bone_index].world_epoch = skeleton.update_epoch;
}

pub(super) fn modify_local(skeleton: &mut Skeleton, bone_index: usize) {
    if bone_index >= skeleton.bones.len() {
        return;
    }
    let epoch = skeleton.update_epoch;
    if skeleton.bones[bone_index].local_epoch == epoch {
        update_applied_transform(skeleton, bone_index);
    }
    skeleton.bones[bone_index].local_epoch = 0;
    skeleton.bones[bone_index].world_epoch = 0;
    reset_world_children_if_updated(skeleton, bone_index, epoch);
}

pub(super) fn update_applied_transform(skeleton: &mut Skeleton, bone_index: usize) {
    if bone_index >= skeleton.bones.len() {
        return;
    }

    let (a, b, c0, d, wx, wy) = {
        let bone = &skeleton.bones[bone_index];
        (bone.a, bone.b, bone.c, bone.d, bone.world_x, bone.world_y)
    };

    let parent = skeleton.bones[bone_index].parent;
    let skeleton_scale_y = skeleton.effective_scale_y();

    if parent.is_none() {
        let sxi = 1.0 / skeleton.scale_x;
        let syi = 1.0 / skeleton_scale_y;
        let ra = a * sxi;
        let rb = b * sxi;
        let rc = c0 * syi;
        let rd = d * syi;
        let (arotation, ascale_x, ascale_y, ashear_x, ashear_y) =
            decompose_local_with_rotation(ra, rb, rc, rd, 0.0);
        let bone = &mut skeleton.bones[bone_index];
        bone.ax = (wx - skeleton.x) * sxi;
        bone.ay = (wy - skeleton.y) * syi;
        bone.arotation = arotation;
        bone.ascale_x = ascale_x;
        bone.ascale_y = ascale_y;
        bone.ashear_x = ashear_x;
        bone.ashear_y = ashear_y;
        bone.local_epoch = 0;
        return;
    }

    let parent_index = parent.unwrap();
    let (mut pa, pb, mut pc, pd, pwx, pwy) = {
        let p = &skeleton.bones[parent_index];
        (p.a, p.b, p.c, p.d, p.world_x, p.world_y)
    };
    let pad = pa * pd - pb * pc;
    let pid = 1.0 / pad;
    let ia = pd * pid;
    let ib = pb * pid;
    let ic = pc * pid;
    let id = pa * pid;

    let dx = wx - pwx;
    let dy = wy - pwy;
    let ax = dx * ia - dy * ib;
    let ay = dy * id - dx * ic;

    let (arotation, ascale_x, ascale_y, ashear_x, ashear_y) =
        match skeleton.bones[bone_index].inherit {
            crate::Inherit::Normal => {
                let ra = ia * a - ib * c0;
                let rb = ia * b - ib * d;
                let rc = id * c0 - ic * a;
                let rd = id * d - ic * b;
                decompose_local_with_rotation(ra, rb, rc, rd, 0.0)
            }
            crate::Inherit::OnlyTranslation => {
                let sxi = 1.0 / skeleton.scale_x;
                let syi = 1.0 / skeleton_scale_y;
                decompose_local_with_rotation(a * sxi, b * sxi, c0 * syi, d * syi, 0.0)
            }
            crate::Inherit::NoRotationOrReflection => {
                let sxi = 1.0 / skeleton.scale_x;
                let syi = 1.0 / skeleton_scale_y;
                pa *= sxi;
                pc *= syi;
                let wa = a * sxi;
                let wb = b * sxi;
                let wc = c0 * syi;
                let wd = d * syi;
                let s = 1.0 / (pa * pa + pc * pc);
                let det = 1.0 / (pad * sxi * syi).abs();
                decompose_local_with_rotation(
                    (pa * wa + pc * wc) * s,
                    (pa * wb + pc * wd) * s,
                    (pa * wc - pc * wa) * det,
                    (pa * wd - pc * wb) * det,
                    atan2_degrees(pc, pa),
                )
            }
            crate::Inherit::NoScale | crate::Inherit::NoScaleOrReflection => {
                let sxi = 1.0 / skeleton.scale_x;
                let syi = 1.0 / skeleton_scale_y;
                let wa = a * sxi;
                let wb = b * sxi;
                let wc = c0 * syi;
                let wd = d * syi;
                let mut tx = pd * a - pb * c0;
                let mut ty = pa * c0 - pc * a;
                if pad < 0.0 {
                    tx = -tx;
                    ty = -ty;
                }
                let rotation = atan2_degrees(ty, tx);
                let r = degrees_to_radians(rotation);
                let cos_r = cos_f32(r);
                let sin_r = sin_f32(r);
                let mut za = (pa * cos_r + pb * sin_r) * sxi;
                let mut zc = (pc * cos_r + pd * sin_r) * syi;
                let s = 1.0 / sqrt_f32(za * za + zc * zc);
                za *= s;
                zc *= s;
                let si = if skeleton.bones[bone_index].inherit == crate::Inherit::NoScale
                    && (pad < 0.0) != ((skeleton.scale_x < 0.0) != (skeleton_scale_y < 0.0))
                {
                    -1.0
                } else {
                    1.0
                };
                let (shear_x, scale_x, scale_y, shear_y) = decompose_local(
                    za * wa + zc * wc,
                    za * wb + zc * wd,
                    (za * wc - zc * wa) * si,
                    (za * wd - zc * wb) * si,
                );
                (rotation, scale_x, scale_y, shear_x, shear_y)
            }
        };

    let bone = &mut skeleton.bones[bone_index];
    bone.ax = ax;
    bone.ay = ay;
    bone.arotation = arotation;
    bone.ascale_x = ascale_x;
    bone.ascale_y = ascale_y;
    bone.ashear_x = ashear_x;
    bone.ashear_y = ashear_y;
    bone.local_epoch = 0;
}

fn reset_world_children_if_updated(skeleton: &mut Skeleton, bone_index: usize, epoch: u32) {
    let children = skeleton
        .bone_children
        .get(bone_index)
        .cloned()
        .unwrap_or_default();
    for child in children {
        if child >= skeleton.bones.len() {
            continue;
        }
        if skeleton.bones[child].world_epoch == epoch {
            skeleton.bones[child].world_epoch = 0;
            skeleton.bones[child].local_epoch = 0;
            reset_world_children_if_updated(skeleton, child, epoch);
        }
    }
}

fn decompose_local(ra: f32, rb: f32, rc: f32, rd: f32) -> (f32, f32, f32, f32) {
    let x = ra * ra + rc * rc;
    let y = rb * rb + rd * rd;
    let (shear_x, scale_x) = if x > 1.0e-10 {
        (atan2_degrees(rc, ra), sqrt_f32(x))
    } else {
        (0.0, 0.0)
    };
    let mut scale_y = sqrt_f32(y);
    let shear_y = if y > 1.0e-10 {
        let mut value = atan2_degrees(rd, rb);
        if ra * rd - rb * rc < 0.0 {
            scale_y = -scale_y;
            value += 90.0;
        } else {
            value -= 90.0;
        }
        if value > 180.0 {
            value -= 360.0;
        } else if value <= -180.0 {
            value += 360.0;
        }
        value
    } else {
        0.0
    };
    (shear_x, scale_x, scale_y, shear_y)
}

fn decompose_local_with_rotation(
    ra: f32,
    rb: f32,
    rc: f32,
    rd: f32,
    ro: f32,
) -> (f32, f32, f32, f32, f32) {
    let shear_x = 0.0;
    let x = ra * ra + rc * rc;
    let y = rb * rb + rd * rd;
    if x > 1.0e-10 {
        let r = atan2_degrees(rc, ra);
        let rotation = r + ro;
        let scale_x = sqrt_f32(x);
        let mut scale_y = sqrt_f32(y);
        let shear_y = if y > 1.0e-10 {
            let mut value = atan2_degrees(rd, rb);
            if ra * rd - rb * rc < 0.0 {
                scale_y = -scale_y;
                value += 90.0 - r;
            } else {
                value -= 90.0 + r;
            }
            if value > 180.0 {
                value -= 360.0;
            } else if value <= -180.0 {
                value += 360.0;
            }
            value
        } else {
            0.0
        };
        (rotation, scale_x, scale_y, shear_x, shear_y)
    } else {
        let scale_x = 0.0;
        let scale_y = sqrt_f32(y);
        let shear_y = 0.0;
        let rotation = if y > 1.0e-10 {
            atan2_degrees(rd, rb) - 90.0 + ro
        } else {
            ro
        };
        (rotation, scale_x, scale_y, shear_x, shear_y)
    }
}
