use crate::SkeletonData;
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

fn estimate_path_attachment_scratch_capacities(
    data: &SkeletonData,
    target_slot_index: usize,
) -> (usize, usize) {
    let mut max_world_floats = 8usize;
    let mut max_curves = 0usize;

    for skin in data.skins.values() {
        let Some(slot_map) = skin.attachments.get(target_slot_index) else {
            continue;
        };
        for attachment in slot_map.values() {
            let crate::AttachmentData::Path(path) = attachment else {
                continue;
            };

            let vertices_count = match &path.vertices {
                crate::MeshVertices::Unweighted(v) => v.len(),
                crate::MeshVertices::Weighted(v) => v.len(),
            };
            let vertices_length = vertices_count * 2;
            if vertices_length < 6 {
                continue;
            }

            if path.constant_speed {
                let world_floats = if path.closed {
                    vertices_length + 2
                } else {
                    vertices_length.saturating_sub(4)
                };
                max_world_floats = max_world_floats.max(world_floats);

                let curves = if path.closed {
                    vertices_length / 6
                } else {
                    (vertices_length / 6).saturating_sub(1)
                };
                max_curves = max_curves.max(curves);
            } else {
                max_world_floats = max_world_floats.max(8);
            }
        }
    }

    (max_world_floats, max_curves)
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

    /// Transforms a point from world coordinates into this bone's local coordinates.
    ///
    /// Mirrors the official runtimes' `Bone.worldToLocal`.
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
    /// Mirrors the official runtimes' `Bone.localToWorld`.
    pub fn local_to_world(&self, local_x: f32, local_y: f32) -> (f32, f32) {
        (
            local_x * self.a + local_y * self.b + self.world_x,
            local_x * self.c + local_y * self.d + self.world_y,
        )
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

#[derive(Clone, Debug, Default)]
struct PathConstraintScratch {
    spaces: Vec<f32>,
    lengths: Vec<f32>,
    positions: Vec<f32>,
    world: Vec<f32>,
    curves: Vec<f32>,
}

#[derive(Copy, Clone, Debug)]
enum UpdateCacheItem {
    Bone(usize),
    Ik(usize),
    Transform(usize),
    Path(usize),
    Physics(usize),
    Slider(usize),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ConstraintKind {
    Ik,
    Transform,
    Path,
    Physics,
    Slider,
}

#[derive(Copy, Clone, Debug)]
struct OrderedConstraint {
    order: i32,
    kind: ConstraintKind,
    index: usize,
}

impl Skeleton {
    fn reset_world_children_if_updated(&mut self, bone_index: usize, epoch: u32) {
        let children = self
            .bone_children
            .get(bone_index)
            .cloned()
            .unwrap_or_default();
        for child in children {
            if child >= self.bones.len() {
                continue;
            }
            if self.bones[child].world_epoch == epoch {
                self.bones[child].world_epoch = 0;
                self.bones[child].local_epoch = 0;
                self.reset_world_children_if_updated(child, epoch);
            }
        }
    }

    fn bone_modify_world(&mut self, bone_index: usize) {
        if bone_index >= self.bones.len() {
            return;
        }
        let epoch = self.update_epoch;
        self.bones[bone_index].world_epoch = epoch;
        self.bones[bone_index].local_epoch = epoch;
        self.reset_world_children_if_updated(bone_index, epoch);
    }

    fn bone_modify_local(&mut self, bone_index: usize) {
        if bone_index >= self.bones.len() {
            return;
        }
        let epoch = self.update_epoch;
        if self.bones[bone_index].local_epoch == epoch {
            self.update_applied_transform(bone_index);
        }
        self.bones[bone_index].local_epoch = 0;
        self.bones[bone_index].world_epoch = 0;
        self.reset_world_children_if_updated(bone_index, epoch);
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
        fn bone_name(skeleton: &Skeleton, index: usize) -> &str {
            skeleton
                .data
                .bones
                .get(index)
                .map(|b| b.name.as_str())
                .unwrap_or("<unknown>")
        }

        self.update_cache
            .iter()
            .map(|item| match *item {
                UpdateCacheItem::Bone(index) => format!("bone {}", bone_name(self, index)),
                UpdateCacheItem::Ik(index) => {
                    let name = self
                        .ik_constraints
                        .get(index)
                        .and_then(|c| self.data.ik_constraints.get(c.data_index))
                        .map(|d| d.name.as_str())
                        .unwrap_or("<unknown>");
                    format!("ik {}", name)
                }
                UpdateCacheItem::Transform(index) => {
                    let name = self
                        .transform_constraints
                        .get(index)
                        .and_then(|c| self.data.transform_constraints.get(c.data_index))
                        .map(|d| d.name.as_str())
                        .unwrap_or("<unknown>");
                    format!("transform {}", name)
                }
                UpdateCacheItem::Path(index) => {
                    let name = self
                        .path_constraints
                        .get(index)
                        .and_then(|c| self.data.path_constraints.get(c.data_index))
                        .map(|d| d.name.as_str())
                        .unwrap_or("<unknown>");
                    format!("path {}", name)
                }
                UpdateCacheItem::Physics(index) => {
                    let name = self
                        .physics_constraints
                        .get(index)
                        .and_then(|c| self.data.physics_constraints.get(c.data_index))
                        .map(|d| d.name.as_str())
                        .unwrap_or("<unknown>");
                    format!("physics {}", name)
                }
                UpdateCacheItem::Slider(index) => {
                    let name = self
                        .slider_constraints
                        .get(index)
                        .and_then(|c| self.data.slider_constraints.get(c.data_index))
                        .map(|d| d.name.as_str())
                        .unwrap_or("<unknown>");
                    format!("slider {}", name)
                }
            })
            .collect()
    }

    fn rebuild_update_cache(&mut self) {
        fn sort_reset(skeleton: &Skeleton, bone_index: usize, sorted: &mut [bool]) {
            if bone_index >= sorted.len() {
                return;
            }
            if !skeleton
                .bones
                .get(bone_index)
                .map(|b| b.active)
                .unwrap_or(false)
            {
                return;
            }
            if !sorted[bone_index] {
                return;
            }

            if let Some(children) = skeleton.bone_children.get(bone_index) {
                for &child in children {
                    sort_reset(skeleton, child, sorted);
                }
            }
            sorted[bone_index] = false;
        }

        fn sort_reset_children(skeleton: &Skeleton, bone_index: usize, sorted: &mut [bool]) {
            let Some(children) = skeleton.bone_children.get(bone_index) else {
                return;
            };
            for &child in children {
                sort_reset(skeleton, child, sorted);
            }
        }

        fn sort_bone(
            skeleton: &Skeleton,
            bone_index: usize,
            sorted: &mut [bool],
            out: &mut Vec<UpdateCacheItem>,
        ) {
            if bone_index >= sorted.len() {
                return;
            }
            if sorted[bone_index] {
                return;
            }
            let Some(bone) = skeleton.bones.get(bone_index) else {
                return;
            };
            if !bone.active {
                sorted[bone_index] = true;
                return;
            }
            if let Some(parent) = bone.parent {
                sort_bone(skeleton, parent, sorted, out);
            }
            sorted[bone_index] = true;
            out.push(UpdateCacheItem::Bone(bone_index));
        }

        fn sort_path_attachment(
            skeleton: &Skeleton,
            attachment: &crate::AttachmentData,
            slot_bone_index: usize,
            sorted: &mut [bool],
            out: &mut Vec<UpdateCacheItem>,
        ) {
            let crate::AttachmentData::Path(path) = attachment else {
                return;
            };
            match &path.vertices {
                crate::MeshVertices::Unweighted(_) => {
                    sort_bone(skeleton, slot_bone_index, sorted, out);
                }
                crate::MeshVertices::Weighted(vertices) => {
                    for weights in vertices {
                        for w in weights {
                            sort_bone(skeleton, w.bone, sorted, out);
                        }
                    }
                }
            }
        }

        fn sort_path_slot(
            skeleton: &Skeleton,
            skin: &crate::SkinData,
            slot_index: usize,
            slot_bone_index: usize,
            sorted: &mut [bool],
            out: &mut Vec<UpdateCacheItem>,
        ) {
            let Some(slot_map) = skin.attachments.get(slot_index) else {
                return;
            };
            for attachment in slot_map.values() {
                sort_path_attachment(skeleton, attachment, slot_bone_index, sorted, out);
            }
        }

        let out = {
            let skeleton: &Skeleton = &*self;
            let bone_count = skeleton.bones.len();
            let mut out = Vec::<UpdateCacheItem>::new();
            let mut sorted = vec![false; bone_count];

            for (i, sorted) in sorted.iter_mut().enumerate().take(bone_count) {
                *sorted = !skeleton.bones.get(i).map(|b| b.active).unwrap_or(false);
            }

            let current_skin_name = skeleton.skin.as_deref();
            let current_skin = current_skin_name.and_then(|n| skeleton.data.skin(n));
            let default_skin = if current_skin_name != Some("default") {
                skeleton.data.skin("default")
            } else {
                None
            };

            let mut ordered = Vec::<OrderedConstraint>::with_capacity(
                skeleton.ik_constraints.len()
                    + skeleton.transform_constraints.len()
                    + skeleton.path_constraints.len()
                    + skeleton.physics_constraints.len()
                    + skeleton.slider_constraints.len(),
            );
            for (index, ik) in skeleton.ik_constraints.iter().enumerate() {
                if !ik.active {
                    continue;
                }
                let order = skeleton
                    .data
                    .ik_constraints
                    .get(ik.data_index)
                    .map(|d| d.order)
                    .unwrap_or(0);
                ordered.push(OrderedConstraint {
                    order,
                    kind: ConstraintKind::Ik,
                    index,
                });
            }
            for (index, c) in skeleton.transform_constraints.iter().enumerate() {
                if !c.active {
                    continue;
                }
                let order = skeleton
                    .data
                    .transform_constraints
                    .get(c.data_index)
                    .map(|d| d.order)
                    .unwrap_or(0);
                ordered.push(OrderedConstraint {
                    order,
                    kind: ConstraintKind::Transform,
                    index,
                });
            }
            for (index, c) in skeleton.path_constraints.iter().enumerate() {
                if !c.active {
                    continue;
                }
                let order = skeleton
                    .data
                    .path_constraints
                    .get(c.data_index)
                    .map(|d| d.order)
                    .unwrap_or(0);
                ordered.push(OrderedConstraint {
                    order,
                    kind: ConstraintKind::Path,
                    index,
                });
            }
            for (index, c) in skeleton.physics_constraints.iter().enumerate() {
                if !c.active {
                    continue;
                }
                let order = skeleton
                    .data
                    .physics_constraints
                    .get(c.data_index)
                    .map(|d| d.order)
                    .unwrap_or(0);
                ordered.push(OrderedConstraint {
                    order,
                    kind: ConstraintKind::Physics,
                    index,
                });
            }
            for (index, c) in skeleton.slider_constraints.iter().enumerate() {
                if !c.active {
                    continue;
                }
                let order = skeleton
                    .data
                    .slider_constraints
                    .get(c.data_index)
                    .map(|d| d.order)
                    .unwrap_or(0);
                ordered.push(OrderedConstraint {
                    order,
                    kind: ConstraintKind::Slider,
                    index,
                });
            }
            ordered.sort_by_key(|c| c.order);

            for item in ordered {
                match item.kind {
                    ConstraintKind::Ik => {
                        let Some(ik) = skeleton.ik_constraints.get(item.index) else {
                            continue;
                        };
                        sort_bone(skeleton, ik.target, &mut sorted, &mut out);
                        let Some(&parent_bone_index) = ik.bones.first() else {
                            continue;
                        };
                        sort_bone(skeleton, parent_bone_index, &mut sorted, &mut out);
                        out.push(UpdateCacheItem::Ik(item.index));
                        if parent_bone_index < sorted.len() {
                            sorted[parent_bone_index] = false;
                        }
                        sort_reset_children(skeleton, parent_bone_index, &mut sorted);
                    }
                    ConstraintKind::Transform => {
                        let Some(c) = skeleton.transform_constraints.get(item.index) else {
                            continue;
                        };
                        let Some(data) = skeleton.data.transform_constraints.get(c.data_index)
                        else {
                            continue;
                        };
                        if !data.local_source {
                            sort_bone(skeleton, c.source, &mut sorted, &mut out);
                        }
                        let world_target = !data.local_target;
                        if world_target {
                            for &bone_index in &c.bones {
                                sort_bone(skeleton, bone_index, &mut sorted, &mut out);
                            }
                        }
                        out.push(UpdateCacheItem::Transform(item.index));
                        for &bone_index in &c.bones {
                            sort_reset_children(skeleton, bone_index, &mut sorted);
                        }
                        for &bone_index in &c.bones {
                            if bone_index < sorted.len() {
                                sorted[bone_index] = world_target;
                            }
                        }
                    }
                    ConstraintKind::Path => {
                        let Some(c) = skeleton.path_constraints.get(item.index) else {
                            continue;
                        };
                        let Some(slot) = skeleton.slots.get(c.target) else {
                            continue;
                        };
                        let slot_bone_index = slot.bone;

                        if let Some(skin) = current_skin {
                            sort_path_slot(
                                skeleton,
                                skin,
                                c.target,
                                slot_bone_index,
                                &mut sorted,
                                &mut out,
                            );
                        }
                        if let Some(default_skin) = default_skin {
                            sort_path_slot(
                                skeleton,
                                default_skin,
                                c.target,
                                slot_bone_index,
                                &mut sorted,
                                &mut out,
                            );
                        }
                        if let Some(att) = skeleton.slot_attachment_data(c.target) {
                            sort_path_attachment(
                                skeleton,
                                att,
                                slot_bone_index,
                                &mut sorted,
                                &mut out,
                            );
                        }

                        for &bone_index in &c.bones {
                            sort_bone(skeleton, bone_index, &mut sorted, &mut out);
                        }
                        out.push(UpdateCacheItem::Path(item.index));
                        for &bone_index in &c.bones {
                            sort_reset_children(skeleton, bone_index, &mut sorted);
                        }
                        for &bone_index in &c.bones {
                            if bone_index < sorted.len() {
                                sorted[bone_index] = true;
                            }
                        }
                    }
                    ConstraintKind::Physics => {
                        let Some(c) = skeleton.physics_constraints.get(item.index) else {
                            continue;
                        };
                        sort_bone(skeleton, c.bone, &mut sorted, &mut out);
                        out.push(UpdateCacheItem::Physics(item.index));
                        sort_reset_children(skeleton, c.bone, &mut sorted);
                    }
                    ConstraintKind::Slider => {
                        let Some(c) = skeleton.slider_constraints.get(item.index) else {
                            continue;
                        };
                        let Some(data) = skeleton.data.slider_constraints.get(c.data_index) else {
                            continue;
                        };
                        if let (Some(bone), false) = (data.bone, data.local) {
                            sort_bone(skeleton, bone, &mut sorted, &mut out);
                        }
                        out.push(UpdateCacheItem::Slider(item.index));
                        for &bone_index in &c.animation_bones {
                            if bone_index < sorted.len() {
                                sorted[bone_index] = false;
                            }
                            sort_reset_children(skeleton, bone_index, &mut sorted);
                        }
                    }
                }
            }

            for bone_index in 0..bone_count {
                sort_bone(skeleton, bone_index, &mut sorted, &mut out);
            }

            out
        };

        self.update_cache = out;
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
        if bone_index >= self.bones.len() {
            return;
        }
        if !self.bones[bone_index].active {
            return;
        }
        if self.bones[bone_index].world_epoch == self.update_epoch {
            return;
        }
        if self.bones[bone_index].local_epoch == self.update_epoch {
            self.update_applied_transform(bone_index);
            self.bones[bone_index].local_epoch = 0;
        }

        let parent_index = self.bones[bone_index].parent;
        if let Some(parent_index) = parent_index {
            if parent_index >= self.bones.len() {
                return;
            }
            if !self.bones[parent_index].active {
                return;
            }

            let parent = {
                let p = &self.bones[parent_index];
                ParentTransform {
                    a: p.a,
                    b: p.b,
                    c: p.c,
                    d: p.d,
                    world_x: p.world_x,
                    world_y: p.world_y,
                }
            };
            update_world_transform_child(
                &mut self.bones[bone_index],
                self.scale_x,
                self.scale_y,
                self.x,
                self.y,
                &parent,
            );
        } else {
            update_world_transform_root(
                &mut self.bones[bone_index],
                self.x,
                self.y,
                self.scale_x,
                self.scale_y,
            );
        }

        self.bones[bone_index].world_epoch = self.update_epoch;
    }

    fn apply_ik_constraint(&mut self, constraint_index: usize) -> bool {
        let Some(ik) = self.ik_constraints.get(constraint_index).cloned() else {
            return false;
        };
        // spine-cpp does not clamp the IK mix; Add blending can intentionally push it beyond 1.
        // Keep behavior identical for strict runtime parity.
        let mix = ik.mix;
        if mix == 0.0 {
            return false;
        }

        let Some(target) = self.bones.get(ik.target) else {
            return false;
        };
        let target_x = target.world_x;
        let target_y = target.world_y;
        match ik.bones.as_slice() {
            [bone] => {
                self.bone_modify_local(*bone);
                self.apply_ik_one(
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
                self.bone_modify_local(*parent);
                self.bone_modify_local(*child);
                self.apply_ik_two(
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
        if constraint_index >= self.slider_constraints.len() {
            return false;
        }

        let (data_index, mix, pose_time) = {
            let c = &self.slider_constraints[constraint_index];
            (c.data_index, c.mix, c.time)
        };
        if mix == 0.0 {
            return false;
        }

        let (looped, additive, local, bone, property, property_from, to, scale, animation_index) = {
            let Some(data) = self.data.slider_constraints.get(data_index) else {
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

        // Avoid borrowing `self.data` across `&mut self` calls during constraint evaluation.
        let data = std::sync::Arc::clone(&self.data);
        let Some(animation) = data.animations.get(animation_index) else {
            return false;
        };
        let animation_duration = animation.duration;

        let mut time_to_apply = pose_time;
        if let (Some(bone_index), Some(property)) = (bone, property) {
            let Some(bone) = self.bones.get(bone_index) else {
                return false;
            };
            if !bone.active {
                return false;
            }

            if local {
                // Match upstream: `validateLocalTransform` on the applied pose before reading local
                // properties (local values may be stale after world-space constraints).
                if bone.local_epoch == self.update_epoch {
                    self.update_applied_transform(bone_index);
                    self.bones[bone_index].local_epoch = 0;
                }
            }

            let property_value = match property {
                crate::TransformProperty::Rotate => {
                    if local {
                        self.bones
                            .get(bone_index)
                            .map(|b| b.arotation)
                            .unwrap_or(0.0)
                    } else {
                        let (a, b, c, d) = {
                            let bone = &self.bones[bone_index];
                            (bone.a, bone.b, bone.c, bone.d)
                        };
                        let sx = self.scale_x;
                        let sy = self.scale_y;
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
                        self.bones.get(bone_index).map(|b| b.ax).unwrap_or(0.0)
                    } else {
                        self.bones
                            .get(bone_index)
                            .map(|b| b.world_x / self.scale_x)
                            .unwrap_or(0.0)
                    }
                }
                crate::TransformProperty::Y => {
                    if local {
                        self.bones.get(bone_index).map(|b| b.ay).unwrap_or(0.0)
                    } else {
                        self.bones
                            .get(bone_index)
                            .map(|b| b.world_y / self.scale_y)
                            .unwrap_or(0.0)
                    }
                }
                crate::TransformProperty::ScaleX => {
                    if local {
                        self.bones
                            .get(bone_index)
                            .map(|b| b.ascale_x)
                            .unwrap_or(0.0)
                    } else {
                        let (a, c) = {
                            let bone = &self.bones[bone_index];
                            (bone.a / self.scale_x, bone.c / self.scale_y)
                        };
                        (a * a + c * c).sqrt()
                    }
                }
                crate::TransformProperty::ScaleY => {
                    if local {
                        self.bones
                            .get(bone_index)
                            .map(|b| b.ascale_y)
                            .unwrap_or(0.0)
                    } else {
                        let (b, d) = {
                            let bone = &self.bones[bone_index];
                            (bone.b / self.scale_x, bone.d / self.scale_y)
                        };
                        (b * b + d * d).sqrt()
                    }
                }
                crate::TransformProperty::ShearY => {
                    if local {
                        self.bones
                            .get(bone_index)
                            .map(|b| b.ashear_y)
                            .unwrap_or(0.0)
                    } else {
                        let (a, b, c, d) = {
                            let bone = &self.bones[bone_index];
                            (bone.a, bone.b, bone.c, bone.d)
                        };
                        let sx = self.scale_x;
                        let sy = self.scale_y;
                        ((d / sy).atan2(b / sx) - (c / sy).atan2(a / sx)).to_degrees() - 90.0
                    }
                }
            };

            time_to_apply = to + (property_value - property_from) * scale;
            if looped {
                if animation_duration > 0.0 {
                    time_to_apply =
                        animation_duration + time_to_apply.rem_euclid(animation_duration);
                }
            } else if time_to_apply < 0.0 {
                time_to_apply = 0.0;
            }
        }

        let animation_bones =
            std::mem::take(&mut self.slider_constraints[constraint_index].animation_bones);
        for &bone_index in &animation_bones {
            self.bone_modify_local(bone_index);
        }

        crate::runtime::apply_animation_applied(
            animation,
            self,
            time_to_apply,
            looped,
            mix,
            if additive {
                crate::MixBlend::Add
            } else {
                crate::MixBlend::Replace
            },
        );

        self.slider_constraints[constraint_index].animation_bones = animation_bones;
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_ik_one(
        &mut self,
        bone_index: usize,
        target_x: f32,
        target_y: f32,
        compress: bool,
        stretch: bool,
        scale_y_mode: crate::ScaleYMode,
        alpha: f32,
    ) {
        fn signum(v: f32) -> f32 {
            if v > 0.0 {
                1.0
            } else if v < 0.0 {
                -1.0
            } else {
                0.0
            }
        }

        if !(alpha.is_finite()) || alpha <= 0.0 {
            return;
        }
        if bone_index >= self.bones.len() {
            return;
        }
        let Some(parent_index) = self.bones[bone_index].parent else {
            return;
        };
        if parent_index >= self.bones.len() {
            return;
        }

        let (pa, mut pb, pc, mut pd, pwx, pwy) = {
            let p = &self.bones[parent_index];
            (p.a, p.b, p.c, p.d, p.world_x, p.world_y)
        };

        let (inherit, world_x, world_y, ax, ay, arotation, mut sx, mut sy, ashear_x, ashear_y) = {
            let b = &self.bones[bone_index];
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
                (target_x - world_x) * signum(self.scale_x),
                (target_y - world_y) * signum(self.scale_y),
            ),
            crate::Inherit::NoRotationOrReflection => {
                let denom = (pa * pa + pc * pc).max(1.0e-5);
                let s = (pa * pd - pb * pc).abs() / denom;
                let sa = pa / self.scale_x;
                let sc = pc / self.scale_y;
                pb = -sc * s * self.scale_x;
                pd = sa * s * self.scale_y;
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
            let length = self
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

        let bone = &mut self.bones[bone_index];
        bone.ax = ax;
        bone.ay = ay;
        bone.arotation = arotation + rotation_ik * alpha;
        bone.ascale_x = sx;
        bone.ascale_y = sy;
        bone.ashear_x = ashear_x;
        bone.ashear_y = ashear_y;
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_ik_two(
        &mut self,
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

        if !(alpha.is_finite()) || alpha <= 0.0 {
            return;
        }
        if parent_index >= self.bones.len() || child_index >= self.bones.len() {
            return;
        }
        if self.bones[parent_index].inherit != crate::Inherit::Normal
            || self.bones[child_index].inherit != crate::Inherit::Normal
        {
            return;
        }

        let Some(pp_index) = self.bones[parent_index].parent else {
            return;
        };
        if pp_index >= self.bones.len() {
            return;
        }

        let (px, py, parent_rotation, psx0, psy0) = {
            let p = &self.bones[parent_index];
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
            let c = &self.bones[child_index];
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
            let p = &self.bones[parent_index];
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
            let pp = &self.bones[pp_index];
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
            self.apply_ik_one(
                parent_index,
                target_x,
                target_y,
                false,
                stretch,
                crate::ScaleYMode::None,
                alpha,
            );
            let child = &mut self.bones[child_index];
            child.ax = cx;
            child.ay = cy;
            child.arotation = 0.0;
            child.ascale_x = csx0;
            child.ascale_y = csy0;
            child.ashear_x = child_shear_x;
            child.ashear_y = child_shear_y;
            return;
        }

        let l2 = self
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
                cos = 1.0;
                a2 = 0.0;
                if stretch {
                    let s = (sqrt_f32(dd) / (l1 + l2u) - 1.0) * alpha + 1.0;
                    {
                        let parent = &mut self.bones[parent_index];
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

        let parent = &mut self.bones[parent_index];
        parent.ax = px;
        parent.ay = py;
        parent.arotation = parent_rotation + a1 * alpha;

        let child = &mut self.bones[child_index];
        child.ax = cx;
        child.ay = cy;
        child.arotation = child_rotation + a2 * alpha;
        child.ascale_x = csx0;
        child.ascale_y = csy0;
        child.ashear_x = child_shear_x;
        child.ashear_y = child_shear_y;
    }

    fn apply_transform_constraint(&mut self, constraint_index: usize) -> bool {
        fn clamp_value(v: f32, a: f32, b: f32) -> f32 {
            let (min, max) = if a <= b { (a, b) } else { (b, a) };
            v.clamp(min, max)
        }

        const PI: f32 = std::f32::consts::PI;
        const PI2: f32 = 2.0 * std::f32::consts::PI;
        const DEG_RAD: f32 = std::f32::consts::PI / 180.0;

        let Some(constraint) = self.transform_constraints.get(constraint_index).cloned() else {
            return false;
        };
        let data_index = constraint.data_index;
        let (local_source, local_target, additive, clamp, offsets) = {
            let Some(data) = self.data.transform_constraints.get(data_index) else {
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

        if constraint.source >= self.bones.len() {
            return false;
        }

        if local_source && self.bones[constraint.source].local_epoch == self.update_epoch {
            self.update_applied_transform(constraint.source);
            self.bones[constraint.source].local_epoch = 0;
        }

        let (source_ax, source_ay, source_rot, source_scale_x, source_scale_y, source_shear_y) = {
            let b = &self.bones[constraint.source];
            (b.ax, b.ay, b.arotation, b.ascale_x, b.ascale_y, b.ashear_y)
        };
        let (source_a, source_b, source_c, source_d, source_wx, source_wy) = {
            let b = &self.bones[constraint.source];
            (b.a, b.b, b.c, b.d, b.world_x, b.world_y)
        };

        let sx = self.scale_x;
        let sy = self.scale_y;

        if local_target {
            for &bone_index in &constraint.bones {
                if bone_index >= self.bones.len() {
                    continue;
                }
                if !self.bones[bone_index].active {
                    continue;
                }
                self.bone_modify_local(bone_index);
            }
        }

        let properties = self
            .data
            .transform_constraints
            .get(data_index)
            .map(|d| d.properties.clone())
            .unwrap_or_default();

        let mut applied = false;
        for &bone_index in &constraint.bones {
            if bone_index >= self.bones.len() {
                continue;
            }
            if !self.bones[bone_index].active {
                continue;
            }
            if !local_target {
                self.bone_modify_world(bone_index);
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
                            (a * a + c0 * c0).sqrt()
                                + offsets[crate::TransformProperty::ScaleX.index()]
                        }
                    }
                    crate::TransformProperty::ScaleY => {
                        if local_source {
                            source_scale_y + offsets[crate::TransformProperty::ScaleY.index()]
                        } else {
                            let b = source_b / sx;
                            let d = source_d / sy;
                            (b * b + d * d).sqrt()
                                + offsets[crate::TransformProperty::ScaleY.index()]
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
                        let bone = &mut self.bones[bone_index];
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
                        let bone = &mut self.bones[bone_index];
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

    fn apply_physics_constraint(&mut self, constraint_index: usize, physics: Physics) -> bool {
        const PI_2: f32 = std::f32::consts::PI * 2.0;
        const INV_PI_2: f32 = 1.0 / PI_2;

        let Some(constraint) = self.physics_constraints.get_mut(constraint_index) else {
            return false;
        };
        if !constraint.active {
            return false;
        }
        let mix = constraint.mix;
        if mix == 0.0 {
            return false;
        }

        let Some(data) = self.data.physics_constraints.get(constraint.data_index) else {
            return false;
        };
        let bone_index = constraint.bone;
        if bone_index >= self.bones.len() {
            return false;
        }

        let x = data.x > 0.0;
        let y = data.y > 0.0;
        let rotate_or_shear_x = data.rotate > 0.0 || data.shear_x > 0.0;
        let scale_x = data.scale_x > 0.0;

        let l = self
            .data
            .bones
            .get(bone_index)
            .map(|b| b.length)
            .unwrap_or(0.0);

        let mut z = 0.0f32;

        let mut physics_mode = physics;
        if matches!(physics_mode, Physics::Reset) {
            constraint.reset_with_time(self.time);
            physics_mode = Physics::Update;
        }

        match physics_mode {
            Physics::None => return false,
            Physics::Update => {
                let delta = (self.time - constraint.last_time).max(0.0);
                let aa = constraint.remaining;
                constraint.remaining += delta;
                constraint.last_time = self.time;

                let (mut bx, mut by) = {
                    let bone = &self.bones[bone_index];
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
                    let reference_scale = self.data.reference_scale;

                    let mut qx = data.limit * delta;
                    let qy = qx * self.scale_y.abs();
                    qx *= self.scale_x.abs();

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
                            let ax = (w * self.wind_x + g * self.gravity_x) * self.scale_x;
                            let ay = (w * self.wind_y + g * self.gravity_y) * self.scale_y;

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
                            let bone = &self.bones[bone_index];
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
                                    constraint.scale_offset +=
                                        (dx * ccos + dy * ssin) * inertia / r;
                                }
                            }
                        } else {
                            ccos = ca.cos();
                            ssin = ca.sin();
                            let world_scale_x = (bone_a * bone_a + bone_c * bone_c).sqrt();
                            let r = l * world_scale_x
                                - constraint.scale_lag * (1.0 - aa / step).max(0.0);
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

                            let ax =
                                constraint.wind * self.wind_x + constraint.gravity * self.gravity_x;
                            let ay =
                                constraint.wind * self.wind_y + constraint.gravity * self.gravity_y;
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
                        let bone = &mut self.bones[bone_index];
                        bone.world_x = bx;
                        bone.world_y = by;
                    }
                }

                constraint.cx = self.bones[bone_index].world_x;
                constraint.cy = self.bones[bone_index].world_y;
            }
            Physics::Pose => {
                z = (1.0 - constraint.remaining / data.step).max(0.0);
                if x {
                    self.bones[bone_index].world_x +=
                        (constraint.x_offset - constraint.x_lag * z) * mix * data.x;
                }
                if y {
                    self.bones[bone_index].world_y +=
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
                    let b = self.bones[bone_index].b;
                    let d = self.bones[bone_index].d;
                    self.bones[bone_index].b = c * b - s * d;
                    self.bones[bone_index].d = s * b + c * d;
                }
                r += o * data.shear_x;
                let s = r.sin();
                let c = r.cos();
                let a = self.bones[bone_index].a;
                let c0 = self.bones[bone_index].c;
                self.bones[bone_index].a = c * a - s * c0;
                self.bones[bone_index].c = s * a + c * c0;
            } else {
                o *= data.rotate;
                let s = o.sin();
                let c = o.cos();
                let a = self.bones[bone_index].a;
                let c0 = self.bones[bone_index].c;
                self.bones[bone_index].a = c * a - s * c0;
                self.bones[bone_index].c = s * a + c * c0;
                let b = self.bones[bone_index].b;
                let d = self.bones[bone_index].d;
                self.bones[bone_index].b = c * b - s * d;
                self.bones[bone_index].d = s * b + c * d;
            }
        }

        if scale_x {
            let s = 1.0 + (constraint.scale_offset - constraint.scale_lag * z) * mix * data.scale_x;
            self.bones[bone_index].a *= s;
            self.bones[bone_index].c *= s;
            match constraint.scale_y_mode {
                crate::ScaleYMode::Uniform => {
                    self.bones[bone_index].b *= s;
                    self.bones[bone_index].d *= s;
                }
                crate::ScaleYMode::Volume => {
                    let sy = s.abs();
                    let sy = if sy >= 0.7 {
                        1.0 / sy
                    } else {
                        4.0 - 3.67347 * sy
                    };
                    self.bones[bone_index].b *= sy;
                    self.bones[bone_index].d *= sy;
                }
                crate::ScaleYMode::None => {}
            }
        }

        if !matches!(physics_mode, Physics::Pose) {
            constraint.tx = l * self.bones[bone_index].a;
            constraint.ty = l * self.bones[bone_index].c;
        }

        self.bone_modify_world(bone_index);
        true
    }

    fn update_applied_transform(&mut self, bone_index: usize) {
        if bone_index >= self.bones.len() {
            return;
        }

        let (a, b, c0, d, wx, wy) = {
            let bone = &self.bones[bone_index];
            (bone.a, bone.b, bone.c, bone.d, bone.world_x, bone.world_y)
        };

        let parent = self.bones[bone_index].parent;

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
            let mut shear_x = 0.0;
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
                shear_x = 0.0;
                (rotation, scale_x, scale_y, shear_x, shear_y)
            }
        }

        if parent.is_none() {
            let sxi = 1.0 / self.scale_x;
            let syi = 1.0 / self.scale_y;
            let ra = a * sxi;
            let rb = b * sxi;
            let rc = c0 * syi;
            let rd = d * syi;
            let (arotation, ascale_x, ascale_y, ashear_x, ashear_y) =
                decompose_local_with_rotation(ra, rb, rc, rd, 0.0);
            let bone = &mut self.bones[bone_index];
            bone.ax = (wx - self.x) * sxi;
            bone.ay = (wy - self.y) * syi;
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
            let p = &self.bones[parent_index];
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
            match self.bones[bone_index].inherit {
                crate::Inherit::Normal => {
                    let ra = ia * a - ib * c0;
                    let rb = ia * b - ib * d;
                    let rc = id * c0 - ic * a;
                    let rd = id * d - ic * b;
                    decompose_local_with_rotation(ra, rb, rc, rd, 0.0)
                }
                crate::Inherit::OnlyTranslation => {
                    let sxi = 1.0 / self.scale_x;
                    let syi = 1.0 / self.scale_y;
                    decompose_local_with_rotation(a * sxi, b * sxi, c0 * syi, d * syi, 0.0)
                }
                crate::Inherit::NoRotationOrReflection => {
                    let sxi = 1.0 / self.scale_x;
                    let syi = 1.0 / self.scale_y;
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
                    let sxi = 1.0 / self.scale_x;
                    let syi = 1.0 / self.scale_y;
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
                    let r = rotation.to_radians();
                    let cos_r = cos_f32(r);
                    let sin_r = sin_f32(r);
                    let mut za = (pa * cos_r + pb * sin_r) * sxi;
                    let mut zc = (pc * cos_r + pd * sin_r) * syi;
                    let s = 1.0 / sqrt_f32(za * za + zc * zc);
                    za *= s;
                    zc *= s;
                    let si = if self.bones[bone_index].inherit == crate::Inherit::NoScale
                        && (pad < 0.0) != ((self.scale_x < 0.0) != (self.scale_y < 0.0))
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

        let bone = &mut self.bones[bone_index];
        bone.ax = ax;
        bone.ay = ay;
        bone.arotation = arotation;
        bone.ascale_x = ascale_x;
        bone.ascale_y = ascale_y;
        bone.ashear_x = ashear_x;
        bone.ashear_y = ashear_y;
        bone.local_epoch = 0;
    }
}

#[derive(Copy, Clone, Debug)]
struct ParentTransform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    world_x: f32,
    world_y: f32,
}

fn update_world_transform_root(bone: &mut Bone, x: f32, y: f32, scale_x: f32, scale_y: f32) {
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

fn update_world_transform_child(
    bone: &mut Bone,
    skeleton_scale_x: f32,
    skeleton_scale_y: f32,
    _skeleton_x: f32,
    _skeleton_y: f32,
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

fn shortest_rotation(mut degrees: f32) -> f32 {
    if degrees > 180.0 {
        degrees -= 360.0;
    } else if degrees <= -180.0 {
        degrees += 360.0;
    }
    degrees
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

fn path_attachment_for_slot(
    skeleton: &Skeleton,
    slot_index: usize,
) -> Option<(usize, &crate::PathAttachmentData)> {
    let attachment_name = skeleton
        .slots
        .get(slot_index)
        .and_then(|s| s.attachment.as_deref())?;
    let attachment = skeleton.attachment(slot_index, attachment_name)?;
    match attachment {
        crate::AttachmentData::Path(p) => Some((slot_index, p)),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_path_world_positions<'a>(
    skeleton: &Skeleton,
    positions: &'a mut Vec<f32>,
    world: &mut Vec<f32>,
    curves: &mut Vec<f32>,
    target_slot_index: usize,
    path: &crate::PathAttachmentData,
    position_mode: crate::PositionMode,
    spacing_mode: crate::SpacingMode,
    spaces_count: usize,
    tangents: bool,
    spaces: &[f32],
    mut position: f32,
) -> &'a [f32] {
    const EPSILON: f32 = 1.0e-5;
    const NONE: i32 = -1;
    const BEFORE: i32 = -2;
    const AFTER: i32 = -3;

    let closed = path.closed;
    let mut vertices_length = match &path.vertices {
        crate::MeshVertices::Unweighted(v) => v.len() * 2,
        crate::MeshVertices::Weighted(v) => v.len() * 2,
    };
    if vertices_length < 6 || spaces_count == 0 {
        positions.clear();
        return positions.as_slice();
    }

    let output_len = spaces_count * 3 + 2;
    positions.resize(output_len, 0.0);
    positions.fill(0.0);
    let output = positions.as_mut_slice();

    if !path.constant_speed {
        let lengths = path.lengths.as_slice();
        if lengths.is_empty() {
            return positions.as_slice();
        }

        let mut curve_count = (vertices_length / 6) as i32;
        curve_count -= if closed { 1 } else { 2 };
        if curve_count < 0 {
            return positions.as_slice();
        }
        let curve_count_usize = curve_count as usize;
        if curve_count_usize >= lengths.len() {
            return positions.as_slice();
        }

        let path_length = lengths[curve_count_usize];
        if position_mode == crate::PositionMode::Percent {
            position *= path_length;
        }
        let multiplier = match spacing_mode {
            crate::SpacingMode::Percent => path_length,
            crate::SpacingMode::Proportional => path_length / spaces_count as f32,
            _ => 1.0,
        };

        world.resize(8, 0.0);
        world.fill(0.0);
        let mut prev_curve = NONE;
        let mut curve = 0usize;
        for i in 0..spaces_count {
            let space = spaces.get(i).copied().unwrap_or(0.0) * multiplier;
            position += space;
            let mut p = position;

            if closed {
                p = p.rem_euclid(path_length);
                curve = 0;
            } else if p < 0.0 {
                if prev_curve != BEFORE {
                    prev_curve = BEFORE;
                    compute_attachment_world_vertices(
                        skeleton,
                        target_slot_index,
                        &path.vertices,
                        2,
                        4,
                        world,
                        0,
                        2,
                    );
                }
                add_before_position(p, world.as_slice(), 0, output, i * 3);
                continue;
            } else if p > path_length {
                if prev_curve != AFTER {
                    prev_curve = AFTER;
                    compute_attachment_world_vertices(
                        skeleton,
                        target_slot_index,
                        &path.vertices,
                        vertices_length.saturating_sub(6),
                        4,
                        world,
                        0,
                        2,
                    );
                }
                add_after_position(p - path_length, world.as_slice(), 0, output, i * 3);
                continue;
            }

            loop {
                if curve >= lengths.len() {
                    break;
                }
                let length = lengths[curve];
                if p > length {
                    curve += 1;
                    continue;
                }
                if curve == 0 {
                    p /= length.max(EPSILON);
                } else {
                    let prev = lengths[curve - 1];
                    p = (p - prev) / (length - prev).max(EPSILON);
                }
                break;
            }

            if curve as i32 != prev_curve {
                prev_curve = curve as i32;
                if closed && curve == curve_count_usize {
                    compute_attachment_world_vertices(
                        skeleton,
                        target_slot_index,
                        &path.vertices,
                        vertices_length.saturating_sub(4),
                        4,
                        world,
                        0,
                        2,
                    );
                    compute_attachment_world_vertices(
                        skeleton,
                        target_slot_index,
                        &path.vertices,
                        0,
                        4,
                        world,
                        4,
                        2,
                    );
                } else {
                    compute_attachment_world_vertices(
                        skeleton,
                        target_slot_index,
                        &path.vertices,
                        curve * 6 + 2,
                        8,
                        world,
                        0,
                        2,
                    );
                }
            }

            let world_slice = world.as_slice();
            add_curve_position(
                p,
                world_slice[0],
                world_slice[1],
                world_slice[2],
                world_slice[3],
                world_slice[4],
                world_slice[5],
                world_slice[6],
                world_slice[7],
                output,
                i * 3,
                tangents || (i > 0 && space.abs() < EPSILON),
            );
        }

        return positions.as_slice();
    }

    let mut curve_count = vertices_length / 6;
    world.clear();
    if closed {
        vertices_length += 2;
        world.resize(vertices_length, 0.0);
        world.fill(0.0);
        compute_attachment_world_vertices(
            skeleton,
            target_slot_index,
            &path.vertices,
            2,
            vertices_length.saturating_sub(4),
            world,
            0,
            2,
        );
        compute_attachment_world_vertices(
            skeleton,
            target_slot_index,
            &path.vertices,
            0,
            2,
            world,
            vertices_length.saturating_sub(4),
            2,
        );
        if vertices_length >= 2 {
            world[vertices_length - 2] = world[0];
            world[vertices_length - 1] = world[1];
        }
    } else {
        curve_count = curve_count.saturating_sub(1);
        vertices_length = vertices_length.saturating_sub(4);
        world.resize(vertices_length, 0.0);
        world.fill(0.0);
        compute_attachment_world_vertices(
            skeleton,
            target_slot_index,
            &path.vertices,
            2,
            vertices_length,
            world,
            0,
            2,
        );
    }

    let world = world.as_slice();
    curves.resize(curve_count, 0.0);
    let curves = curves.as_mut_slice();
    let mut path_length = 0.0f32;
    let mut x1 = world.first().copied().unwrap_or(0.0);
    let mut y1 = world.get(1).copied().unwrap_or(0.0);
    let mut cx1 = 0.0f32;
    let mut cy1 = 0.0f32;
    let mut cx2 = 0.0f32;
    let mut cy2 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y2 = 0.0f32;
    let mut w = 2usize;
    for curve in curves.iter_mut().take(curve_count) {
        cx1 = *world.get(w).unwrap_or(&0.0);
        cy1 = *world.get(w + 1).unwrap_or(&0.0);
        cx2 = *world.get(w + 2).unwrap_or(&0.0);
        cy2 = *world.get(w + 3).unwrap_or(&0.0);
        x2 = *world.get(w + 4).unwrap_or(&0.0);
        y2 = *world.get(w + 5).unwrap_or(&0.0);

        let tmpx = (x1 - cx1 * 2.0 + cx2) * 0.1875;
        let tmpy = (y1 - cy1 * 2.0 + cy2) * 0.1875;
        let dddfx = ((cx1 - cx2) * 3.0 - x1 + x2) * 0.09375;
        let dddfy = ((cy1 - cy2) * 3.0 - y1 + y2) * 0.09375;
        let mut ddfx = tmpx * 2.0 + dddfx;
        let mut ddfy = tmpy * 2.0 + dddfy;
        let mut dfx = (cx1 - x1) * 0.75 + tmpx + dddfx * 0.16666667;
        let mut dfy = (cy1 - y1) * 0.75 + tmpy + dddfy * 0.16666667;

        path_length += (dfx * dfx + dfy * dfy).sqrt();
        dfx += ddfx;
        dfy += ddfy;
        ddfx += dddfx;
        ddfy += dddfy;
        path_length += (dfx * dfx + dfy * dfy).sqrt();
        dfx += ddfx;
        dfy += ddfy;
        path_length += (dfx * dfx + dfy * dfy).sqrt();
        dfx += ddfx + dddfx;
        dfy += ddfy + dddfy;
        path_length += (dfx * dfx + dfy * dfy).sqrt();

        *curve = path_length;
        x1 = x2;
        y1 = y2;
        w += 6;
    }

    if position_mode == crate::PositionMode::Percent {
        position *= path_length;
    }

    let multiplier = match spacing_mode {
        crate::SpacingMode::Percent => path_length,
        crate::SpacingMode::Proportional => path_length / spaces_count as f32,
        _ => 1.0,
    };

    let mut segments = [0.0f32; 10];
    let mut curve_length = 0.0f32;
    let mut prev_curve = NONE;
    let mut curve = 0usize;
    let mut segment = 0usize;

    let mut i = 0usize;
    while i < spaces_count {
        let space = spaces.get(i).copied().unwrap_or(0.0) * multiplier;
        position += space;
        let mut p = position;

        if closed {
            p = p.rem_euclid(path_length);
            curve = 0;
        } else if p < 0.0 {
            add_before_position(p, world, 0, output, i * 3);
            i += 1;
            continue;
        } else if p > path_length {
            add_after_position(
                p - path_length,
                world,
                vertices_length.saturating_sub(4),
                output,
                i * 3,
            );
            i += 1;
            continue;
        }

        loop {
            if curve >= curves.len() {
                break;
            }
            let length = curves[curve];
            if p > length {
                curve += 1;
                continue;
            }
            if curve == 0 {
                p /= length.max(EPSILON);
            } else {
                let prev = curves[curve - 1];
                p = (p - prev) / (length - prev).max(EPSILON);
            }
            break;
        }

        if curve as i32 != prev_curve {
            prev_curve = curve as i32;
            let ii = curve * 6;
            x1 = *world.get(ii).unwrap_or(&0.0);
            y1 = *world.get(ii + 1).unwrap_or(&0.0);
            cx1 = *world.get(ii + 2).unwrap_or(&0.0);
            cy1 = *world.get(ii + 3).unwrap_or(&0.0);
            cx2 = *world.get(ii + 4).unwrap_or(&0.0);
            cy2 = *world.get(ii + 5).unwrap_or(&0.0);
            x2 = *world.get(ii + 6).unwrap_or(&0.0);
            y2 = *world.get(ii + 7).unwrap_or(&0.0);

            let tmpx = (x1 - cx1 * 2.0 + cx2) * 0.03;
            let tmpy = (y1 - cy1 * 2.0 + cy2) * 0.03;
            let dddfx = ((cx1 - cx2) * 3.0 - x1 + x2) * 0.006;
            let dddfy = ((cy1 - cy2) * 3.0 - y1 + y2) * 0.006;
            let mut ddfx = tmpx * 2.0 + dddfx;
            let mut ddfy = tmpy * 2.0 + dddfy;
            let mut dfx = (cx1 - x1) * 0.3 + tmpx + dddfx * 0.16666667;
            let mut dfy = (cy1 - y1) * 0.3 + tmpy + dddfy * 0.16666667;

            curve_length = (dfx * dfx + dfy * dfy).sqrt();
            segments[0] = curve_length;
            for seg in segments.iter_mut().take(8).skip(1) {
                dfx += ddfx;
                dfy += ddfy;
                ddfx += dddfx;
                ddfy += dddfy;
                curve_length += (dfx * dfx + dfy * dfy).sqrt();
                *seg = curve_length;
            }
            dfx += ddfx;
            dfy += ddfy;
            curve_length += (dfx * dfx + dfy * dfy).sqrt();
            segments[8] = curve_length;
            dfx += ddfx + dddfx;
            dfy += ddfy + dddfy;
            curve_length += (dfx * dfx + dfy * dfy).sqrt();
            segments[9] = curve_length;
            segment = 0;
        }

        p *= curve_length;
        loop {
            let length = segments.get(segment).copied().unwrap_or(curve_length);
            if p > length {
                segment += 1;
                if segment >= 10 {
                    segment = 9;
                    break;
                }
                continue;
            }
            if segment == 0 {
                p /= length.max(EPSILON);
            } else {
                let prev = segments[segment - 1];
                p = segment as f32 + (p - prev) / (length - prev).max(EPSILON);
            }
            break;
        }

        add_curve_position(
            p * 0.1,
            x1,
            y1,
            cx1,
            cy1,
            cx2,
            cy2,
            x2,
            y2,
            output,
            i * 3,
            tangents || (i > 0 && space.abs() < EPSILON),
        );
        i += 1;
    }

    positions.as_slice()
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

fn add_before_position(p: f32, temp: &[f32], i: usize, output: &mut [f32], o: usize) {
    let x1 = *temp.get(i).unwrap_or(&0.0);
    let y1 = *temp.get(i + 1).unwrap_or(&0.0);
    let dx = *temp.get(i + 2).unwrap_or(&x1) - x1;
    let dy = *temp.get(i + 3).unwrap_or(&y1) - y1;
    let r = dy.atan2(dx);
    output[o] = x1 + p * r.cos();
    output[o + 1] = y1 + p * r.sin();
    output[o + 2] = r;
}

fn add_after_position(p: f32, temp: &[f32], i: usize, output: &mut [f32], o: usize) {
    let x1 = *temp.get(i + 2).unwrap_or(&0.0);
    let y1 = *temp.get(i + 3).unwrap_or(&0.0);
    let dx = x1 - *temp.get(i).unwrap_or(&x1);
    let dy = y1 - *temp.get(i + 1).unwrap_or(&y1);
    let r = dy.atan2(dx);
    output[o] = x1 + p * r.cos();
    output[o + 1] = y1 + p * r.sin();
    output[o + 2] = r;
}

#[allow(clippy::too_many_arguments)]
fn add_curve_position(
    p: f32,
    x1: f32,
    y1: f32,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
    x2: f32,
    y2: f32,
    output: &mut [f32],
    o: usize,
    tangents: bool,
) {
    const EPSILON: f32 = 1.0e-5;
    if p < EPSILON || p.is_nan() {
        output[o] = x1;
        output[o + 1] = y1;
        output[o + 2] = (cy1 - y1).atan2(cx1 - x1);
        return;
    }
    let tt = p * p;
    let ttt = tt * p;
    let u = 1.0 - p;
    let uu = u * u;
    let uuu = uu * u;
    let ut = u * p;
    let ut3 = ut * 3.0;
    let uut3 = u * ut3;
    let utt3 = ut3 * p;
    let x = x1 * uuu + cx1 * uut3 + cx2 * utt3 + x2 * ttt;
    let y = y1 * uuu + cy1 * uut3 + cy2 * utt3 + y2 * ttt;
    output[o] = x;
    output[o + 1] = y;
    if tangents {
        if p < 0.001 {
            output[o + 2] = (cy1 - y1).atan2(cx1 - x1);
        } else {
            output[o + 2] = (y - (y1 * uu + cy1 * ut * 2.0 + cy2 * tt))
                .atan2(x - (x1 * uu + cx1 * ut * 2.0 + cx2 * tt));
        }
    }
}
