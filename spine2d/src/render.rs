use crate::{Atlas, AttachmentData, BlendMode, MeshVertices, Skeleton, geometry::SkeletonClipper};
use std::borrow::Cow;

fn effective_attachment_path<'a>(
    base_path: &'a str,
    sequence: Option<&crate::SequenceDef>,
    sequence_index: i32,
) -> Cow<'a, str> {
    let Some(sequence) = sequence else {
        return Cow::Borrowed(base_path);
    };

    if sequence.count == 0 {
        return Cow::Borrowed(base_path);
    }

    let mut index = sequence_index;
    if index == -1 {
        index = sequence.setup_index;
    }
    index = index.clamp(0, i32::try_from(sequence.count).unwrap_or(i32::MAX) - 1);

    let frame_number = sequence.start.saturating_add(index);
    let mut out = String::with_capacity(base_path.len() + sequence.digits.max(1));
    out.push_str(base_path);
    if sequence.digits > 0 {
        out.push_str(&format!(
            "{:0width$}",
            frame_number,
            width = sequence.digits
        ));
    } else {
        out.push_str(&frame_number.to_string());
    }
    Cow::Owned(out)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub dark_color: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Draw {
    pub texture_path: String,
    pub blend: BlendMode,
    pub premultiplied_alpha: bool,
    pub first_index: usize,
    pub index_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawList {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub draws: Vec<Draw>,
}

impl DrawList {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.draws.clear();
    }
}

pub fn build_draw_list(skeleton: &Skeleton) -> DrawList {
    let mut out = DrawList::default();
    append_draw_list(&mut out, skeleton);
    out
}

pub fn append_draw_list(out: &mut DrawList, skeleton: &Skeleton) {
    append_draw_list_internal(out, skeleton, None);
}

pub fn build_draw_list_with_atlas(skeleton: &Skeleton, atlas: &Atlas) -> DrawList {
    let mut out = DrawList::default();
    append_draw_list_internal(&mut out, skeleton, Some(atlas));
    out
}

pub fn append_draw_list_with_atlas(out: &mut DrawList, skeleton: &Skeleton, atlas: &Atlas) {
    append_draw_list_internal(out, skeleton, Some(atlas));
}

