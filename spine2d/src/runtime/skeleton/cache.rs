use super::Skeleton;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UpdateCacheItem {
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

pub(super) fn rebuild_update_cache(skeleton: &Skeleton) -> Vec<UpdateCacheItem> {
    let bone_count = skeleton.bones.len();
    let mut out = Vec::<UpdateCacheItem>::new();
    let mut sorted = vec![false; bone_count];

    for (i, sorted) in sorted.iter_mut().enumerate().take(bone_count) {
        *sorted = !skeleton.bones.get(i).map(|b| b.active).unwrap_or(false);
    }

    let current_skin_name = skeleton.skin.as_deref();
    let current_skin = current_skin_name.and_then(|n| skeleton.data.find_skin(n));
    let default_skin = if current_skin_name != Some("default") {
        skeleton.data.find_skin("default")
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
                let Some(data) = skeleton.data.transform_constraints.get(c.data_index) else {
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
                if let Some(att) = skeleton.slot_attachment_data_for_pose(c.target, false) {
                    sort_path_attachment(skeleton, att, slot_bone_index, &mut sorted, &mut out);
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
                if let (Some(bone), false) = (c.bone, data.local) {
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
}

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
