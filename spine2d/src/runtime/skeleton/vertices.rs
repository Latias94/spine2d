use super::Skeleton;

#[allow(clippy::too_many_arguments)]
pub(super) fn compute_attachment_world_vertices(
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