fn append_draw_list_internal(out: &mut DrawList, skeleton: &Skeleton, atlas: Option<&Atlas>) {
    let mut clipper = SkeletonClipper::default();
    let mut clip_end_slot: Option<usize> = None;

    for &slot_index in skeleton.draw_order() {
        // Match spine runtimes' clipping semantics:
        // - `clipEnd(slot)` is called for null attachments and for early-outs on region/mesh.
        // - `clipEnd(slot)` is NOT called for clipping attachments (they `continue` after `clipStart`).
        let mut call_clip_end_for_slot = true;

        'process_slot: {
            let Some(slot) = skeleton.slots.get(slot_index) else {
                break 'process_slot;
            };
            let Some(attachment) = slot.get_applied_attachment(skeleton) else {
                break 'process_slot;
            };

            match attachment {
                AttachmentData::Region(region) => {
                    let Some(bone) = skeleton.bones.get(slot.bone) else {
                        break 'process_slot;
                    };
                    // Match spine-cpp `SkeletonRenderer` early-outs:
                    // - Skip region/mesh attachments when the slot color alpha is 0.
                    // - Skip when the slot's bone is inactive (skinRequired bones not included by the current skin).
                    let slot_color = slot.applied_color();
                    if slot_color[3] <= 0.0 || !bone.active {
                        break 'process_slot;
                    }
                    // Match spine runtimes: region attachments have their own tint and can be alpha-zero.
                    if region.color[3] <= 0.0 {
                        break 'process_slot;
                    }

                    let attachment_path = effective_attachment_path(
                        region.path.as_str(),
                        region.sequence.as_ref(),
                        slot.applied_sequence_index(),
                    );
                    let atlas_region_opt =
                        atlas.and_then(|a| a.find_region(attachment_path.as_ref()));

                    let local = region_local_vertices_with_atlas_region(
                        region.x,
                        region.y,
                        region.rotation,
                        region.width,
                        region.height,
                        region.scale_x,
                        region.scale_y,
                        atlas_region_opt,
                    );
                    let world = local.map(|(x, y)| {
                        (
                            bone.a * x + bone.b * y + bone.world_x,
                            bone.c * x + bone.d * y + bone.world_y,
                        )
                    });

                    let blend = skeleton
                        .data
                        .slots
                        .get(slot.data_index)
                        .map_or(BlendMode::Normal, |data| data.blend);
                    let (texture_path, uvs, premultiplied_alpha) = if let Some(atlas) = atlas {
                        if let Some(atlas_region) = atlas_region_opt {
                            let page = atlas.pages.get(atlas_region.page);
                            if let Some(page) = page {
                                if page.width > 0 && page.height > 0 {
                                    let uvs = atlas_region_uvs_for_region_attachment(atlas_region);
                                    (page.texture_path.clone(), uvs, page.pma)
                                } else {
                                    (
                                        attachment_path.to_string(),
                                        [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
                                        false,
                                    )
                                }
                            } else {
                                (
                                    attachment_path.to_string(),
                                    [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
                                    false,
                                )
                            }
                        } else {
                            (
                                attachment_path.to_string(),
                                [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
                                false,
                            )
                        }
                    } else {
                        (
                            attachment_path.to_string(),
                            [[1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
                            false,
                        )
                    };

                    let light_unpma =
                        multiply_rgba(multiply_rgba(skeleton.color, slot_color), region.color);
                    let light_alpha = light_unpma[3];
                    let color = apply_pma(light_unpma, premultiplied_alpha);
                    let dark_color = slot_dark_color_rgba(slot, premultiplied_alpha, light_alpha);

                    if !clipper.is_clipping() {
                        let vertices = vec![
                            Vertex {
                                position: [world[0].0, world[0].1],
                                uv: uvs[0],
                                color,
                                dark_color,
                            },
                            Vertex {
                                position: [world[1].0, world[1].1],
                                uv: uvs[1],
                                color,
                                dark_color,
                            },
                            Vertex {
                                position: [world[2].0, world[2].1],
                                uv: uvs[2],
                                color,
                                dark_color,
                            },
                            Vertex {
                                position: [world[3].0, world[3].1],
                                uv: uvs[3],
                                color,
                                dark_color,
                            },
                        ];
                        append_indexed(
                            out,
                            &texture_path,
                            blend,
                            premultiplied_alpha,
                            vertices,
                            &[0_u32, 1, 2, 2, 3, 0],
                        );
                    } else {
                        let positions: [f32; 8] = [
                            world[0].0, world[0].1, world[1].0, world[1].1, world[2].0, world[2].1,
                            world[3].0, world[3].1,
                        ];
                        let uvs_flat: [f32; 8] = [
                            uvs[0][0], uvs[0][1], uvs[1][0], uvs[1][1], uvs[2][0], uvs[2][1],
                            uvs[3][0], uvs[3][1],
                        ];
                        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

                        let (clipped_pos, clipped_uv, clipped_idx) =
                            clipper.clip_triangles(&positions, &indices, &uvs_flat, 2);
                        if clipped_pos.is_empty() || clipped_uv.is_empty() || clipped_idx.is_empty()
                        {
                            break 'process_slot;
                        }

                        let mut clipped_vertices: Vec<Vertex> =
                            Vec::with_capacity(clipped_pos.len() / 2);
                        for i in 0..(clipped_pos.len() / 2) {
                            clipped_vertices.push(Vertex {
                                position: [clipped_pos[i * 2], clipped_pos[i * 2 + 1]],
                                uv: [clipped_uv[i * 2], clipped_uv[i * 2 + 1]],
                                color,
                                dark_color,
                            });
                        }

                        append_indexed_u16(
                            out,
                            &texture_path,
                            blend,
                            premultiplied_alpha,
                            clipped_vertices,
                            &clipped_idx,
                        );
                    }
                }
                AttachmentData::Point(_) => {}
                AttachmentData::Path(_) => {}
                AttachmentData::BoundingBox(_) => {}
                AttachmentData::Clipping(clip) => {
                    if clipper.is_clipping() {
                        call_clip_end_for_slot = false;
                        break 'process_slot;
                    }

                    let Some(slot) = skeleton.slots.get(slot_index) else {
                        break 'process_slot;
                    };
                    let Some(bone) = skeleton.bones.get(slot.bone) else {
                        break 'process_slot;
                    };
                    // Match spine runtimes: clipping attachments do nothing when their slot's bone is inactive.
                    if !bone.active {
                        break 'process_slot;
                    }

                    call_clip_end_for_slot = false;
                    let deform = slot.applied_deform();

                    let polygon =
                        attachment_world_positions(skeleton, slot_index, &clip.vertices, deform);
                    if polygon.len() < 3 {
                        break 'process_slot;
                    }

                    let mut polygon_flat: Vec<f32> = Vec::with_capacity(polygon.len() * 2);
                    for p in polygon {
                        polygon_flat.push(p[0]);
                        polygon_flat.push(p[1]);
                    }

                    if clipper.clip_start(&polygon_flat, clip.convex, clip.inverse) {
                        clip_end_slot = clip.end_slot;
                    }
                }
                AttachmentData::Mesh(mesh) => {
                    let Some(slot) = skeleton.slots.get(slot_index) else {
                        break 'process_slot;
                    };
                    let Some(bone) = skeleton.bones.get(slot.bone) else {
                        break 'process_slot;
                    };
                    // Match spine-cpp `SkeletonRenderer` early-outs (see region case).
                    let slot_color = slot.applied_color();
                    if slot_color[3] <= 0.0 || !bone.active {
                        break 'process_slot;
                    }
                    if mesh.color[3] <= 0.0 {
                        break 'process_slot;
                    }
                    let deform = slot.applied_deform();

                    let blend = skeleton
                        .data
                        .slots
                        .get(slot.data_index)
                        .map_or(BlendMode::Normal, |data| data.blend);
                    let attachment_path = effective_attachment_path(
                        mesh.path.as_str(),
                        mesh.sequence.as_ref(),
                        slot.applied_sequence_index(),
                    );
                    let (texture_path, atlas_region_and_page, premultiplied_alpha) =
                        if let Some(atlas) = atlas {
                            if let Some(atlas_region) = atlas.find_region(attachment_path.as_ref())
                            {
                                if let Some(page) = atlas.pages.get(atlas_region.page) {
                                    if page.width > 0 && page.height > 0 {
                                        (
                                            page.texture_path.clone(),
                                            Some((atlas_region, page)),
                                            page.pma,
                                        )
                                    } else {
                                        (attachment_path.to_string(), None, false)
                                    }
                                } else {
                                    (attachment_path.to_string(), None, false)
                                }
                            } else {
                                (attachment_path.to_string(), None, false)
                            }
                        } else {
                            (attachment_path.to_string(), None, false)
                        };

                    let light_unpma =
                        multiply_rgba(multiply_rgba(skeleton.color, slot_color), mesh.color);
                    let light_alpha = light_unpma[3];
                    let color = apply_pma(light_unpma, premultiplied_alpha);
                    let dark_color = slot_dark_color_rgba(slot, premultiplied_alpha, light_alpha);

                    let world_positions: Vec<[f32; 2]> = match &mesh.vertices {
                        MeshVertices::Unweighted(vertices) => {
                            let use_deform =
                                !deform.is_empty() && deform.len() >= vertices.len() * 2;
                            vertices
                                .iter()
                                .enumerate()
                                .map(|(i, p)| {
                                    let (x, y) = if use_deform {
                                        (deform[i * 2], deform[i * 2 + 1])
                                    } else {
                                        (p[0], p[1])
                                    };
                                    [
                                        bone.a * x + bone.b * y + bone.world_x,
                                        bone.c * x + bone.d * y + bone.world_y,
                                    ]
                                })
                                .collect()
                        }
                        MeshVertices::Weighted(vertices) => {
                            let mut f = 0usize;
                            vertices
                                .iter()
                                .map(|weights| {
                                    let mut wx = 0.0;
                                    let mut wy = 0.0;
                                    for w in weights {
                                        let Some(b) = skeleton.bones.get(w.bone) else {
                                            f = f.saturating_add(2);
                                            continue;
                                        };
                                        let dx = deform.get(f).copied().unwrap_or(0.0);
                                        let dy = deform.get(f + 1).copied().unwrap_or(0.0);
                                        f += 2;
                                        let vx = w.x + dx;
                                        let vy = w.y + dy;
                                        let x = b.a * vx + b.b * vy + b.world_x;
                                        let y = b.c * vx + b.d * vy + b.world_y;
                                        wx += x * w.weight;
                                        wy += y * w.weight;
                                    }
                                    [wx, wy]
                                })
                                .collect()
                        }
                    };

                    if !clipper.is_clipping() {
                        let mut vertices = Vec::with_capacity(world_positions.len());
                        for (i, pos) in world_positions.iter().enumerate() {
                            let uv = mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]);
                            let uv = atlas_region_and_page
                                .map(|(r, p)| map_mesh_uv_to_page(uv, r, p))
                                .unwrap_or(uv);

                            vertices.push(Vertex {
                                position: [pos[0], pos[1]],
                                uv,
                                color,
                                dark_color,
                            });
                        }

                        append_indexed(
                            out,
                            &texture_path,
                            blend,
                            premultiplied_alpha,
                            vertices,
                            &mesh.triangles,
                        );
                    } else {
                        let mut positions: Vec<f32> = Vec::with_capacity(world_positions.len() * 2);
                        let mut uvs_flat: Vec<f32> = Vec::with_capacity(world_positions.len() * 2);

                        for (i, pos) in world_positions.iter().enumerate() {
                            let uv = mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]);
                            let uv = atlas_region_and_page
                                .map(|(r, p)| map_mesh_uv_to_page(uv, r, p))
                                .unwrap_or(uv);

                            positions.push(pos[0]);
                            positions.push(pos[1]);
                            uvs_flat.push(uv[0]);
                            uvs_flat.push(uv[1]);
                        }

                        let mut indices_u16: Vec<u16> = Vec::with_capacity(mesh.triangles.len());
                        for &idx in &mesh.triangles {
                            let Ok(v) = u16::try_from(idx) else {
                                break 'process_slot;
                            };
                            indices_u16.push(v);
                        }

                        let (clipped_pos, clipped_uv, clipped_idx) =
                            clipper.clip_triangles(&positions, &indices_u16, &uvs_flat, 2);
                        if clipped_pos.is_empty() || clipped_uv.is_empty() || clipped_idx.is_empty()
                        {
                            break 'process_slot;
                        }

                        let mut clipped_vertices: Vec<Vertex> =
                            Vec::with_capacity(clipped_pos.len() / 2);
                        for i in 0..(clipped_pos.len() / 2) {
                            clipped_vertices.push(Vertex {
                                position: [clipped_pos[i * 2], clipped_pos[i * 2 + 1]],
                                uv: [clipped_uv[i * 2], clipped_uv[i * 2 + 1]],
                                color,
                                dark_color,
                            });
                        }

                        append_indexed_u16(
                            out,
                            &texture_path,
                            blend,
                            premultiplied_alpha,
                            clipped_vertices,
                            &clipped_idx,
                        );
                    }
                }
            }
        }

        if call_clip_end_for_slot && clipper.is_clipping() && clip_end_slot == Some(slot_index) {
            clipper.clip_end();
            clip_end_slot = None;
        }
    }

    clipper.clip_end();
}

