mod bone;
mod cache;
mod ik;
mod path;
mod physics;
mod slider;
mod transform;

use crate::SkeletonData;
use cache::UpdateCacheItem;
use path::{
    PathConstraintScratch, compute_path_world_positions,
    estimate_path_attachment_scratch_capacities, path_attachment_for_slot,
};
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

#[derive(Clone, Debug)]
pub struct Bone {
    data_index: usize,
    parent: Option<usize>,

    pub inherit: crate::Inherit,
    pub active: bool,

    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub shear_x: f32,
    pub shear_y: f32,

    pub ax: f32,
    pub ay: f32,
    pub arotation: f32,
    pub ascale_x: f32,
    pub ascale_y: f32,
    pub ashear_x: f32,
    pub ashear_y: f32,

    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub world_x: f32,
    pub world_y: f32,

    world_epoch: u32,
    local_epoch: u32,
}

impl Bone {
    pub fn data_index(&self) -> usize {
        self.data_index
    }

    pub fn parent_index(&self) -> Option<usize> {
        self.parent
    }
}

#[derive(Clone, Debug)]
pub struct IkConstraint {
    data_index: usize,
    pub bones: Vec<usize>,
    pub target: usize,
    pub scale_y_mode: crate::ScaleYMode,
    pub mix: f32,
    pub softness: f32,
    pub compress: bool,
    pub stretch: bool,
    pub bend_direction: i32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct TransformConstraint {
    data_index: usize,
    pub bones: Vec<usize>,
    pub source: usize,
    pub mix_rotate: f32,
    pub mix_x: f32,
    pub mix_y: f32,
    pub mix_scale_x: f32,
    pub mix_scale_y: f32,
    pub mix_shear_y: f32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct PathConstraint {
    data_index: usize,
    pub bones: Vec<usize>,
    pub target: usize, // slot index
    pub position: f32,
    pub spacing: f32,
    pub mix_rotate: f32,
    pub mix_x: f32,
    pub mix_y: f32,
    pub active: bool,
}

/// Determines how physics and other non-deterministic updates are applied.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Physics {
    /// Physics are not updated or applied.
    None,
    /// Physics are reset to the current pose.
    Reset,
    /// Physics are updated and the pose from physics is applied.
    Update,
    /// Physics are not updated but the pose from physics is applied.
    Pose,
}

#[derive(Clone, Debug)]
pub struct PhysicsConstraint {
    data_index: usize,
    pub bone: usize,

    pub inertia: f32,
    pub strength: f32,
    pub damping: f32,
    pub mass_inverse: f32,
    pub wind: f32,
    pub gravity: f32,
    pub mix: f32,
    pub scale_y_mode: crate::ScaleYMode,

    pub reset: bool,
    pub ux: f32,
    pub uy: f32,
    pub cx: f32,
    pub cy: f32,
    pub tx: f32,
    pub ty: f32,
    pub x_offset: f32,
    pub x_lag: f32,
    pub x_velocity: f32,
    pub y_offset: f32,
    pub y_lag: f32,
    pub y_velocity: f32,
    pub rotate_offset: f32,
    pub rotate_lag: f32,
    pub rotate_velocity: f32,
    pub scale_offset: f32,
    pub scale_lag: f32,
    pub scale_velocity: f32,

    pub active: bool,
    pub remaining: f32,
    pub last_time: f32,
}

#[derive(Clone, Debug)]
pub struct SliderConstraint {
    pub(crate) data_index: usize,
    pub time: f32,
    pub mix: f32,
    pub active: bool,
    animation_bones: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Slot {
    data_index: usize,
    pub bone: usize,
    pub attachment: Option<String>,
    pub(crate) attachment_skin: Option<String>,
    pub(crate) attachment_state: i32,
    pub sequence_index: i32,
    pub deform: Vec<f32>,
    pub color: [f32; 4],
    pub has_dark: bool,
    pub dark_color: [f32; 3],
    pub blend: crate::BlendMode,
}

impl Slot {
    pub fn data_index(&self) -> usize {
        self.data_index
    }
}

impl PhysicsConstraint {
    pub fn data_index(&self) -> usize {
        self.data_index
    }

