use super::{Skeleton, compute_attachment_world_vertices};
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

pub(super) fn path_attachment_for_slot(
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
pub(super) fn compute_path_world_positions<'a>(
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