fn append_indexed_u16(
    out: &mut DrawList,
    texture_path: &str,
    blend: BlendMode,
    premultiplied_alpha: bool,
    vertices: Vec<Vertex>,
    indices: &[u16],
) {
    let indices = indices.iter().map(|&idx| idx as u32).collect::<Vec<_>>();
    append_indexed(
        out,
        texture_path,
        blend,
        premultiplied_alpha,
        vertices,
        &indices,
    );
}

fn append_indexed(
    out: &mut DrawList,
    texture_path: &str,
    blend: BlendMode,
    premultiplied_alpha: bool,
    vertices: Vec<Vertex>,
    indices: &[u32],
) {
    if vertices.is_empty() || indices.is_empty() {
        return;
    }

    let base = out.vertices.len() as u32;
    out.vertices.extend(vertices);

    let first_index = out.indices.len();
    out.indices.extend(indices.iter().map(|&idx| base + idx));

    if let Some(last) = out.draws.last_mut() {
        let expected = last.first_index + last.index_count;
        let last_first_index = out.indices[last.first_index] as usize;
        let first_new_index = base as usize;
        let colors_match = out
            .vertices
            .get(last_first_index)
            .zip(out.vertices.get(first_new_index))
            .map(|(last, new)| last.color == new.color && last.dark_color == new.dark_color)
            .unwrap_or(false);

        if last.texture_path == texture_path
            && last.blend == blend
            && last.premultiplied_alpha == premultiplied_alpha
            && expected == first_index
            && colors_match
            && last.index_count + indices.len() < 0xffff
        {
            last.index_count += indices.len();
            return;
        }
    }

    out.draws.push(Draw {
        texture_path: texture_path.to_string(),
        blend,
        premultiplied_alpha,
        first_index,
        index_count: indices.len(),
    });
}

