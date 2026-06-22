use crate::SkeletonData;

#[derive(Clone, Debug, Default)]
pub(super) struct PathConstraintScratch {
    pub(super) spaces: Vec<f32>,
    pub(super) positions: Vec<f32>,
    pub(super) world: Vec<f32>,
    pub(super) curves: Vec<f32>,
    pub(super) lengths: Vec<f32>,
}

pub(super) fn estimate_path_attachment_scratch_capacities(
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
