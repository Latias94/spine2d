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
pub use ik::IkConstraint;
pub use path::PathConstraint;
pub use physics::{Physics, PhysicsConstraint};
pub use slider::SliderConstraint;
pub use slot::Slot;
pub use transform::TransformConstraint;

use crate::SkeletonData;
use cache::UpdateCacheItem;
use path::{PathConstraintScratch, estimate_path_attachment_scratch_capacities};
use std::sync::Arc;

fn atan2_degrees(y: f32, x: f32) -> f32 {
    atan2_radians(y, x).to_degrees()
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
        atan2_degrees(bone.c, bone.a) + self.rotation
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

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub(crate) data: Arc<SkeletonData>,
    pub(crate) bones: Vec<Bone>,
    bone_children: Vec<Vec<usize>>,
    pub(crate) slots: Vec<Slot>,
    pub(crate) draw_order: Vec<usize>,
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
                sequence_index: 0,
                deform: Vec::new(),
                color: slot.color,
                has_dark: slot.has_dark,
                dark_color: slot.dark_color,
                blend: slot.blend,
            })
            .collect::<Vec<_>>();

        let draw_order = (0..slots.len()).collect::<Vec<_>>();
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

        fn collect_animation_bones(animation: &crate::Animation) -> Vec<usize> {
            let mut out = Vec::<usize>::new();
            for tl in &animation.bone_timelines {
                let bone_index = match tl {
                    crate::BoneTimeline::Rotate(t) => t.bone_index,
                    crate::BoneTimeline::Translate(t) => t.bone_index,
                    crate::BoneTimeline::TranslateX(t) => t.bone_index,
                    crate::BoneTimeline::TranslateY(t) => t.bone_index,
                    crate::BoneTimeline::Scale(t) => t.bone_index,
                    crate::BoneTimeline::ScaleX(t) => t.bone_index,
                    crate::BoneTimeline::ScaleY(t) => t.bone_index,
                    crate::BoneTimeline::Shear(t) => t.bone_index,
                    crate::BoneTimeline::ShearX(t) => t.bone_index,
                    crate::BoneTimeline::ShearY(t) => t.bone_index,
                    crate::BoneTimeline::Inherit(t) => t.bone_index,
                };
                out.push(bone_index);
            }
            out.sort_unstable();
            out.dedup();
            out
        }

        let slider_constraints = data
            .slider_constraints
            .iter()
            .enumerate()
            .map(|(data_index, c)| {
                let animation_bones = c
                    .animation
                    .and_then(|idx| data.animations.get(idx))
                    .map(collect_animation_bones)
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

    pub fn root_bone(&self) -> Option<&Bone> {
        self.bones.first()
    }

    pub fn root_bone_mut(&mut self) -> Option<&mut Bone> {
        self.bones.first_mut()
    }

    pub fn find_bone_index(&self, bone_name: &str) -> Option<usize> {
        if bone_name.is_empty() {
            return None;
        }
        self.data
            .bones
            .iter()
            .position(|bone| bone.name == bone_name)
    }

    pub fn find_bone(&self, bone_name: &str) -> Option<&Bone> {
        let index = self.find_bone_index(bone_name)?;
        self.bones.get(index)
    }

    pub fn find_bone_mut(&mut self, bone_name: &str) -> Option<&mut Bone> {
        let index = self.find_bone_index(bone_name)?;
        self.bones.get_mut(index)
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn slots_mut(&mut self) -> &mut [Slot] {
        &mut self.slots
    }

    pub fn find_slot_index(&self, slot_name: &str) -> Option<usize> {
        if slot_name.is_empty() {
            return None;
        }
        self.data
            .slots
            .iter()
            .position(|slot| slot.name == slot_name)
    }

    pub fn find_slot(&self, slot_name: &str) -> Option<&Slot> {
        let index = self.find_slot_index(slot_name)?;
        self.slots.get(index)
    }

    pub fn find_slot_mut(&mut self, slot_name: &str) -> Option<&mut Slot> {
        let index = self.find_slot_index(slot_name)?;
        self.slots.get_mut(index)
    }

    pub fn draw_order(&self) -> &[usize] {
        &self.draw_order
    }

    pub fn draw_order_mut(&mut self) -> &mut [usize] {
        &mut self.draw_order
    }

    pub fn skin(&self) -> Option<&str> {
        self.skin.as_deref()
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn ik_constraints(&self) -> &[IkConstraint] {
        &self.ik_constraints
    }

    pub fn ik_constraints_mut(&mut self) -> &mut [IkConstraint] {
        &mut self.ik_constraints
    }

    pub fn find_ik_constraint_index(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .ik_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_ik_constraint(&self, constraint_name: &str) -> Option<&IkConstraint> {
        let index = self.find_ik_constraint_index(constraint_name)?;
        self.ik_constraints.get(index)
    }

    pub fn find_ik_constraint_mut(&mut self, constraint_name: &str) -> Option<&mut IkConstraint> {
        let index = self.find_ik_constraint_index(constraint_name)?;
        self.ik_constraints.get_mut(index)
    }

    pub fn transform_constraints(&self) -> &[TransformConstraint] {
        &self.transform_constraints
    }

    pub fn transform_constraints_mut(&mut self) -> &mut [TransformConstraint] {
        &mut self.transform_constraints
    }

    pub fn find_transform_constraint_index(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .transform_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_transform_constraint(&self, constraint_name: &str) -> Option<&TransformConstraint> {
        let index = self.find_transform_constraint_index(constraint_name)?;
        self.transform_constraints.get(index)
    }

    pub fn find_transform_constraint_mut(
        &mut self,
        constraint_name: &str,
    ) -> Option<&mut TransformConstraint> {
        let index = self.find_transform_constraint_index(constraint_name)?;
        self.transform_constraints.get_mut(index)
    }

    pub fn path_constraints(&self) -> &[PathConstraint] {
        &self.path_constraints
    }

    pub fn path_constraints_mut(&mut self) -> &mut [PathConstraint] {
        &mut self.path_constraints
    }

    pub fn find_path_constraint_index(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .path_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_path_constraint(&self, constraint_name: &str) -> Option<&PathConstraint> {
        let index = self.find_path_constraint_index(constraint_name)?;
        self.path_constraints.get(index)
    }

    pub fn find_path_constraint_mut(
        &mut self,
        constraint_name: &str,
    ) -> Option<&mut PathConstraint> {
        let index = self.find_path_constraint_index(constraint_name)?;
        self.path_constraints.get_mut(index)
    }

    pub fn physics_constraints(&self) -> &[PhysicsConstraint] {
        &self.physics_constraints
    }

    pub fn physics_constraints_mut(&mut self) -> &mut [PhysicsConstraint] {
        &mut self.physics_constraints
    }

    pub fn find_physics_constraint_index(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .physics_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_physics_constraint(&self, constraint_name: &str) -> Option<&PhysicsConstraint> {
        let index = self.find_physics_constraint_index(constraint_name)?;
        self.physics_constraints.get(index)
    }

    pub fn find_physics_constraint_mut(
        &mut self,
        constraint_name: &str,
    ) -> Option<&mut PhysicsConstraint> {
        let index = self.find_physics_constraint_index(constraint_name)?;
        self.physics_constraints.get_mut(index)
    }

    pub fn slider_constraints(&self) -> &[SliderConstraint] {
        &self.slider_constraints
    }

    pub fn slider_constraints_mut(&mut self) -> &mut [SliderConstraint] {
        &mut self.slider_constraints
    }

    pub fn find_slider_constraint_index(&self, constraint_name: &str) -> Option<usize> {
        if constraint_name.is_empty() {
            return None;
        }
        self.data
            .slider_constraints
            .iter()
            .position(|constraint| constraint.name == constraint_name)
    }

    pub fn find_slider_constraint(&self, constraint_name: &str) -> Option<&SliderConstraint> {
        let index = self.find_slider_constraint_index(constraint_name)?;
        self.slider_constraints.get(index)
    }

    pub fn find_slider_constraint_mut(
        &mut self,
        constraint_name: &str,
    ) -> Option<&mut SliderConstraint> {
        let index = self.find_slider_constraint_index(constraint_name)?;
        self.slider_constraints.get_mut(index)
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

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn wind(&self) -> (f32, f32) {
        (self.wind_x, self.wind_y)
    }

    pub fn set_wind(&mut self, x: f32, y: f32) {
        if x.is_finite() && y.is_finite() {
            self.wind_x = x;
            self.wind_y = y;
        }
    }

    pub fn gravity(&self) -> (f32, f32) {
        (self.gravity_x, self.gravity_y)
    }

    pub fn set_gravity(&mut self, x: f32, y: f32) {
        if x.is_finite() && y.is_finite() {
            self.gravity_x = x;
            self.gravity_y = y;
        }
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
        if time.is_finite() {
            self.time = time;
        }
    }

    pub fn update(&mut self, delta: f32) {
        if delta.is_finite() && delta >= 0.0 {
            self.time += delta;
        }
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

        let skin = self.skin.as_deref().and_then(|n| self.data.skin(n));
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
    }

    #[doc(hidden)]
    pub fn debug_update_cache(&self) -> Vec<String> {
        cache::debug_update_cache(self)
    }

    fn rebuild_update_cache(&mut self) {
        self.update_cache = cache::rebuild_update_cache(self);
    }

    pub fn set_skin(&mut self, skin_name: Option<&str>) -> Result<(), crate::Error> {
        let old_skin = self.skin.clone();
        match skin_name {
            None => {
                self.skin = None;
            }
            Some(name) => {
                if self.data.skins.contains_key(name) {
                    self.skin = Some(name.to_string());
                } else {
                    return Err(crate::Error::UnknownSkin {
                        name: name.to_string(),
                    });
                }
            }
        }
        let new_skin = self.skin.as_deref().and_then(|n| self.data.skin(n));

        // Spine-cpp: when switching from no skin to a skin, the setup attachment names are
        // applied from the new skin.
        if old_skin.is_none() {
            if let Some(new_skin) = new_skin {
                for (slot_index, slot) in self.slots.iter_mut().enumerate() {
                    let setup_name = self
                        .data
                        .slots
                        .get(slot_index)
                        .and_then(|s| s.attachment.as_deref());
                    let Some(setup_name) = setup_name else {
                        continue;
                    };
                    if new_skin.attachment(slot_index, setup_name).is_some() {
                        slot.attachment = Some(setup_name.to_string());
                        slot.attachment_skin = self.skin.clone();
                        slot.deform.clear();
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
            for (slot_index, slot) in self.slots.iter_mut().enumerate() {
                let Some(current_key) = slot.attachment.as_deref() else {
                    continue;
                };
                if slot.attachment_skin.as_deref() != Some(old_skin_name) {
                    continue;
                }
                if new_skin.attachment(slot_index, current_key).is_some() {
                    slot.attachment_skin = Some(new_skin_name.to_string());
                    slot.deform.clear();
                    slot.sequence_index = -1;
                }
            }
        }

        self.update_cache();
        Ok(())
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
        let skin = skin_name.and_then(|n| self.data.skin(n));
        let default_skin = if skin_name != Some("default") {
            self.data.skin("default")
        } else {
            None
        };

        for (i, slot) in self.slots.iter_mut().enumerate() {
            let Some(data) = self.data.slots.get(i) else {
                continue;
            };
            let setup_name = data.attachment.as_deref();

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
                    if skin.and_then(|s| s.attachment(i, name)).is_some() {
                        resolved = Some((name.to_string(), skin_name.map(|n| n.to_string())));
                    } else if default_skin.and_then(|s| s.attachment(i, name)).is_some() {
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

            slot.color = data.color;
            slot.has_dark = data.has_dark;
            slot.dark_color = data.dark_color;
            slot.blend = data.blend;
        }

        self.draw_order = (0..self.slots.len()).collect::<Vec<_>>();
    }

    pub fn attachment(
        &self,
        slot_index: usize,
        attachment_name: &str,
    ) -> Option<&crate::AttachmentData> {
        if attachment_name.is_empty() {
            return None;
        }

        let skin_name = self.skin.as_deref();
        if let Some(skin_name) = skin_name {
            if let Some(skin) = self.data.skin(skin_name)
                && let Some(att) = skin.attachment(slot_index, attachment_name)
            {
                return Some(att);
            }
            if skin_name != "default"
                && let Some(default_skin) = self.data.skin("default")
                && let Some(att) = default_skin.attachment(slot_index, attachment_name)
            {
                return Some(att);
            }
        } else if let Some(default_skin) = self.data.skin("default")
            && let Some(att) = default_skin.attachment(slot_index, attachment_name)
        {
            return Some(att);
        }

        None
    }

    pub fn attachment_by_slot_name(
        &self,
        slot_name: &str,
        attachment_name: &str,
    ) -> Option<&crate::AttachmentData> {
        let slot_index = self.find_slot_index(slot_name)?;
        self.attachment(slot_index, attachment_name)
    }

    pub fn set_attachment(&mut self, slot_name: &str, attachment_name: &str) -> bool {
        if slot_name.is_empty() {
            return false;
        }
        let Some(slot_index) = self.find_slot_index(slot_name) else {
            return false;
        };

        if attachment_name.is_empty() {
            let slot = &mut self.slots[slot_index];
            if slot.attachment.is_none() && slot.attachment_skin.is_none() {
                return false;
            }
            slot.attachment = None;
            slot.attachment_skin = None;
            slot.deform.clear();
            slot.sequence_index = -1;
            return true;
        }

        let Some(source_skin) = self.attachment_source_skin_name(slot_index, attachment_name)
        else {
            return false;
        };
        let slot = &mut self.slots[slot_index];
        if slot.attachment.as_deref() == Some(attachment_name)
            && slot.attachment_skin.as_deref() == Some(source_skin.as_str())
        {
            return false;
        }

        slot.attachment = Some(attachment_name.to_string());
        slot.attachment_skin = Some(source_skin);
        slot.deform.clear();
        slot.sequence_index = -1;
        true
    }

    pub fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut has_vertices = false;

        for &slot_index in &self.draw_order {
            let Some(slot) = self.slots.get(slot_index) else {
                continue;
            };
            let Some(bone) = self.bones.get(slot.bone) else {
                continue;
            };
            if !bone.active {
                continue;
            }

            match self.slot_attachment_data(slot_index) {
                Some(crate::AttachmentData::Region(region)) => {
                    for (x, y) in region_world_vertices(region, bone) {
                        has_vertices = true;
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
                Some(crate::AttachmentData::Mesh(_)) => {
                    let Some(vertices) = self.slot_vertex_attachment_world_vertices(slot_index)
                    else {
                        continue;
                    };
                    for point in vertices.chunks_exact(2) {
                        let x = point[0];
                        let y = point[1];
                        has_vertices = true;
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
                _ => {}
            }
        }

        has_vertices.then_some((min_x, min_y, max_x - min_x, max_y - min_y))
    }

    fn attachment_source_skin_name(
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
                .skin(skin_name)
                .and_then(|skin| skin.attachment(slot_index, attachment_name))
                .is_some()
            {
                return Some(skin_name.to_string());
            }
            if skin_name != "default"
                && self
                    .data
                    .skin("default")
                    .and_then(|skin| skin.attachment(slot_index, attachment_name))
                    .is_some()
            {
                return Some("default".to_string());
            }
            return None;
        }

        if self
            .data
            .skin("default")
            .and_then(|skin| skin.attachment(slot_index, attachment_name))
            .is_some()
        {
            return Some("default".to_string());
        }

        None
    }

    pub fn slot_attachment_data(&self, slot_index: usize) -> Option<&crate::AttachmentData> {
        let slot = self.slots.get(slot_index)?;
        let key = slot.attachment.as_deref()?;

        if let Some(source_skin) = slot.attachment_skin.as_deref()
            && let Some(skin) = self.data.skin(source_skin)
            && let Some(att) = skin.attachment(slot_index, key)
        {
            return Some(att);
        }

        self.attachment(slot_index, key)
    }

    #[doc(hidden)]
    pub fn slot_vertex_attachment_world_vertices(&self, slot_index: usize) -> Option<Vec<f32>> {
        let attachment = self.slot_attachment_data(slot_index)?;
        let vertices = match attachment {
            crate::AttachmentData::Mesh(a) => &a.vertices,
            crate::AttachmentData::Point(_) => return None,
            crate::AttachmentData::Path(a) => &a.vertices,
            crate::AttachmentData::BoundingBox(a) => &a.vertices,
            crate::AttachmentData::Clipping(a) => &a.vertices,
            crate::AttachmentData::Region(_) => return None,
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
            self,
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

    pub fn update_world_transform(&mut self) {
        self.update_world_transform_with_physics(Physics::None);
    }

    pub fn update_world_transform_with_physics(&mut self, physics: Physics) {
        self.update_epoch = self.update_epoch.wrapping_add(1);
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

    fn update_bone_world_transform(&mut self, bone_index: usize) {
        bone::update_world_transform(self, bone_index);
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