#[allow(clippy::too_many_arguments)]
fn region_local_vertices_with_atlas_region(
    attachment_x: f32,
    attachment_y: f32,
    rotation_degrees: f32,
    width: f32,
    height: f32,
    scale_x: f32,
    scale_y: f32,
    atlas_region: Option<&crate::AtlasRegion>,
) -> [(f32, f32); 4] {
    // Ported from upstream `RegionAttachment::computeUVs()` (spine-cpp). This produces
    // the 4 local vertices (after attachment rotation) in the same order as `spine-cpp`
    // `RegionAttachment.computeWorldVertices`: BR, BL, UL, UR.
    let (region_scale_x, region_scale_y) = if let Some(r) = atlas_region {
        let ow = r.original_width.max(1) as f32;
        let oh = r.original_height.max(1) as f32;
        (width / ow * scale_x, height / oh * scale_y)
    } else {
        (scale_x, scale_y)
    };

    let (local_x, local_y, local_x2, local_y2) = if let Some(r) = atlas_region {
        let ox = r.offset_x as f32;
        let oy = r.offset_y as f32;
        let local_x = -width * 0.5 * scale_x + ox * region_scale_x;
        let local_y = -height * 0.5 * scale_y + oy * region_scale_y;
        let (packed_x, packed_y) = if r.degrees == 90 {
            (r.packed_height, r.packed_width)
        } else {
            (r.packed_width, r.packed_height)
        };
        let local_x2 = local_x + packed_x as f32 * region_scale_x;
        let local_y2 = local_y + packed_y as f32 * region_scale_y;
        (local_x, local_y, local_x2, local_y2)
    } else {
        (
            -width * 0.5 * scale_x,
            -height * 0.5 * scale_y,
            width * 0.5 * scale_x,
            height * 0.5 * scale_y,
        )
    };

    let r = rotation_degrees.to_radians();
    let cos = r.cos();
    let sin = r.sin();

    let x = attachment_x;
    let y = attachment_y;

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

    [br, bl, ul, ur]
}

