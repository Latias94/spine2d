mod bone;
mod cache;
mod ik;
mod path;
mod physics;
mod slider;
mod slot;
mod transform;
mod vertices;

pub use bone::Bone;
pub use cache::UpdateCacheItem;
pub use ik::IkConstraint;
pub use path::PathConstraint;
pub use physics::{Physics, PhysicsConstraint};
pub use slider::SliderConstraint;
pub use slot::Slot;
pub use transform::TransformConstraint;

use crate::{SkeletonData, geometry::SkeletonClipper};
use path::{PathConstraintScratch, estimate_path_attachment_scratch_capacities};
use slot::SlotPose;
use std::sync::Arc;

#[derive(Copy, Clone, Debug)]
pub enum ConstraintRef<'a> {
    Ik(&'a IkConstraint),
    Transform(&'a TransformConstraint),
    Path(&'a PathConstraint),
    Physics(&'a PhysicsConstraint),
    Slider(&'a SliderConstraint),
}

impl ConstraintRef<'_> {
    fn order(&self, data: &SkeletonData) -> i32 {
        match self {
            ConstraintRef::Ik(constraint) => data
                .ik_constraints
                .get(constraint.data_index)
                .map(|constraint| constraint.order)
                .unwrap_or(0),
            ConstraintRef::Transform(constraint) => data
                .transform_constraints
                .get(constraint.data_index)
                .map(|constraint| constraint.order)
                .unwrap_or(0),
            ConstraintRef::Path(constraint) => data
                .path_constraints
                .get(constraint.data_index)
                .map(|constraint| constraint.order)
                .unwrap_or(0),
            ConstraintRef::Physics(constraint) => data
                .physics_constraints
                .get(constraint.data_index)
                .map(|constraint| constraint.order)
                .unwrap_or(0),
            ConstraintRef::Slider(constraint) => data
                .slider_constraints
                .get(constraint.data_index)
                .map(|constraint| constraint.order)
                .unwrap_or(0),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            ConstraintRef::Ik(constraint) => constraint.is_active(),
            ConstraintRef::Transform(constraint) => constraint.is_active(),
            ConstraintRef::Path(constraint) => constraint.is_active(),
            ConstraintRef::Physics(constraint) => constraint.is_active(),
            ConstraintRef::Slider(constraint) => constraint.is_active(),
        }
    }
}

fn atan2_degrees(y: f32, x: f32) -> f32 {
    atan2_radians(y, x) * (180.0f32 / std::f32::consts::PI)
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * (std::f32::consts::PI / 180.0f32)
}

fn atan2_radians(y: f32, x: f32) -> f32 {
    (y as f64).atan2(x as f64) as f32
}

fn sqrt_f32(v: f32) -> f32 {
    (v as f64).sqrt() as f32
}

fn acos_f32(v: f32) -> f32 {
    (v as f64).acos() as f32
}

fn sin_f32(v: f32) -> f32 {
    (v as f64).sin() as f32
}

fn cos_f32(v: f32) -> f32 {
    (v as f64).cos() as f32
}

impl crate::PointAttachmentData {
    pub fn compute_world_position(&self, bone: &Bone) -> [f32; 2] {
        [
            bone.a * self.x + bone.b * self.y + bone.world_x,
            bone.c * self.x + bone.d * self.y + bone.world_y,
        ]
    }

    pub fn compute_world_rotation(&self, bone: &Bone) -> f32 {
        let r = self.rotation.to_radians();
        let cos = cos_f32(r);
        let sin = sin_f32(r);
        let x = cos * bone.a + sin * bone.b;
        let y = cos * bone.c + sin * bone.d;
        atan2_degrees(y, x)
    }
}

impl crate::AttachmentData {
    /// Computes world vertices for vertex-bearing attachments using the current skeleton pose.
    pub fn compute_world_vertices(
        &self,
        skeleton: &Skeleton,
        slot_index: usize,
    ) -> Option<Vec<f32>> {
        let vertices = match self {
            crate::AttachmentData::Mesh(a) => &a.vertices,
            crate::AttachmentData::Path(a) => &a.vertices,
            crate::AttachmentData::BoundingBox(a) => &a.vertices,
            crate::AttachmentData::Clipping(a) => &a.vertices,
            crate::AttachmentData::Region(_) | crate::AttachmentData::Point(_) => return None,
        };

        let world_vertices_length = match vertices {
            crate::MeshVertices::Unweighted(v) => v.len() * 2,
            crate::MeshVertices::Weighted(v) => v.len() * 2,
        };
        if world_vertices_length == 0 {
            return Some(Vec::new());
        }

        let mut out = vec![0.0f32; world_vertices_length];
        vertices::compute_attachment_world_vertices(
            skeleton,
            slot_index,
            vertices,
            0,
            world_vertices_length,
            &mut out,
            0,
            2,
        );
        Some(out)
    }
}

fn region_world_vertices(region: &crate::RegionAttachmentData, bone: &Bone) -> [(f32, f32); 4] {
    let local_x = -region.width * 0.5 * region.scale_x;
    let local_y = -region.height * 0.5 * region.scale_y;
    let local_x2 = region.width * 0.5 * region.scale_x;
    let local_y2 = region.height * 0.5 * region.scale_y;

    let r = region.rotation.to_radians();
    let cos = cos_f32(r);
    let sin = sin_f32(r);
    let x = region.x;
    let y = region.y;

    let local_x_cos = local_x * cos + x;
    let local_x_sin = local_x * sin;
    let local_y_cos = local_y * cos + y;
    let local_y_sin = local_y * sin;
    let local_x2_cos = local_x2 * cos + x;
    let local_x2_sin = local_x2 * sin;
    let local_y2_cos = local_y2 * cos + y;
    let local_y2_sin = local_y2 * sin;

    let bl = (local_x_cos - local_y_sin, local_y_cos + local_x_sin);
    let ul = (local_x_cos - local_y2_sin, local_y2_cos + local_x_sin);
    let ur = (local_x2_cos - local_y2_sin, local_y2_cos + local_x2_sin);
    let br = (local_x2_cos - local_y_sin, local_y_cos + local_x2_sin);

    [br, bl, ul, ur].map(|(x, y)| {
        (
            bone.a * x + bone.b * y + bone.world_x,
            bone.c * x + bone.d * y + bone.world_y,
        )
    })
}

fn include_flat_vertices_in_bounds(
    vertices: &[f32],
    min_x: &mut f32,
    min_y: &mut f32,
    max_x: &mut f32,
    max_y: &mut f32,
    has_vertices: &mut bool,
) {
    for point in vertices.chunks_exact(2) {
        let x = point[0];
        let y = point[1];
        *has_vertices = true;
        *min_x = (*min_x).min(x);
        *min_y = (*min_y).min(y);
        *max_x = (*max_x).max(x);
        *max_y = (*max_y).max(y);
    }
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub(crate) data: Arc<SkeletonData>,
    pub(crate) bones: Vec<Bone>,
    bone_children: Vec<Vec<usize>>,
    pub(crate) slots: Vec<Slot>,
    pub(crate) draw_order: Vec<usize>,
    applied_draw_order: Vec<usize>,
    draw_order_constrained: bool,
    pub(crate) skin: Option<String>,
    pub(crate) color: [f32; 4],
    wind_x: f32,
    wind_y: f32,
    gravity_x: f32,
    gravity_y: f32,
    pub(crate) ik_constraints: Vec<IkConstraint>,
    pub(crate) transform_constraints: Vec<TransformConstraint>,
    pub(crate) path_constraints: Vec<PathConstraint>,
    pub(crate) physics_constraints: Vec<PhysicsConstraint>,
    pub(crate) slider_constraints: Vec<SliderConstraint>,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) scale_x: f32,
    pub(crate) scale_y: f32,
    time: f32,
    update_epoch: u32,
    update_cache: Vec<UpdateCacheItem>,
    path_constraint_scratch: Vec<PathConstraintScratch>,
}

impl Skeleton {
    fn bone_modify_world(&mut self, bone_index: usize) {
        bone::modify_world(self, bone_index);
    }

    fn bone_modify_local(&mut self, bone_index: usize) {
        bone::modify_local(self, bone_index);
    }
    pub fn new(data: Arc<SkeletonData>) -> Self {
        let bones = data
            .bones
            .iter()
            .enumerate()
            .map(|(data_index, bone)| Bone {
                data_index,
                parent: bone.parent,
                inherit: bone.inherit,
                active: !bone.skin_required,
                x: bone.x,
                y: bone.y,
                rotation: bone.rotation,
                scale_x: bone.scale_x,
                scale_y: bone.scale_y,
                shear_x: bone.shear_x,
                shear_y: bone.shear_y,
                ax: bone.x,
                ay: bone.y,
                arotation: bone.rotation,
                ascale_x: bone.scale_x,
                ascale_y: bone.scale_y,
                ashear_x: bone.shear_x,
                ashear_y: bone.shear_y,
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                world_x: 0.0,
                world_y: 0.0,
                world_epoch: 0,
                local_epoch: 0,
            })
            .collect::<Vec<_>>();

        let bone_children = build_bone_children_indices(&bones);

        let slots = data
            .slots
            .iter()
            .enumerate()
            .map(|(data_index, slot)| Slot {
                data_index,
                bone: slot.bone,
                attachment: slot.attachment.clone(),
                attachment_skin: None,
                attachment_state: 0,
                sequence_index: slot.setup_pose.sequence_index,
                deform: Vec::new(),
                color: slot.setup_pose.color,
                has_dark: slot.setup_pose.has_dark,
                dark_color: slot.setup_pose.dark_color,
                applied_pose: SlotPose {
                    attachment: slot.attachment.clone(),
                    attachment_skin: None,
                    sequence_index: slot.setup_pose.sequence_index,
                    deform: Vec::new(),
                    color: slot.setup_pose.color,
                    has_dark: slot.setup_pose.has_dark,
                    dark_color: slot.setup_pose.dark_color,
                },
                pose_constrained: false,
            })
            .collect::<Vec<_>>();

        let draw_order = (0..slots.len()).collect::<Vec<_>>();
        let applied_draw_order = Vec::new();
        // Match upstream: skeletons start with no skin. The "default" skin (if present) is only
        // used as a fallback for attachment resolution.
        let skin = None;
        let color = [1.0, 1.0, 1.0, 1.0];

        let ik_constraints = data
            .ik_constraints
            .iter()
            .enumerate()
            .map(|(data_index, ik)| IkConstraint {
                data_index,
                bones: ik.bones.clone(),
                target: ik.target,
                scale_y_mode: ik.scale_y_mode,
                mix: ik.mix,
                softness: ik.softness,
                compress: ik.compress,
                stretch: ik.stretch,
                bend_direction: ik.bend_direction,
                active: true,
            })
            .collect::<Vec<_>>();

        let transform_constraints = data
            .transform_constraints
            .iter()
            .enumerate()
            .map(|(data_index, c)| TransformConstraint {
                data_index,
                bones: c.bones.clone(),
                source: c.source,
                mix_rotate: c.mix_rotate,
                mix_x: c.mix_x,
                mix_y: c.mix_y,
                mix_scale_x: c.mix_scale_x,
                mix_scale_y: c.mix_scale_y,
                mix_shear_y: c.mix_shear_y,
                active: true,
            })
            .collect::<Vec<_>>();

        let path_constraints = data
            .path_constraints
            .iter()
            .enumerate()
            .map(|(data_index, c)| PathConstraint {
                data_index,
                bones: c.bones.clone(),
                target: c.target,
                position: c.position,
                spacing: c.spacing,
                mix_rotate: c.mix_rotate,
                mix_x: c.mix_x,
                mix_y: c.mix_y,
                active: true,
            })
            .collect::<Vec<_>>();

        let physics_constraints = data
            .physics_constraints
            .iter()
            .enumerate()
            .map(|(data_index, c)| PhysicsConstraint {
                data_index,
                bone: c.bone,
                inertia: c.inertia,
                strength: c.strength,
                damping: c.damping,
                mass_inverse: c.mass_inverse,
                wind: c.wind,
                gravity: c.gravity,
                mix: c.mix,
                scale_y_mode: c.scale_y_mode,
                reset: true,
                ux: 0.0,
                uy: 0.0,
                cx: 0.0,
                cy: 0.0,
                tx: 0.0,
                ty: 0.0,
                x_offset: 0.0,
                x_lag: 0.0,
                x_velocity: 0.0,
                y_offset: 0.0,
                y_lag: 0.0,
                y_velocity: 0.0,
                rotate_offset: 0.0,
                rotate_lag: 0.0,
                rotate_velocity: 0.0,
                scale_offset: 0.0,
                scale_lag: 0.0,
                scale_velocity: 0.0,
                active: false,
                remaining: 0.0,
                last_time: 0.0,
            })
            .collect::<Vec<_>>();

        let slider_constraints = data
            .slider_constraints
            .iter()
            .enumerate()
            .map(|(data_index, c)| {
                let animation_bones = c
                    .animation
                    .and_then(|idx| data.animations.get(idx))
                    .map(|animation| animation.bones())
                    .unwrap_or_default();
                SliderConstraint {
                    data_index,
                    time: c.setup_time,
                    mix: c.setup_mix,
                    active: true,
                    animation_bones,
                }
            })
            .collect::<Vec<_>>();

        // Reduce per-frame allocations: pre-size scratch buffers based on constraint topology.
        let path_constraint_scratch = data
            .path_constraints
            .iter()
            .map(|c| {
                let bone_count = c.bones.len();
                let spaces_count = bone_count + 1;
                let (max_world_floats, max_curves) =
                    estimate_path_attachment_scratch_capacities(&data, c.target);
                let mut scratch = PathConstraintScratch::default();
                scratch.spaces.reserve(spaces_count);
                scratch.lengths.reserve(bone_count);
                scratch.positions.reserve(spaces_count * 3 + 2);
                scratch.world.reserve(max_world_floats);
                scratch.curves.reserve(max_curves);
                scratch
            })
            .collect::<Vec<_>>();

        let mut out = Self {
            data,
            bones,
            bone_children,
            slots,
            draw_order,
            applied_draw_order,
            draw_order_constrained: false,
            skin,
            color,
            wind_x: 1.0,
            wind_y: 0.0,
            gravity_x: 0.0,
            gravity_y: 1.0,
            ik_constraints,
            transform_constraints,
            path_constraints,
            physics_constraints,
            slider_constraints,
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            time: 0.0,
            update_epoch: 0,
            update_cache: Vec::new(),
            path_constraint_scratch,
        };
        // Match upstream runtime initialization:
        // - Slots start in setup pose (including setup attachments resolved via default skin fallback).
        // - The cache is built after setup values are in place.
        out.setup_pose();
        out.update_cache();
        out
    }

    pub fn data(&self) -> &SkeletonData {
        self.data.as_ref()
    }

    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }

    pub fn bones_mut(&mut self) -> &mut [Bone] {
        &mut self.bones
    }

    pub fn get_root_bone(&self) -> Option<&Bone> {
        self.bones.first()
    }

    fn bone_index_by_name(&self, bone_name: &str) -> Option<usize> {
        if bone_name.is_empty() {
            return None;
        }
        self.data
            .bones
            .iter()
            .position(|bone| bone.name == bone_name)
    }

    pub fn find_bone(&self, bone_name: &str) -> Option<&Bone> {
        let index = self.bone_index_by_name(bone_name)?;
        self.bones.get(index)
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn slots_mut(&mut self) -> &mut [Slot] {
        &mut self.slots
    }

    pub(crate) fn reset_constrained_slot_poses(&mut self) {
        for slot in &mut self.slots {
            slot.reset_constrained_pose();
        }
    }

    fn slot_index_by_name(&self, slot_name: &str) -> Option<usize> {
        if slot_name.is_empty() {
            return None;
        }
        self.data
            .slots
            .iter()
            .position(|slot| slot.name == slot_name)
    }

    pub fn find_slot(&self, slot_name: &str) -> Option<&Slot> {
        let index = self.slot_index_by_name(slot_name)?;
        self.slots.get(index)
    }

    /// The draw order used for rendering.
    ///
    /// This matches C++ `DrawOrder::getAppliedPose()`: it is normally the same as
    /// `draw_order_pose`, but slider constraints with draw-order timelines may modify this
    /// applied order during world transform updates.
    pub fn draw_order(&self) -> &[usize] {
        if self.draw_order_constrained {
            &self.applied_draw_order
        } else {
            &self.draw_order
        }
    }

    /// The unconstrained draw order pose set by animations or application code.
    pub fn draw_order_pose(&self) -> &[usize] {
        &self.draw_order
    }

    /// Mutates the unconstrained draw order pose.
    pub fn draw_order_mut(&mut self) -> &mut [usize] {
        &mut self.draw_order
    }

    pub(crate) fn draw_order_target_mut(&mut self, applied_pose: bool) -> &mut Vec<usize> {
        if applied_pose {
            if !self.draw_order_constrained {
                self.draw_order_constrained = true;
                self.applied_draw_order.clone_from(&self.draw_order);
            }
            &mut self.applied_draw_order
        } else {
            &mut self.draw_order
        }
    }

    pub(crate) fn slot_attachment_data_for_pose(
        &self,
        slot_index: usize,
        applied_pose: bool,
    ) -> Option<&crate::AttachmentData> {
        let slot = self.slots.get(slot_index)?;
        let pose = slot.pose_for(applied_pose);
        let key = pose.attachment_name()?;

        if let Some(source_skin) = pose.attachment_skin()
            && let Some(skin) = self.data.find_skin(source_skin)
            && let Some(att) = skin.get_attachment(slot_index, key)
        {
            return Some(att);
        }

        self.get_attachment(slot_index, key)
    }

    pub fn get_skin(&self) -> Option<&crate::SkinData> {
        self.skin
            .as_deref()
            .and_then(|name| self.data.find_skin(name))
    }

    pub fn get_color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn ik_constraints(&self) -> &[IkConstraint] {
        &self.ik_constraints
    }

    pub fn get_constraints(&self) -> Vec<ConstraintRef<'_>> {
        let mut constraints = Vec::with_capacity(
            self.ik_constraints.len()
                + self.transform_constraints.len()
                + self.path_constraints.len()
                + self.physics_constraints.len()
                + self.slider_constraints.len(),
        );
        constraints.extend(self.ik_constraints.iter().map(ConstraintRef::Ik));
        constraints.extend(
            self.transform_constraints
                .iter()
                .map(ConstraintRef::Transform),
        );
        constraints.extend(self.path_constraints.iter().map(ConstraintRef::Path));
        constraints.extend(self.physics_constraints.iter().map(ConstraintRef::Physics));
        constraints.extend(self.slider_constraints.iter().map(ConstraintRef::Slider));
        constraints.sort_by_key(|constraint| constraint.order(self.data.as_ref()));
        constraints
    }

    pub fn ik_constraints_mut(&mut self) -> &mut [IkConstraint] {
        &mut self.ik_constraints
    }

    fn ik_constraint_index_by_name(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .ik_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_ik_constraint(&self, constraint_name: &str) -> Option<&IkConstraint> {
        let index = self.ik_constraint_index_by_name(constraint_name)?;
        self.ik_constraints.get(index)
    }

    pub fn transform_constraints(&self) -> &[TransformConstraint] {
        &self.transform_constraints
    }

    pub fn transform_constraints_mut(&mut self) -> &mut [TransformConstraint] {
        &mut self.transform_constraints
    }

    fn transform_constraint_index_by_name(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .transform_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_transform_constraint(&self, constraint_name: &str) -> Option<&TransformConstraint> {
        let index = self.transform_constraint_index_by_name(constraint_name)?;
        self.transform_constraints.get(index)
    }

    pub fn path_constraints(&self) -> &[PathConstraint] {
        &self.path_constraints
    }

    pub fn path_constraints_mut(&mut self) -> &mut [PathConstraint] {
        &mut self.path_constraints
    }

    fn path_constraint_index_by_name(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .path_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_path_constraint(&self, constraint_name: &str) -> Option<&PathConstraint> {
        let index = self.path_constraint_index_by_name(constraint_name)?;
        self.path_constraints.get(index)
    }

    pub fn physics_constraints(&self) -> &[PhysicsConstraint] {
        &self.physics_constraints
    }

    pub fn physics_constraints_mut(&mut self) -> &mut [PhysicsConstraint] {
        &mut self.physics_constraints
    }

    fn physics_constraint_index_by_name(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .physics_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_physics_constraint(&self, constraint_name: &str) -> Option<&PhysicsConstraint> {
        let index = self.physics_constraint_index_by_name(constraint_name)?;
        self.physics_constraints.get(index)
    }

    pub fn slider_constraints(&self) -> &[SliderConstraint] {
        &self.slider_constraints
    }

    pub fn slider_constraints_mut(&mut self) -> &mut [SliderConstraint] {
        &mut self.slider_constraints
    }

    fn slider_constraint_index_by_name(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .slider_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_slider_constraint(&self, constraint_name: &str) -> Option<&SliderConstraint> {
        let index = self.slider_constraint_index_by_name(constraint_name)?;
        self.slider_constraints.get(index)
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

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }

    pub fn set_scale_x(&mut self, scale_x: f32) {
        self.scale_x = scale_x;
    }

    pub fn scale_y(&self) -> f32 {
        self.effective_scale_y()
    }

    pub fn set_scale_y(&mut self, scale_y: f32) {
        self.scale_y = scale_y;
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
    }

    pub(super) fn effective_scale_y(&self) -> f32 {
        self.scale_y * if Bone::is_y_down() { -1.0 } else { 1.0 }
    }

    pub fn get_time(&self) -> f32 {
        self.time
    }

    pub fn wind_x(&self) -> f32 {
        self.wind_x
    }

    pub fn set_wind_x(&mut self, wind_x: f32) {
        self.wind_x = wind_x;
    }

    pub fn wind_y(&self) -> f32 {
        self.wind_y
    }

    pub fn set_wind_y(&mut self, wind_y: f32) {
        self.wind_y = wind_y;
    }

    pub fn gravity_x(&self) -> f32 {
        self.gravity_x
    }

    pub fn set_gravity_x(&mut self, gravity_x: f32) {
        self.gravity_x = gravity_x;
    }

    pub fn gravity_y(&self) -> f32 {
        self.gravity_y
    }

    pub fn set_gravity_y(&mut self, gravity_y: f32) {
        self.gravity_y = gravity_y;
    }

    pub fn physics_translate(&mut self, x: f32, y: f32) {
        for constraint in &mut self.physics_constraints {
            constraint.translate(x, y);
        }
    }

    pub fn physics_rotate(&mut self, x: f32, y: f32, degrees: f32) {
        for constraint in &mut self.physics_constraints {
            constraint.rotate(x, y, degrees);
        }
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn update(&mut self, delta: f32) {
        self.time += delta;
    }

    pub fn update_cache(&mut self) {
        // Bones: active unless skinRequired, then only active if included by the current skin
        // (plus all parents of included bones).
        for (i, bone) in self.bones.iter_mut().enumerate() {
            let required = self
                .data
                .bones
                .get(i)
                .map(|b| b.skin_required)
                .unwrap_or(false);
            bone.active = !required;
        }

        let skin = self.skin.as_deref().and_then(|n| self.data.find_skin(n));
        if let Some(skin) = skin {
            for &bone_index in &skin.bones {
                let mut cur = Some(bone_index);
                while let Some(i) = cur {
                    if i >= self.bones.len() {
                        break;
                    }
                    self.bones[i].active = true;
                    cur = self.bones[i].parent;
                }
            }
        }

        // Constraints: active when the target is active and (if skinRequired) the constraint is
        // included by the current skin.
        for c in &mut self.ik_constraints {
            let data = self.data.ik_constraints.get(c.data_index);
            let skin_required = data.map(|d| d.skin_required).unwrap_or(false);
            let in_skin = skin
                .map(|s| s.ik_constraints.contains(&c.data_index))
                .unwrap_or(false);
            let target_active = self.bones.get(c.target).map(|b| b.active).unwrap_or(false);
            c.active = target_active && (!skin_required || in_skin);
        }

        for c in &mut self.transform_constraints {
            let data = self.data.transform_constraints.get(c.data_index);
            let skin_required = data.map(|d| d.skin_required).unwrap_or(false);
            let in_skin = skin
                .map(|s| s.transform_constraints.contains(&c.data_index))
                .unwrap_or(false);
            let source_active = self.bones.get(c.source).map(|b| b.active).unwrap_or(false);
            c.active = source_active && (!skin_required || in_skin);
        }

        for c in &mut self.path_constraints {
            let data = self.data.path_constraints.get(c.data_index);
            let skin_required = data.map(|d| d.skin_required).unwrap_or(false);
            let in_skin = skin
                .map(|s| s.path_constraints.contains(&c.data_index))
                .unwrap_or(false);
            let target_bone_active = self
                .slots
                .get(c.target)
                .and_then(|s| self.bones.get(s.bone))
                .map(|b| b.active)
                .unwrap_or(false);
            c.active = target_bone_active && (!skin_required || in_skin);
        }

        for c in &mut self.physics_constraints {
            let data = self.data.physics_constraints.get(c.data_index);
            let skin_required = data.map(|d| d.skin_required).unwrap_or(false);
            let in_skin = skin
                .map(|s| s.physics_constraints.contains(&c.data_index))
                .unwrap_or(false);
            let bone_active = self.bones.get(c.bone).map(|b| b.active).unwrap_or(false);
            c.active = bone_active && (!skin_required || in_skin);
        }

        for c in &mut self.slider_constraints {
            let data = self.data.slider_constraints.get(c.data_index);
            let skin_required = data.map(|d| d.skin_required).unwrap_or(false);
            let in_skin = skin
                .map(|s| s.slider_constraints.contains(&c.data_index))
                .unwrap_or(false);
            let source_active = data
                .and_then(|d| d.bone)
                .and_then(|i| self.bones.get(i))
                .map(|b| b.active)
                .unwrap_or(true);
            c.active = source_active && (!skin_required || in_skin);
        }

        self.rebuild_update_cache();
        self.update_draw_order_constrained_state();
    }

    pub fn update_cache_items(&self) -> &[UpdateCacheItem] {
        &self.update_cache
    }

    fn rebuild_update_cache(&mut self) {
        self.update_cache = cache::rebuild_update_cache(self);
    }

    fn update_draw_order_constrained_state(&mut self) {
        let constrained = self.slider_constraints.iter().any(|slider| {
            if !slider.is_active() {
                return false;
            }
            self.data
                .slider_constraints
                .get(slider.data_index)
                .and_then(|data| data.animation)
                .and_then(|animation_index| self.data.animations.get(animation_index))
                .is_some_and(|animation| {
                    animation.draw_order_timeline.is_some()
                        || !animation.draw_order_folder_timelines.is_empty()
                })
        });
        self.draw_order_constrained = constrained;
        if constrained {
            self.applied_draw_order.clone_from(&self.draw_order);
        } else {
            self.applied_draw_order.clear();
        }
        self.update_slot_constrained_state();
    }

    fn update_slot_constrained_state(&mut self) {
        let mut constrained = vec![false; self.slots.len()];
        for slider in &self.slider_constraints {
            if !slider.is_active() {
                continue;
            }
            let Some(data) = self.data.slider_constraints.get(slider.data_index) else {
                continue;
            };
            let Some(animation_index) = data.animation else {
                continue;
            };
            let Some(animation) = self.data.animations.get(animation_index) else {
                continue;
            };

            for timeline in &animation.slot_attachment_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.slot_color_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.slot_rgb_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.slot_alpha_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.slot_rgba2_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.slot_rgb2_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.deform_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
            for timeline in &animation.sequence_timelines {
                if let Some(slot) = constrained.get_mut(timeline.slot_index) {
                    *slot = true;
                }
            }
        }

        for (slot, constrained) in self.slots.iter_mut().zip(constrained) {
            slot.set_pose_constrained(constrained);
        }
    }

    pub fn set_skin(&mut self, skin_name: Option<&str>) {
        let old_skin = self.skin.clone();
        match skin_name {
            None => {
                if old_skin.is_none() {
                    return;
                }
                self.skin = None;
            }
            Some(name) => {
                if !self.data.skins.contains_key(name) {
                    return;
                }
                if old_skin.as_deref() == Some(name) {
                    return;
                }
                self.skin = Some(name.to_string());
            }
        }
        let new_skin = self.skin.as_deref().and_then(|n| self.data.find_skin(n));

        // Spine-cpp: when switching from no skin to a skin, the setup attachment names are
        // applied from the new skin.
        if old_skin.is_none() {
            if let (Some(new_skin_name), Some(new_skin)) = (self.skin.as_deref(), new_skin) {
                for slot_index in 0..self.slots.len() {
                    let setup_name = self
                        .data
                        .slots
                        .get(slot_index)
                        .and_then(|s| s.attachment.as_deref());
                    let Some(setup_name) = setup_name else {
                        continue;
                    };
                    if new_skin.get_attachment(slot_index, setup_name).is_some() {
                        let clear_deform = {
                            let slot = &self.slots[slot_index];
                            self.attachment_change_clears_deform(
                                slot_index,
                                slot.attachment.as_deref(),
                                slot.attachment_skin.as_deref(),
                                Some(setup_name),
                                Some(new_skin_name),
                            )
                        };
                        let slot = &mut self.slots[slot_index];
                        slot.attachment = Some(setup_name.to_string());
                        slot.attachment_skin = Some(new_skin_name.to_string());
                        if clear_deform {
                            slot.deform.clear();
                        }
                        slot.sequence_index = -1;
                    }
                }
            }
        } else if let (Some(old_skin_name), Some(new_skin_name), Some(new_skin)) =
            (old_skin.as_deref(), self.skin.as_deref(), new_skin)
        {
            // Spine-cpp: when switching from skin -> skin, perform `attachAll` semantics:
            // attachments currently sourced from the old skin are replaced by attachments from the
            // new skin with the same key (if present). Attachments not present in the new skin are
            // kept as-is.
            for slot_index in 0..self.slots.len() {
                let Some(current_key) = self.slots[slot_index].attachment.clone() else {
                    continue;
                };
                if self.slots[slot_index].attachment_skin.as_deref() != Some(old_skin_name) {
                    continue;
                }
                if new_skin
                    .get_attachment(slot_index, current_key.as_str())
                    .is_some()
                {
                    let clear_deform = self.attachment_change_clears_deform(
                        slot_index,
                        Some(current_key.as_str()),
                        Some(old_skin_name),
                        Some(current_key.as_str()),
                        Some(new_skin_name),
                    );
                    let slot = &mut self.slots[slot_index];
                    slot.attachment_skin = Some(new_skin_name.to_string());
                    if clear_deform {
                        slot.deform.clear();
                    }
                    slot.sequence_index = -1;
                }
            }
        }

        self.update_cache();
    }

    pub fn setup_pose(&mut self) {
        self.setup_pose_bones();
        self.setup_pose_slots();
    }

    pub fn setup_pose_bones(&mut self) {
        for (i, bone) in self.bones.iter_mut().enumerate() {
            let Some(data) = self.data.bones.get(i) else {
                continue;
            };
            bone.inherit = data.inherit;
            bone.x = data.x;
            bone.y = data.y;
            bone.rotation = data.rotation;
            bone.scale_x = data.scale_x;
            bone.scale_y = data.scale_y;
            bone.shear_x = data.shear_x;
            bone.shear_y = data.shear_y;

            bone.ax = data.x;
            bone.ay = data.y;
            bone.arotation = data.rotation;
            bone.ascale_x = data.scale_x;
            bone.ascale_y = data.scale_y;
            bone.ashear_x = data.shear_x;
            bone.ashear_y = data.shear_y;
        }

        for ik in &mut self.ik_constraints {
            if let Some(data) = self.data.ik_constraints.get(ik.data_index) {
                ik.mix = data.mix;
                ik.softness = data.softness;
                ik.compress = data.compress;
                ik.stretch = data.stretch;
                ik.scale_y_mode = data.scale_y_mode;
                ik.bend_direction = data.bend_direction;
            }
        }

        for c in &mut self.transform_constraints {
            if let Some(data) = self.data.transform_constraints.get(c.data_index) {
                c.mix_rotate = data.mix_rotate;
                c.mix_x = data.mix_x;
                c.mix_y = data.mix_y;
                c.mix_scale_x = data.mix_scale_x;
                c.mix_scale_y = data.mix_scale_y;
                c.mix_shear_y = data.mix_shear_y;
            }
        }

        for c in &mut self.path_constraints {
            if let Some(data) = self.data.path_constraints.get(c.data_index) {
                c.position = data.position;
                c.spacing = data.spacing;
                c.mix_rotate = data.mix_rotate;
                c.mix_x = data.mix_x;
                c.mix_y = data.mix_y;
            }
        }

        for c in &mut self.physics_constraints {
            if let Some(data) = self.data.physics_constraints.get(c.data_index) {
                c.inertia = data.inertia;
                c.strength = data.strength;
                c.damping = data.damping;
                c.mass_inverse = data.mass_inverse;
                c.wind = data.wind;
                c.gravity = data.gravity;
                c.mix = data.mix;
            }
        }

        for c in &mut self.slider_constraints {
            if let Some(data) = self.data.slider_constraints.get(c.data_index) {
                c.time = data.setup_time;
                c.mix = data.setup_mix;
            }
        }
    }

    pub fn setup_pose_slots(&mut self) {
        let skin_name = self.skin.as_deref();
        let skin = skin_name.and_then(|n| self.data.find_skin(n));
        let default_skin = if skin_name != Some(crate::SkeletonData::DEFAULT_SKIN_NAME) {
            self.data.get_default_skin()
        } else {
            None
        };

        for (i, slot) in self.slots.iter_mut().enumerate() {
            let Some(data) = self.data.slots.get(i) else {
                continue;
            };
            let setup_name = data.attachment.as_deref();

            slot.color = data.setup_pose.color;
            slot.has_dark = data.setup_pose.has_dark;
            slot.dark_color = data.setup_pose.dark_color;
            slot.sequence_index = data.setup_pose.sequence_index;
            slot.deform.clear();

            match setup_name {
                None => {
                    // Match spine-cpp `Slot::setToSetupPose`:
                    // - When setup attachment is empty, it calls `setAttachment(NULL)`.
                    if slot.attachment.is_some() || slot.attachment_skin.is_some() {
                        slot.attachment = None;
                        slot.attachment_skin = None;
                        slot.deform.clear();
                        slot.sequence_index = -1;
                    } else {
                        slot.attachment = None;
                        slot.attachment_skin = None;
                        slot.sequence_index = 0;
                    }
                }
                Some(name) => {
                    let mut resolved = None;
                    if skin.and_then(|s| s.get_attachment(i, name)).is_some() {
                        resolved = Some((name.to_string(), skin_name.map(|n| n.to_string())));
                    } else if default_skin
                        .and_then(|s| s.get_attachment(i, name))
                        .is_some()
                    {
                        resolved = Some((name.to_string(), Some("default".to_string())));
                    }

                    if let Some((key, source_skin)) = resolved {
                        // Match spine-cpp: `Slot::setToSetupPose` forces `_attachment=NULL` before
                        // calling `setAttachment`, so even if the same attachment is already set
                        // we reset the sequence index to `-1`.
                        slot.attachment = Some(key);
                        slot.attachment_skin = source_skin;
                        slot.deform.clear();
                        slot.sequence_index = -1;
                    } else {
                        // Setup attachment name exists but can't be resolved to an attachment.
                        // Match spine-cpp: it sets `_attachment=NULL` and then `setAttachment(NULL)`
                        // which early-returns, leaving `sequenceIndex` at the setup value.
                        slot.attachment = None;
                        slot.attachment_skin = None;
                        slot.sequence_index = 0;
                    }
                }
            }
            slot.applied_pose = SlotPose::from_slot(slot);
        }

        self.draw_order = (0..self.slots.len()).collect::<Vec<_>>();
        if self.draw_order_constrained {
            self.applied_draw_order.clone_from(&self.draw_order);
        }
        self.reset_constrained_slot_poses();
    }

    pub fn get_attachment(
        &self,
        slot_index: usize,
        attachment_name: &str,
    ) -> Option<&crate::AttachmentData> {
        if attachment_name.is_empty() {
            return None;
        }

        let skin_name = self.skin.as_deref();
        if let Some(skin_name) = skin_name {
            if let Some(skin) = self.data.find_skin(skin_name)
                && let Some(att) = skin.get_attachment(slot_index, attachment_name)
            {
                return Some(att);
            }
            if skin_name != crate::SkeletonData::DEFAULT_SKIN_NAME
                && let Some(default_skin) = self.data.get_default_skin()
                && let Some(att) = default_skin.get_attachment(slot_index, attachment_name)
            {
                return Some(att);
            }
        } else if let Some(default_skin) = self.data.get_default_skin()
            && let Some(att) = default_skin.get_attachment(slot_index, attachment_name)
        {
            return Some(att);
        }

        None
    }

    pub fn set_attachment(&mut self, slot_name: &str, attachment_name: &str) {
        if slot_name.is_empty() {
            return;
        }
        let Some(slot_index) = self.slot_index_by_name(slot_name) else {
            return;
        };

        if attachment_name.is_empty() {
            let clear_deform = {
                let slot = &self.slots[slot_index];
                self.attachment_change_clears_deform(
                    slot_index,
                    slot.attachment.as_deref(),
                    slot.attachment_skin.as_deref(),
                    None,
                    None,
                )
            };
            let slot = &mut self.slots[slot_index];
            if slot.attachment.is_none() && slot.attachment_skin.is_none() {
                return;
            }
            let mut pose = slot.pose_mut_for(false);
            pose.set_attachment(None, None, clear_deform);
            return;
        }

        let Some(source_skin) = self.attachment_source_skin_name(slot_index, attachment_name)
        else {
            return;
        };
        let (old_key, old_skin) = {
            let slot = &self.slots[slot_index];
            (slot.attachment.clone(), slot.attachment_skin.clone())
        };
        if old_key.as_deref() == Some(attachment_name)
            && old_skin.as_deref() == Some(source_skin.as_str())
        {
            return;
        }

        let clear_deform = self.attachment_change_clears_deform(
            slot_index,
            old_key.as_deref(),
            old_skin.as_deref(),
            Some(attachment_name),
            Some(source_skin.as_str()),
        );
        let slot = &mut self.slots[slot_index];
        let mut pose = slot.pose_mut_for(false);
        pose.set_attachment(
            Some(attachment_name.to_string()),
            Some(source_skin),
            clear_deform,
        );
    }

    pub fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut has_vertices = false;

        for &slot_index in self.draw_order() {
            let Some(slot) = self.slots.get(slot_index) else {
                continue;
            };
            let Some(bone) = self.bones.get(slot.bone) else {
                continue;
            };
            if !bone.active {
                continue;
            }

            match self.slot_attachment_data_for_pose(slot_index, true) {
                Some(crate::AttachmentData::Region(region)) => {
                    let mut vertices = [0.0; 8];
                    for (i, (x, y)) in region_world_vertices(region, bone).into_iter().enumerate() {
                        vertices[i * 2] = x;
                        vertices[i * 2 + 1] = y;
                    }
                    include_flat_vertices_in_bounds(
                        &vertices,
                        &mut min_x,
                        &mut min_y,
                        &mut max_x,
                        &mut max_y,
                        &mut has_vertices,
                    );
                }
                Some(crate::AttachmentData::Mesh(_)) => {
                    let Some(vertices) = self
                        .slot_attachment_data_for_pose(slot_index, true)
                        .and_then(|attachment| attachment.compute_world_vertices(self, slot_index))
                    else {
                        continue;
                    };
                    include_flat_vertices_in_bounds(
                        &vertices,
                        &mut min_x,
                        &mut min_y,
                        &mut max_x,
                        &mut max_y,
                        &mut has_vertices,
                    );
                }
                _ => {}
            }
        }

        has_vertices.then_some((min_x, min_y, max_x - min_x, max_y - min_y))
    }

    pub fn bounds_with_clipping(&self) -> Option<(f32, f32, f32, f32)> {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut has_vertices = false;
        let mut clipper = SkeletonClipper::default();
        let mut clip_end_slot = None;

        for &slot_index in self.draw_order() {
            let Some(slot) = self.slots.get(slot_index) else {
                continue;
            };
            let Some(bone) = self.bones.get(slot.bone) else {
                continue;
            };
            if !bone.active {
                continue;
            }

            let Some(attachment) = self.slot_attachment_data_for_pose(slot_index, true) else {
                if clipper.is_clipping() && clip_end_slot == Some(slot_index) {
                    clipper.clip_end();
                    clip_end_slot = None;
                }
                continue;
            };

            match attachment {
                crate::AttachmentData::Region(region) => {
                    let mut vertices = [0.0; 8];
                    for (i, (x, y)) in region_world_vertices(region, bone).into_iter().enumerate() {
                        vertices[i * 2] = x;
                        vertices[i * 2 + 1] = y;
                    }

                    if clipper.is_clipping() {
                        let uvs = [0.0; 8];
                        let indices = [0_u16, 1, 2, 2, 3, 0];
                        let (clipped, _, _) = clipper.clip_triangles(&vertices, &indices, &uvs, 2);
                        include_flat_vertices_in_bounds(
                            &clipped,
                            &mut min_x,
                            &mut min_y,
                            &mut max_x,
                            &mut max_y,
                            &mut has_vertices,
                        );
                    } else {
                        include_flat_vertices_in_bounds(
                            &vertices,
                            &mut min_x,
                            &mut min_y,
                            &mut max_x,
                            &mut max_y,
                            &mut has_vertices,
                        );
                    }
                }
                crate::AttachmentData::Mesh(mesh) => {
                    let Some(vertices) = self
                        .slot_attachment_data_for_pose(slot_index, true)
                        .and_then(|attachment| attachment.compute_world_vertices(self, slot_index))
                    else {
                        continue;
                    };

                    if clipper.is_clipping() {
                        let mut indices = Vec::with_capacity(mesh.triangles.len());
                        for &index in &mesh.triangles {
                            let Ok(index) = u16::try_from(index) else {
                                indices.clear();
                                break;
                            };
                            indices.push(index);
                        }

                        if indices.is_empty() {
                            include_flat_vertices_in_bounds(
                                &vertices,
                                &mut min_x,
                                &mut min_y,
                                &mut max_x,
                                &mut max_y,
                                &mut has_vertices,
                            );
                        } else {
                            let uvs = vec![0.0; vertices.len()];
                            let (clipped, _, _) =
                                clipper.clip_triangles(&vertices, &indices, &uvs, 2);
                            include_flat_vertices_in_bounds(
                                &clipped,
                                &mut min_x,
                                &mut min_y,
                                &mut max_x,
                                &mut max_y,
                                &mut has_vertices,
                            );
                        }
                    } else {
                        include_flat_vertices_in_bounds(
                            &vertices,
                            &mut min_x,
                            &mut min_y,
                            &mut max_x,
                            &mut max_y,
                            &mut has_vertices,
                        );
                    }
                }
                crate::AttachmentData::Clipping(clip) => {
                    if clipper.is_clipping() && clip_end_slot == Some(slot_index) {
                        clipper.clip_end();
                        clip_end_slot = None;
                    }

                    let Some(vertices) = self
                        .slot_attachment_data_for_pose(slot_index, true)
                        .and_then(|attachment| attachment.compute_world_vertices(self, slot_index))
                    else {
                        continue;
                    };
                    if clipper.clip_start(&vertices, clip.convex, clip.inverse) {
                        clip_end_slot = clip.end_slot;
                    }
                    continue;
                }
                _ => {}
            }

            if clipper.is_clipping() && clip_end_slot == Some(slot_index) {
                clipper.clip_end();
                clip_end_slot = None;
            }
        }

        clipper.clip_end();
        has_vertices.then_some((min_x, min_y, max_x - min_x, max_y - min_y))
    }

    pub(crate) fn attachment_source_skin_name(
        &self,
        slot_index: usize,
        attachment_name: &str,
    ) -> Option<String> {
        if attachment_name.is_empty() {
            return None;
        }

        if let Some(skin_name) = self.skin.as_deref() {
            if self
                .data
                .find_skin(skin_name)
                .and_then(|skin| skin.get_attachment(slot_index, attachment_name))
                .is_some()
            {
                return Some(skin_name.to_string());
            }
            if skin_name != crate::SkeletonData::DEFAULT_SKIN_NAME
                && self
                    .data
                    .get_default_skin()
                    .and_then(|skin| skin.get_attachment(slot_index, attachment_name))
                    .is_some()
            {
                return Some("default".to_string());
            }
            return None;
        }

        if self
            .data
            .get_default_skin()
            .and_then(|skin| skin.get_attachment(slot_index, attachment_name))
            .is_some()
        {
            return Some("default".to_string());
        }

        None
    }

    pub(crate) fn attachment_timeline_key(
        &self,
        slot_index: usize,
        source_skin: &str,
        key: &str,
    ) -> Option<(bool, String, String)> {
        let skin = self.data.find_skin(source_skin)?;
        let attachment = skin.get_attachment(slot_index, key)?;
        match attachment {
            crate::AttachmentData::Mesh(mesh) => Some((
                true,
                mesh.timeline_skin.clone(),
                mesh.timeline_attachment.clone(),
            )),
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

    pub(crate) fn attachment_change_clears_deform(
        &self,
        slot_index: usize,
        old_key: Option<&str>,
        old_skin: Option<&str>,
        new_key: Option<&str>,
        new_skin: Option<&str>,
    ) -> bool {
        if old_key == new_key && old_skin == new_skin {
            return false;
        }

        match (
            old_key
                .zip(old_skin)
                .and_then(|(key, skin)| self.attachment_timeline_key(slot_index, skin, key)),
            new_key
                .zip(new_skin)
                .and_then(|(key, skin)| self.attachment_timeline_key(slot_index, skin, key)),
        ) {
            (Some((old_vertex, old_skin, old_key)), Some((new_vertex, new_skin, new_key))) => {
                !(old_vertex && new_vertex && old_skin == new_skin && old_key == new_key)
            }
            _ => true,
        }
    }

    /// Computes full world vertices for the current vertex attachment in a slot.
    ///
    /// This is the Rust runtime equivalent of calling C++ `VertexAttachment::computeWorldVertices`
    /// with the slot's current attachment. Region and point attachments return `None`.
    pub fn update_world_transform_with_physics(&mut self, physics: Physics) {
        self.update_epoch = self.update_epoch.wrapping_add(1);
        if self.draw_order_constrained {
            self.applied_draw_order.clone_from(&self.draw_order);
        }
        self.reset_constrained_slot_poses();
        self.reset_applied_transforms();

        let cache = std::mem::take(&mut self.update_cache);
        for item in cache.iter().copied() {
            match item {
                UpdateCacheItem::Bone(bone_index) => self.update_bone_world_transform(bone_index),
                UpdateCacheItem::Ik(index) => {
                    self.apply_ik_constraint(index);
                }
                UpdateCacheItem::Transform(index) => {
                    self.apply_transform_constraint(index);
                }
                UpdateCacheItem::Path(index) => {
                    self.apply_path_constraint(index);
                }
                UpdateCacheItem::Physics(index) => {
                    self.apply_physics_constraint(index, physics);
                }
                UpdateCacheItem::Slider(index) => {
                    self.apply_slider_constraint(index);
                }
            }
        }
        self.update_cache = cache;
    }

    fn reset_applied_transforms(&mut self) {
        for bone in &mut self.bones {
            bone.ax = bone.x;
            bone.ay = bone.y;
            bone.arotation = bone.rotation;
            bone.ascale_x = bone.scale_x;
            bone.ascale_y = bone.scale_y;
            bone.ashear_x = bone.shear_x;
            bone.ashear_y = bone.shear_y;
            bone.local_epoch = 0;
        }
    }

    /// Computes one bone's world transform from its applied local transform.
    ///
    /// Mirrors `BonePose::updateWorldTransform` in the official runtime. Child
    /// bones are not updated by this method.
    pub fn update_bone_world_transform(&mut self, bone_index: usize) {
        bone::update_world_transform(self, bone_index);
    }

    /// Computes one bone's applied local transform from its world transform.
    ///
    /// Mirrors `BonePose::updateLocalTransform` in the official runtime.
    pub fn update_bone_local_transform(&mut self, bone_index: usize) {
        bone::update_applied_transform(self, bone_index);
        if let Some(bone) = self.bones.get_mut(bone_index) {
            bone.world_epoch = self.update_epoch;
        }
    }

    /// Computes one bone's applied local transform if its world transform has
    /// been marked modified in the current update epoch.
    ///
    /// Mirrors `BonePose::validateLocalTransform` in the official runtime.
    pub fn validate_bone_local_transform(&mut self, bone_index: usize) {
        if self
            .bones
            .get(bone_index)
            .is_some_and(|bone| bone.local_epoch == self.update_epoch)
        {
            bone::update_applied_transform(self, bone_index);
            if let Some(bone) = self.bones.get_mut(bone_index) {
                bone.world_epoch = self.update_epoch;
            }
        }
    }

    /// Marks one bone's applied local transform as modified for the current
    /// update epoch, invalidating descendants that were already updated.
    ///
    /// Mirrors `BonePose::modifyLocal` in the official runtime.
    pub fn modify_bone_local(&mut self, bone_index: usize) {
        self.bone_modify_local(bone_index);
    }

    /// Marks one bone's world transform as modified for the current update
    /// epoch, invalidating descendants that were already updated.
    ///
    /// Mirrors `BonePose::modifyWorld` in the official runtime without
    /// exposing the raw update counter.
    pub fn modify_bone_world(&mut self, bone_index: usize) {
        self.bone_modify_world(bone_index);
    }

    fn apply_ik_constraint(&mut self, constraint_index: usize) -> bool {
        ik::apply(self, constraint_index)
    }

    fn apply_path_constraint(&mut self, constraint_index: usize) -> bool {
        path::apply(self, constraint_index)
    }

    fn apply_slider_constraint(&mut self, constraint_index: usize) -> bool {
        slider::apply(self, constraint_index)
    }

    fn apply_transform_constraint(&mut self, constraint_index: usize) -> bool {
        transform::apply(self, constraint_index)
    }

    fn apply_physics_constraint(&mut self, constraint_index: usize, physics_mode: Physics) -> bool {
        physics::apply(self, constraint_index, physics_mode)
    }

    fn update_applied_transform(&mut self, bone_index: usize) {
        bone::update_applied_transform(self, bone_index);
    }
}

fn wrap_pi(mut radians: f32) -> f32 {
    const PI: f32 = std::f32::consts::PI;
    const PI2: f32 = 2.0 * std::f32::consts::PI;
    if radians > PI {
        radians -= PI2;
    } else if radians < -PI {
        radians += PI2;
    }
    radians
}

fn build_bone_children_indices(bones: &[Bone]) -> Vec<Vec<usize>> {
    let mut children = vec![Vec::<usize>::new(); bones.len()];
    for (index, bone) in bones.iter().enumerate() {
        if let Some(parent) = bone.parent
            && parent < children.len()
        {
            children[parent].push(index);
        }
    }
    children
}