    pub(crate) fn reset_with_time(&mut self, time: f32) {
        self.remaining = 0.0;
        self.last_time = time;
        self.reset = true;
        self.x_offset = 0.0;
        self.x_lag = 0.0;
        self.x_velocity = 0.0;
        self.y_offset = 0.0;
        self.y_lag = 0.0;
        self.y_velocity = 0.0;
        self.rotate_offset = 0.0;
        self.rotate_lag = 0.0;
        self.rotate_velocity = 0.0;
        self.scale_offset = 0.0;
        self.scale_lag = 0.0;
        self.scale_velocity = 0.0;
    }
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

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub data: Arc<SkeletonData>,
    pub bones: Vec<Bone>,
    bone_children: Vec<Vec<usize>>,
    pub slots: Vec<Slot>,
    pub draw_order: Vec<usize>,
    pub skin: Option<String>,
    pub color: [f32; 4],
    wind_x: f32,
    wind_y: f32,
    gravity_x: f32,
    gravity_y: f32,
    pub ik_constraints: Vec<IkConstraint>,
    pub transform_constraints: Vec<TransformConstraint>,
    pub path_constraints: Vec<PathConstraint>,
    pub physics_constraints: Vec<PhysicsConstraint>,
    pub slider_constraints: Vec<SliderConstraint>,
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
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
        out.set_to_setup_pose();
        out.update_cache();
        out
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

    pub fn set_to_setup_pose(&mut self) {
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
                    // - If the slot already has `NULL`, `setAttachment` early returns and does not
                    //   modify `sequenceIndex`.
                    if slot.attachment.is_some() || slot.attachment_skin.is_some() {
                        slot.attachment = None;
                        slot.attachment_skin = None;
                        slot.deform.clear();
                        slot.sequence_index = -1;
                    } else {
                        slot.attachment = None;
                        slot.attachment_skin = None;
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
                        // which early-returns, leaving `sequenceIndex` unchanged.
                        slot.attachment = None;
                        slot.attachment_skin = None;
                    }
                }
            }

            slot.color = data.color;
            slot.has_dark = data.has_dark;
            slot.dark_color = data.dark_color;
            slot.blend = data.blend;
        }

        self.draw_order = (0..self.slots.len()).collect::<Vec<_>>();

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