fn atlas_region_uvs_for_region_attachment(region: &crate::AtlasRegion) -> [[f32; 2]; 4] {
    let u = region.u;
    let v = region.v;
    let u2 = region.u2;
    let v2 = region.v2;

    // Mirror the upstream `RegionAttachment.updateRegion()` UV assignment, expressed in the same
    // vertex order as `region_local_vertices_with_atlas_region`: BR, BL, UL, UR.
    if region.degrees == 90 {
        [[u2, v], [u2, v2], [u, v2], [u, v]]
    } else {
        [[u2, v2], [u, v2], [u, v], [u2, v]]
    }
}

fn map_mesh_uv_to_page(
    region_uv: [f32; 2],
    region: &crate::AtlasRegion,
    page: &crate::AtlasPage,
) -> [f32; 2] {
    // Ported from upstream `MeshAttachment::computeUVs()` (spine-cpp).
    let tex_w = page.width.max(1) as f32;
    let tex_h = page.height.max(1) as f32;

    let mut u = region.u;
    let mut v = region.v;

    let ow = region.original_width.max(1) as f32;
    let oh = region.original_height.max(1) as f32;
    let ox = region.offset_x as f32;
    let oy = region.offset_y as f32;
    let pw = region.packed_width as f32;
    let ph = region.packed_height as f32;

    let width;
    let height;
    match region.degrees {
        90 => {
            u -= (oh - oy - pw) / tex_w;
            v -= (ow - ox - ph) / tex_h;
            width = oh / tex_w;
            height = ow / tex_h;
            [u + region_uv[1] * width, v + (1.0 - region_uv[0]) * height]
        }
        180 => {
            u -= (ow - ox - pw) / tex_w;
            v -= oy / tex_h;
            width = ow / tex_w;
            height = oh / tex_h;
            [
                u + (1.0 - region_uv[0]) * width,
                v + (1.0 - region_uv[1]) * height,
            ]
        }
        270 => {
            u -= oy / tex_w;
            v -= ox / tex_h;
            width = oh / tex_w;
            height = ow / tex_h;
            [u + (1.0 - region_uv[1]) * width, v + region_uv[0] * height]
        }
        _ => {
            u -= ox / tex_w;
            v -= (oh - oy - ph) / tex_h;
            width = ow / tex_w;
            height = oh / tex_h;
            [u + region_uv[0] * width, v + region_uv[1] * height]
        }
    }
}

fn apply_pma(mut color: [f32; 4], premultiplied_alpha: bool) -> [f32; 4] {
    if premultiplied_alpha {
        let a = color[3];
        color[0] *= a;
        color[1] *= a;
        color[2] *= a;
    }
    color
}

fn multiply_rgba(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

fn slot_dark_color_rgba(
    slot: &crate::runtime::Slot,
    premultiplied_alpha: bool,
    light_alpha: f32,
) -> [f32; 4] {
    // Mirror the renderer oracle's two-color packing convention:
    // - No dark color: (0,0,0,1) so the shader becomes a no-op for the dark term.
    // - With dark color:
    //   - PMA: dark.rgb is premultiplied by the *final* light alpha, dark.a=1.
    //   - non-PMA: dark.rgb is not premultiplied, dark.a=0 (shader formula switch).
    if !slot.applied_has_dark_color() {
        return [0.0, 0.0, 0.0, 1.0];
    }

    if premultiplied_alpha {
        [
            slot.applied_dark_color()[0] * light_alpha,
            slot.applied_dark_color()[1] * light_alpha,
            slot.applied_dark_color()[2] * light_alpha,
            1.0,
        ]
    } else {
        [
            slot.applied_dark_color()[0],
            slot.applied_dark_color()[1],
            slot.applied_dark_color()[2],
            0.0,
        ]
    }
}

fn attachment_world_positions(
    skeleton: &Skeleton,
    slot_index: usize,
    vertices: &MeshVertices,
    deform: &[f32],
) -> Vec<[f32; 2]> {
    let Some(slot) = skeleton.slots.get(slot_index) else {
        return Vec::new();
    };

    match vertices {
        MeshVertices::Unweighted(points) => {
            let Some(bone) = skeleton.bones.get(slot.bone) else {
                return Vec::new();
            };
            let use_deform = !deform.is_empty() && deform.len() >= points.len() * 2;
            points
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let (x, y) = if use_deform {
                        (deform[i * 2], deform[i * 2 + 1])
                    } else {
                        (p[0], p[1])
                    };
                    [
                        bone.a * x + bone.b * y + bone.world_x,
                        bone.c * x + bone.d * y + bone.world_y,
                    ]
                })
                .collect()
        }
        MeshVertices::Weighted(points) => {
            let mut f = 0usize;
            points
                .iter()
                .map(|weights| {
                    let mut wx = 0.0;
                    let mut wy = 0.0;
                    for w in weights {
                        let Some(b) = skeleton.bones.get(w.bone) else {
                            f = f.saturating_add(2);
                            continue;
                        };
                        let dx = deform.get(f).copied().unwrap_or(0.0);
                        let dy = deform.get(f + 1).copied().unwrap_or(0.0);
                        f += 2;
                        let vx = w.x + dx;
                        let vy = w.y + dy;
                        let x = b.a * vx + b.b * vy + b.world_x;
                        let y = b.c * vx + b.d * vy + b.world_y;
                        wx += x * w.weight;
                        wy += y * w.weight;
                    }
                    [wx, wy]
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_vertices(color: [f32; 4], dark_color: [f32; 4]) -> Vec<Vertex> {
        vec![
            Vertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
                color,
                dark_color,
            };
            4
        ]
    }

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 2, 3, 0]
    }

    #[test]
    fn append_indexed_batches_when_texture_blend_pma_and_first_colors_match() {
        let mut out = DrawList::default();
        let color = [0.25, 0.5, 0.75, 1.0];
        let dark_color = [0.1, 0.2, 0.3, 0.4];

        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices(color, dark_color),
            &quad_indices(),
        );
        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices(color, dark_color),
            &quad_indices(),
        );

        assert_eq!(out.draws.len(), 1);
        assert_eq!(out.draws[0].first_index, 0);
        assert_eq!(out.draws[0].index_count, 12);
    }

    #[test]
    fn append_indexed_splits_when_first_vertex_color_differs() {
        let mut out = DrawList::default();
        let dark_color = [0.1, 0.2, 0.3, 0.4];

        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices([1.0, 0.0, 0.0, 1.0], dark_color),
            &quad_indices(),
        );
        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices([0.0, 1.0, 0.0, 1.0], dark_color),
            &quad_indices(),
        );

        assert_eq!(out.draws.len(), 2);
    }

    #[test]
    fn append_indexed_splits_when_first_vertex_dark_color_differs() {
        let mut out = DrawList::default();
        let color = [0.25, 0.5, 0.75, 1.0];

        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices(color, [0.1, 0.2, 0.3, 0.4]),
            &quad_indices(),
        );
        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices(color, [0.4, 0.3, 0.2, 0.1]),
            &quad_indices(),
        );

        assert_eq!(out.draws.len(), 2);
    }

    #[test]
    fn append_indexed_splits_before_16bit_index_limit() {
        let mut out = DrawList::default();
        let color = [0.25, 0.5, 0.75, 1.0];
        let dark_color = [0.1, 0.2, 0.3, 0.4];
        let large_indices: Vec<u32> = (0..65532).map(|i| (i % 4) as u32).collect();

        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices(color, dark_color),
            &large_indices,
        );
        append_indexed(
            &mut out,
            "page.png",
            BlendMode::Normal,
            false,
            quad_vertices(color, dark_color),
            &quad_indices(),
        );

        assert_eq!(out.draws.len(), 2);
        assert_eq!(out.draws[0].index_count, 65532);
        assert_eq!(out.draws[1].index_count, 6);
    }
}