    pub fn attachment(
        &self,
        slot_index: usize,
        attachment_name: &str,
    ) -> Option<&crate::AttachmentData> {
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
        compute_attachment_world_vertices(
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
        const EPSILON: f32 = 1.0e-5;

        if constraint_index >= self.path_constraints.len()
            || constraint_index >= self.path_constraint_scratch.len()
        {
            return false;
        }

        let (data_index, target, position, spacing, mix_rotate, mix_x, mix_y, bone_count) = {
            let c = &self.path_constraints[constraint_index];
            (
                c.data_index,
                c.target,
                c.position,
                c.spacing,
                c.mix_rotate,
                c.mix_x,
                c.mix_y,
                c.bones.len(),
            )
        };

        let Some(data) = self.data.path_constraints.get(data_index) else {
            return false;
        };
        if mix_rotate == 0.0 && mix_x == 0.0 && mix_y == 0.0 {
            return false;
        }

        let tangents = data.rotate_mode == crate::RotateMode::Tangent;
        let scale = data.rotate_mode == crate::RotateMode::ChainScale;
        if bone_count == 0 {
            return false;
        }
        let spaces_count = if tangents { bone_count } else { bone_count + 1 };

        // Reduce per-frame allocations: avoid cloning the bone index list.
        let bones = std::mem::take(&mut self.path_constraints[constraint_index].bones);

        let mut scratch = std::mem::take(&mut self.path_constraint_scratch[constraint_index]);

        let applied = 'applied: {
            let Some((target_slot_index, path)) = path_attachment_for_slot(self, target) else {
                break 'applied false;
            };
            scratch.spaces.resize(spaces_count, 0.0);
            scratch.spaces.fill(0.0);
            scratch.lengths.clear();
            if scale {
                scratch.lengths.resize(bone_count, 0.0);
            }
            let spaces = scratch.spaces.as_mut_slice();
            let lengths = scratch.lengths.as_mut_slice();

            match data.spacing_mode {
                crate::SpacingMode::Percent => {
                    if scale {
                        for i in 0..spaces_count.saturating_sub(1) {
                            let Some(bone_index) = bones.get(i).copied() else {
                                continue;
                            };
                            let setup_length = self
                                .data
                                .bones
                                .get(bone_index)
                                .map(|b| b.length)
                                .unwrap_or(0.0);
                            let Some(bone) = self.bones.get(bone_index) else {
                                continue;
                            };
                            let x = setup_length * bone.a;
                            let y = setup_length * bone.c;
                            if let Some(out) = lengths.get_mut(i) {
                                *out = (x * x + y * y).sqrt();
                            }
                        }
                    }
                    for space in spaces.iter_mut().take(spaces_count).skip(1) {
                        *space = spacing;
                    }
                }
                crate::SpacingMode::Proportional => {
                    let mut sum = 0.0f32;
                    let mut i = 0usize;
                    let n = spaces_count.saturating_sub(1);
                    while i < n {
                        let Some(bone_index) = bones.get(i).copied() else {
                            i += 1;
                            continue;
                        };
                        let setup_length = self
                            .data
                            .bones
                            .get(bone_index)
                            .map(|b| b.length)
                            .unwrap_or(0.0);
                        if setup_length < EPSILON {
                            if scale && let Some(out) = lengths.get_mut(i) {
                                *out = 0.0;
                            }
                            i += 1;
                            spaces[i] = spacing;
                            continue;
                        }
                        let Some(bone) = self.bones.get(bone_index) else {
                            i += 1;
                            continue;
                        };
                        let x = setup_length * bone.a;
                        let y = setup_length * bone.c;
                        let length = (x * x + y * y).sqrt();
                        if scale && let Some(out) = lengths.get_mut(i) {
                            *out = length;
                        }
                        i += 1;
                        spaces[i] = length;
                        sum += length;
                    }
                    if sum > 0.0 {
                        let scale_factor = spaces_count as f32 / sum * spacing;
                        for space in spaces.iter_mut().take(spaces_count).skip(1) {
                            *space *= scale_factor;
                        }
                    }
                }
                spacing_mode => {
                    let length_spacing = spacing_mode == crate::SpacingMode::Length;
                    let mut i = 0usize;
                    let n = spaces_count.saturating_sub(1);
                    while i < n {
                        let Some(bone_index) = bones.get(i).copied() else {
                            i += 1;
                            continue;
                        };
                        let setup_length = self
                            .data
                            .bones
                            .get(bone_index)
                            .map(|b| b.length)
                            .unwrap_or(0.0);
                        if setup_length < EPSILON {
                            if scale && let Some(out) = lengths.get_mut(i) {
                                *out = 0.0;
                            }
                            i += 1;
                            spaces[i] = spacing;
                            continue;
                        }
                        let Some(bone) = self.bones.get(bone_index) else {
                            i += 1;
                            continue;
                        };
                        let x = setup_length * bone.a;
                        let y = setup_length * bone.c;
                        let length = (x * x + y * y).sqrt();
                        if scale && let Some(out) = lengths.get_mut(i) {
                            *out = length;
                        }
                        i += 1;
                        spaces[i] = (if length_spacing {
                            setup_length + spacing
                        } else {
                            spacing
                        }) * length
                            / setup_length;
                    }
                }
            }

            let positions = compute_path_world_positions(
                self,
                &mut scratch.positions,
                &mut scratch.world,
                &mut scratch.curves,
                target_slot_index,
                path,
                data.position_mode,
                data.spacing_mode,
                spaces_count,
                tangents,
                spaces,
                position,
            );
            if positions.len() < 2 {
                break 'applied false;
            }

            let mut bone_x = positions[0];
            let mut bone_y = positions[1];
            let mut offset_rotation = data.offset_rotation;
            let tip = if offset_rotation == 0.0 {
                data.rotate_mode == crate::RotateMode::Chain
            } else {
                let deg_rad_reflect = {
                    let Some(target_slot) = self.slots.get(target_slot_index) else {
                        break 'applied false;
                    };
                    let Some(parent) = self.bones.get(target_slot.bone) else {
                        break 'applied false;
                    };
                    if parent.a * parent.d - parent.b * parent.c > 0.0 {
                        std::f32::consts::PI / 180.0
                    } else {
                        -std::f32::consts::PI / 180.0
                    }
                };
                offset_rotation *= deg_rad_reflect;
                false
            };

            let mut applied = false;
            let mut p = 3usize;
            for i in 0..bone_count {
                let Some(&bone_index) = bones.get(i) else {
                    p = p.saturating_add(3);
                    continue;
                };
                if bone_index >= self.bones.len() {
                    p = p.saturating_add(3);
                    continue;
                }

                let (mut a, mut b, mut c0, mut d, world_x, world_y) = {
                    let bone = &self.bones[bone_index];
                    (bone.a, bone.b, bone.c, bone.d, bone.world_x, bone.world_y)
                };
                let new_world_x = world_x + (bone_x - world_x) * mix_x;
                let new_world_y = world_y + (bone_y - world_y) * mix_y;

                let x = *positions.get(p).unwrap_or(&bone_x);
                let y = *positions.get(p + 1).unwrap_or(&bone_y);
                let dx = x - bone_x;
                let dy = y - bone_y;

                if scale {
                    let length = *lengths.get(i).unwrap_or(&0.0);
                    if length >= EPSILON {
                        let s = (((dx * dx + dy * dy).sqrt() / length) - 1.0) * mix_rotate + 1.0;
                        a *= s;
                        c0 *= s;
                    }
                }

                bone_x = x;
                bone_y = y;

                if mix_rotate > 0.0 {
                    let mut r = if tangents {
                        *positions.get(p - 1).unwrap_or(&0.0)
                    } else if *spaces.get(i + 1).unwrap_or(&0.0) < EPSILON {
                        *positions.get(p + 2).unwrap_or(&0.0)
                    } else {
                        dy.atan2(dx)
                    };
                    r -= c0.atan2(a);
                    if tip {
                        let cos = r.cos();
                        let sin = r.sin();
                        let length = self
                            .data
                            .bones
                            .get(bone_index)
                            .map(|b| b.length)
                            .unwrap_or(0.0);
                        bone_x += (length * (cos * a - sin * c0) - dx) * mix_rotate;
                        bone_y += (length * (sin * a + cos * c0) - dy) * mix_rotate;
                    } else {
                        r += offset_rotation;
                    }

                    let r = wrap_pi(r) * mix_rotate;
                    let cos = r.cos();
                    let sin = r.sin();
                    let rotated_a = cos * a - sin * c0;
                    let rotated_b = cos * b - sin * d;
                    let rotated_c = sin * a + cos * c0;
                    let rotated_d = sin * b + cos * d;
                    a = rotated_a;
                    b = rotated_b;
                    c0 = rotated_c;
                    d = rotated_d;
                }

                {
                    let bone = &mut self.bones[bone_index];
                    bone.world_x = new_world_x;
                    bone.world_y = new_world_y;
                    bone.a = a;
                    bone.b = b;
                    bone.c = c0;
                    bone.d = d;
                }

                self.bone_modify_world(bone_index);

                applied = true;
                p += 3;
            }

            applied
        };

        self.path_constraint_scratch[constraint_index] = scratch;
        self.path_constraints[constraint_index].bones = bones;
        applied
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

#[allow(clippy::too_many_arguments)]
fn compute_attachment_world_vertices(
    skeleton: &Skeleton,
    slot_index: usize,
    vertices: &crate::MeshVertices,
    start: usize,
    count: usize,
    world_vertices: &mut Vec<f32>,
    offset: usize,
    stride: usize,
) {
    let Some(slot) = skeleton.slots.get(slot_index) else {
        return;
    };
    let Some(bone) = skeleton.bones.get(slot.bone) else {
        return;
    };

    let start_vertex = start / 2;
    let vertex_count = count / 2;
    let out_end = offset + vertex_count * stride;
    if world_vertices.len() < out_end {
        world_vertices.resize(out_end, 0.0);
    }

    match vertices {
        crate::MeshVertices::Unweighted(v) => {
            if start_vertex >= v.len() {
                return;
            }
            let available = v.len().saturating_sub(start_vertex);
            let n = vertex_count.min(available);
            let deform = slot.deform.as_slice();
            let use_deform = !deform.is_empty() && deform.len() >= v.len() * 2;
            for i in 0..n {
                let vi = start_vertex + i;
                let (vx, vy) = if use_deform {
                    (
                        deform.get(vi * 2).copied().unwrap_or(0.0),
                        deform.get(vi * 2 + 1).copied().unwrap_or(0.0),
                    )
                } else {
                    let p = &v[vi];
                    (p[0], p[1])
                };
                let w = offset + i * stride;
                world_vertices[w] = vx * bone.a + vy * bone.b + bone.world_x;
                world_vertices[w + 1] = vx * bone.c + vy * bone.d + bone.world_y;
            }
        }
        crate::MeshVertices::Weighted(v) => {
            if start_vertex >= v.len() {
                return;
            }
            let available = v.len().saturating_sub(start_vertex);
            let n = vertex_count.min(available);

            let mut skip_weights = 0usize;
            for i in 0..start_vertex {
                skip_weights = skip_weights.saturating_add(v.get(i).map(|w| w.len()).unwrap_or(0));
            }
            let mut f = skip_weights * 2;
            let deform = slot.deform.as_slice();

            for i in 0..n {
                let vi = start_vertex + i;
                let mut wx = 0.0f32;
                let mut wy = 0.0f32;
                for wgt in v.get(vi).into_iter().flatten() {
                    let Some(b) = skeleton.bones.get(wgt.bone) else {
                        f = f.saturating_add(2);
                        continue;
                    };
                    let dx = deform.get(f).copied().unwrap_or(0.0);
                    let dy = deform.get(f + 1).copied().unwrap_or(0.0);
                    f += 2;
                    let vx = wgt.x + dx;
                    let vy = wgt.y + dy;
                    let x = b.a * vx + b.b * vy + b.world_x;
                    let y = b.c * vx + b.d * vy + b.world_y;
                    wx += x * wgt.weight;
                    wy += y * wgt.weight;
                }
                let w = offset + i * stride;
                world_vertices[w] = wx;
                world_vertices[w + 1] = wy;
            }
        }
    }
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
