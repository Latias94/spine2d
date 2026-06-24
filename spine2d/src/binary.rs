//! Spine `.skel` (binary) loader for Spine 4.3 exports.
//!
//! The loader is IO-free: it operates on an in-memory byte slice.

use crate::{
    AlphaFrame, AlphaTimeline, Animation, AttachmentData, AttachmentFrame, AttachmentTimeline,
    BlendMode, BoneData, BoneTimeline, BoundingBoxAttachmentData, ClippingAttachmentData,
    ColorFrame, ColorTimeline, Curve, DeformFrame, DeformTimeline, DrawOrderFolderFrame,
    DrawOrderFolderTimeline, Error, Event, EventTimeline, FloatFrame, IkConstraintData,
    IkConstraintTimeline, IkFrame, Inherit, InheritFrame, InheritTimeline, MeshAttachmentData,
    MeshVertices, PathAttachmentData, PathConstraintData, PathConstraintMixTimeline,
    PathConstraintPositionTimeline, PathConstraintSpacingTimeline, PathConstraintTimeline,
    PathMixFrame, PointAttachmentData, PositionMode, RegionAttachmentData, Rgb2Frame, Rgb2Timeline,
    RgbFrame, RgbTimeline, Rgba2Frame, Rgba2Timeline, RotateFrame, RotateMode, RotateTimeline,
    ScaleTimeline, ScaleXTimeline, ScaleYTimeline, SequenceDef, SequenceFrame, SequenceMode,
    SequenceTimeline, ShearTimeline, ShearXTimeline, ShearYTimeline, SkinData, SlotData,
    SpacingMode, TimelineKind, TransformConstraintData, TransformConstraintTimeline,
    TransformFrame, TranslateTimeline, TranslateXTimeline, TranslateYTimeline, Vec2Frame,
    VertexWeight,
};
use byteorder::{BigEndian, ByteOrder};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

const CURVE_LINEAR: i8 = 0;
const CURVE_STEPPED: i8 = 1;
const CURVE_BEZIER: i8 = 2;

const ATTACHMENT_DEFORM: u8 = 0;
const ATTACHMENT_SEQUENCE: u8 = 1;

const SLOT_ATTACHMENT: u8 = 0;
const SLOT_RGBA: u8 = 1;
const SLOT_RGB: u8 = 2;
const SLOT_RGBA2: u8 = 3;
const SLOT_RGB2: u8 = 4;
const SLOT_ALPHA: u8 = 5;

const BONE_ROTATE: u8 = 0;
const BONE_TRANSLATE: u8 = 1;
const BONE_TRANSLATEX: u8 = 2;
const BONE_TRANSLATEY: u8 = 3;
const BONE_SCALE: u8 = 4;
const BONE_SCALEX: u8 = 5;
const BONE_SCALEY: u8 = 6;
const BONE_SHEAR: u8 = 7;
const BONE_SHEARX: u8 = 8;
const BONE_SHEARY: u8 = 9;
const BONE_INHERIT: u8 = 10;

const PATH_POSITION: u8 = 0;
const PATH_SPACING: u8 = 1;
const PATH_MIX: u8 = 2;

const PHYSICS_INERTIA: u8 = 0;
const PHYSICS_STRENGTH: u8 = 1;
const PHYSICS_DAMPING: u8 = 2;
const PHYSICS_MASS: u8 = 4;
const PHYSICS_WIND: u8 = 5;
const PHYSICS_GRAVITY: u8 = 6;
const PHYSICS_MIX: u8 = 7;
const PHYSICS_RESET: u8 = 8;

const SLIDER_TIME: u8 = 0;
const SLIDER_MIX: u8 = 1;

const CONSTRAINT_IK: u8 = 0;
const CONSTRAINT_PATH: u8 = 1;
const CONSTRAINT_TRANSFORM: u8 = 2;
const CONSTRAINT_PHYSICS: u8 = 3;
const CONSTRAINT_SLIDER: u8 = 4;

struct TimelineOrderBuilder {
    order: Vec<TimelineKind>,
}

impl TimelineOrderBuilder {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            order: Vec::with_capacity(capacity),
        }
    }

    fn slot_attachment(&mut self, timelines: &[AttachmentTimeline]) {
        self.record_last(timelines.len(), TimelineKind::SlotAttachment);
    }

    fn slot_color(&mut self, timelines: &[ColorTimeline]) {
        self.record_last(timelines.len(), TimelineKind::SlotColor);
    }

    fn slot_rgb(&mut self, timelines: &[RgbTimeline]) {
        self.record_last(timelines.len(), TimelineKind::SlotRgb);
    }

    fn slot_alpha(&mut self, timelines: &[AlphaTimeline]) {
        self.record_last(timelines.len(), TimelineKind::SlotAlpha);
    }

    fn slot_rgba2(&mut self, timelines: &[Rgba2Timeline]) {
        self.record_last(timelines.len(), TimelineKind::SlotRgba2);
    }

    fn slot_rgb2(&mut self, timelines: &[Rgb2Timeline]) {
        self.record_last(timelines.len(), TimelineKind::SlotRgb2);
    }

    fn bone(&mut self, timelines: &[BoneTimeline]) {
        self.record_last(timelines.len(), TimelineKind::Bone);
    }

    fn ik_constraint(&mut self, timelines: &[IkConstraintTimeline]) {
        self.record_last(timelines.len(), TimelineKind::IkConstraint);
    }

    fn transform_constraint(&mut self, timelines: &[TransformConstraintTimeline]) {
        self.record_last(timelines.len(), TimelineKind::TransformConstraint);
    }

    fn path_constraint(&mut self, timelines: &[PathConstraintTimeline]) {
        self.record_last(timelines.len(), TimelineKind::PathConstraint);
    }

    fn physics_constraint(&mut self, timelines: &[crate::PhysicsConstraintTimeline]) {
        self.record_last(timelines.len(), TimelineKind::PhysicsConstraint);
    }

    fn physics_reset(&mut self, timelines: &[crate::PhysicsConstraintResetTimeline]) {
        self.record_last(timelines.len(), TimelineKind::PhysicsReset);
    }

    fn slider_time(&mut self, timelines: &[crate::SliderConstraintTimeline]) {
        self.record_last(timelines.len(), TimelineKind::SliderTime);
    }

    fn slider_mix(&mut self, timelines: &[crate::SliderConstraintTimeline]) {
        self.record_last(timelines.len(), TimelineKind::SliderMix);
    }

    fn deform(&mut self, timelines: &[DeformTimeline]) {
        self.record_last(timelines.len(), TimelineKind::Deform);
    }

    fn sequence(&mut self, timelines: &[SequenceTimeline]) {
        self.record_last(timelines.len(), TimelineKind::Sequence);
    }

    fn draw_order(&mut self) {
        self.order.push(TimelineKind::DrawOrder);
    }

    fn draw_order_folder(&mut self, timelines: &[DrawOrderFolderTimeline]) {
        self.record_last(timelines.len(), TimelineKind::DrawOrderFolder);
    }

    fn finish(self) -> Vec<TimelineKind> {
        self.order
    }

    fn record_last<F>(&mut self, len: usize, make: F)
    where
        F: FnOnce(usize) -> TimelineKind,
    {
        debug_assert!(
            len > 0,
            "timeline order recorded before timeline was pushed"
        );
        if let Some(index) = len.checked_sub(1) {
            self.order.push(make(index));
        }
    }
}

fn decode_path_constraint_position_mode(flags: u8) -> PositionMode {
    if ((flags >> 1) & 1) != 0 {
        PositionMode::Percent
    } else {
        PositionMode::Fixed
    }
}

#[cfg(test)]
pub(crate) fn decode_path_constraint_position_mode_for_test(flags: u8) -> PositionMode {
    decode_path_constraint_position_mode(flags)
}

// Spine 4.3 binary format stores constraints in a single ordered list (mixed types). Animation
// timelines reference constraints by that combined index, so we need a mapping to our per-type
// vectors.
#[derive(Copy, Clone, Debug)]
enum ConstraintRef {
    Ik(usize),
    Path(usize),
    Transform(usize),
    Physics(usize),
    Slider(usize),
}

#[derive(Clone, Debug)]
struct BinaryInput<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> BinaryInput<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.cursor)
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        if self.cursor >= self.bytes.len() {
            return Err(Error::BinaryParse {
                message: "unexpected EOF".to_string(),
            });
        }
        let b = self.bytes[self.cursor];
        self.cursor += 1;
        Ok(b)
    }

    fn read_i8(&mut self) -> Result<i8, Error> {
        Ok(self.read_u8()? as i8)
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_u8()? != 0)
    }

    fn read_i32_be(&mut self) -> Result<i32, Error> {
        if self.remaining() < 4 {
            return Err(Error::BinaryParse {
                message: "unexpected EOF".to_string(),
            });
        }
        let v = BigEndian::read_i32(&self.bytes[self.cursor..self.cursor + 4]);
        self.cursor += 4;
        Ok(v)
    }

    fn read_i64_be(&mut self) -> Result<i64, Error> {
        if self.remaining() < 8 {
            return Err(Error::BinaryParse {
                message: "unexpected EOF".to_string(),
            });
        }
        let v = BigEndian::read_i64(&self.bytes[self.cursor..self.cursor + 8]);
        self.cursor += 8;
        Ok(v)
    }

    fn read_f32_be(&mut self) -> Result<f32, Error> {
        if self.remaining() < 4 {
            return Err(Error::BinaryParse {
                message: "unexpected EOF".to_string(),
            });
        }
        let v = BigEndian::read_f32(&self.bytes[self.cursor..self.cursor + 4]);
        self.cursor += 4;
        Ok(v)
    }

    fn read_varint(&mut self, optimize_positive: bool) -> Result<i32, Error> {
        let mut b = self.read_u8()?;
        let mut value: u32 = (b & 0x7F) as u32;
        if (b & 0x80) != 0 {
            b = self.read_u8()?;
            value |= ((b & 0x7F) as u32) << 7;
            if (b & 0x80) != 0 {
                b = self.read_u8()?;
                value |= ((b & 0x7F) as u32) << 14;
                if (b & 0x80) != 0 {
                    b = self.read_u8()?;
                    value |= ((b & 0x7F) as u32) << 21;
                    if (b & 0x80) != 0 {
                        b = self.read_u8()?;
                        value |= ((b & 0x7F) as u32) << 28;
                    }
                }
            }
        }

        if optimize_positive {
            Ok(value as i32)
        } else {
            Ok((value >> 1) as i32 ^ -((value & 1) as i32))
        }
    }

    fn read_string(&mut self) -> Result<Option<String>, Error> {
        fn hex_preview(bytes: &[u8], max: usize) -> String {
            let n = bytes.len().min(max);
            let mut out = String::new();
            for (i, b) in bytes[..n].iter().copied().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                out.push_str(&format!("{b:02x}"));
            }
            if bytes.len() > n {
                out.push_str(" …");
            }
            out
        }

        let length_offset = self.cursor;
        let length = self.read_varint(true)?;
        if length == 0 {
            return Ok(None);
        }
        let length = length as usize;
        if length == 1 {
            return Ok(Some(String::new()));
        }
        let byte_len = length - 1;
        if self.remaining() < byte_len {
            return Err(Error::BinaryParse {
                message: format!(
                    "unexpected EOF while reading string (len={byte_len}) at offset {}",
                    self.cursor
                ),
            });
        }
        let bytes_offset = self.cursor;
        let bytes = &self.bytes[self.cursor..self.cursor + byte_len];
        self.cursor += byte_len;
        let s = std::str::from_utf8(bytes).map_err(|e| Error::BinaryParse {
            message: format!(
                "invalid utf-8 in string at lenOffset={length_offset} bytesOffset={bytes_offset} len={byte_len}: {e}; bytes=[{}]",
                hex_preview(bytes, 48)
            ),
        })?;
        Ok(Some(s.to_string()))
    }

    fn read_string_ref(&mut self, strings: &[String]) -> Result<Option<String>, Error> {
        let offset = self.cursor;
        let idx = self.read_varint(true)?;
        if idx == 0 {
            return Ok(None);
        }
        let i = (idx - 1) as usize;
        let s = strings.get(i).ok_or_else(|| Error::BinaryParse {
            message: format!(
                "invalid stringRef index {idx} (len={}) at offset {offset}",
                strings.len()
            ),
        })?;
        Ok(Some(s.clone()))
    }

    fn read_color_rgba(&mut self) -> Result<[f32; 4], Error> {
        Ok([
            self.read_u8()? as f32 / 255.0,
            self.read_u8()? as f32 / 255.0,
            self.read_u8()? as f32 / 255.0,
            self.read_u8()? as f32 / 255.0,
        ])
    }
}

#[derive(Clone, Debug)]
struct PendingLinkedMesh {
    skin_name: String,
    slot_index: usize,
    attachment_key: String,
    source_slot_index: usize,
    parent_skin_index: usize,
    parent_key: String,
    inherit_timelines: bool,
}

#[derive(Clone, Debug)]
struct ReadVertices {
    vertices: MeshVertices,
    world_vertices_length: usize,
}

fn validate_spine_version(value: &str) -> Result<(), Error> {
    if !crate::version::spine_version_matches_runtime(value) {
        return Err(Error::BinarySpineVersion {
            value: value.to_string(),
        });
    }
    Ok(())
}

fn map_inherit(v: i32) -> Inherit {
    match v {
        0 => Inherit::Normal,
        1 => Inherit::OnlyTranslation,
        2 => Inherit::NoRotationOrReflection,
        3 => Inherit::NoScale,
        4 => Inherit::NoScaleOrReflection,
        _ => Inherit::Normal,
    }
}

fn map_blend(v: i32) -> BlendMode {
    match v {
        0 => BlendMode::Normal,
        1 => BlendMode::Additive,
        2 => BlendMode::Multiply,
        3 => BlendMode::Screen,
        _ => BlendMode::Normal,
    }
}

fn map_sequence_mode(v: i32) -> Result<SequenceMode, Error> {
    Ok(match v {
        0 => SequenceMode::Hold,
        1 => SequenceMode::Once,
        2 => SequenceMode::Loop,
        3 => SequenceMode::PingPong,
        4 => SequenceMode::OnceReverse,
        5 => SequenceMode::LoopReverse,
        6 => SequenceMode::PingPongReverse,
        _ => {
            return Err(Error::BinaryParse {
                message: format!("invalid SequenceMode {v}"),
            });
        }
    })
}

fn read_sequence(input: &mut BinaryInput<'_>) -> Result<SequenceDef, Error> {
    let id = crate::ids::next_sequence_id();
    Ok(SequenceDef {
        id,
        count: input.read_varint(true)? as usize,
        start: input.read_varint(true)?,
        digits: input.read_varint(true)? as usize,
        setup_index: input.read_varint(true)?,
    })
}

fn read_vertices(
    input: &mut BinaryInput<'_>,
    weighted: bool,
    scale: f32,
) -> Result<ReadVertices, Error> {
    let vertex_count = input.read_varint(true)? as usize;
    let world_vertices_length = vertex_count << 1;

    if !weighted {
        let mut out = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            let x = input.read_f32_be()? * scale;
            let y = input.read_f32_be()? * scale;
            out.push([x, y]);
        }
        return Ok(ReadVertices {
            vertices: MeshVertices::Unweighted(out),
            world_vertices_length,
        });
    }

    let bones_buffer_len = input.read_varint(true)? as usize;
    if bones_buffer_len < vertex_count {
        return Err(Error::BinaryParse {
            message: format!(
                "invalid weighted vertices buffer length {bones_buffer_len} for {vertex_count} vertices"
            ),
        });
    }

    let mut weights_per_vertex = Vec::with_capacity(vertex_count);
    let mut consumed_bones = 0usize;
    while consumed_bones < bones_buffer_len {
        let bone_count = input.read_varint(true)? as usize;
        consumed_bones = consumed_bones.saturating_add(1);
        if consumed_bones + bone_count > bones_buffer_len {
            return Err(Error::BinaryParse {
                message: format!(
                    "weighted vertex bone count exceeds buffer length: consumed={consumed_bones} count={bone_count} len={bones_buffer_len}"
                ),
            });
        }
        let mut weights = Vec::with_capacity(bone_count);
        for _ in 0..bone_count {
            let bone = input.read_varint(true)? as usize;
            let x = input.read_f32_be()? * scale;
            let y = input.read_f32_be()? * scale;
            let weight = input.read_f32_be()?;
            weights.push(VertexWeight { bone, x, y, weight });
            consumed_bones += 1;
        }
        weights_per_vertex.push(weights);
    }
    if weights_per_vertex.len() != vertex_count {
        return Err(Error::BinaryParse {
            message: format!(
                "weighted vertices decoded {} vertices, expected {vertex_count}",
                weights_per_vertex.len()
            ),
        });
    }

    Ok(ReadVertices {
        vertices: MeshVertices::Weighted(weights_per_vertex),
        world_vertices_length,
    })
}

#[cfg(test)]
pub(crate) fn read_vertices_for_test(
    bytes: &[u8],
    weighted: bool,
    scale: f32,
) -> Result<(MeshVertices, usize, usize), Error> {
    let mut input = BinaryInput::new(bytes);
    let vertices = read_vertices(&mut input, weighted, scale)?;
    Ok((
        vertices.vertices,
        vertices.world_vertices_length,
        input.cursor,
    ))
}

fn attachment_deform_setup(vertices: &MeshVertices) -> (usize, Option<Vec<f32>>) {
    match vertices {
        MeshVertices::Unweighted(v) => {
            let mut setup = Vec::with_capacity(v.len() * 2);
            for [x, y] in v {
                setup.push(*x);
                setup.push(*y);
            }
            (setup.len(), Some(setup))
        }
        MeshVertices::Weighted(v) => {
            let weight_count = v.iter().map(|vv| vv.len()).sum::<usize>();
            (weight_count * 2, None)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn read_attachment(
    input: &mut BinaryInput<'_>,
    strings: &[String],
    nonessential: bool,
    scale: f32,
    skin_name: &str,
    slot_index: usize,
    attachment_key: &str,
    pending_linked_meshes: &mut Vec<PendingLinkedMesh>,
) -> Result<AttachmentData, Error> {
    let flags = input.read_u8()?;
    let name = if (flags & 8) != 0 {
        input
            .read_string_ref(strings)?
            .unwrap_or_else(|| attachment_key.to_string())
    } else {
        attachment_key.to_string()
    };
    let ty = flags & 0x7;

    match ty {
        0 => {
            // region
            let (path, inherit_path) = if (flags & 16) != 0 {
                (
                    input
                        .read_string_ref(strings)?
                        .unwrap_or_else(|| name.clone()),
                    false,
                )
            } else {
                (name.clone(), true)
            };
            let color = if (flags & 32) != 0 {
                input.read_color_rgba()?
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };
            let sequence = if (flags & 64) != 0 {
                Some(read_sequence(input)?)
            } else {
                None
            };
            let rotation = if (flags & 128) != 0 {
                input.read_f32_be()?
            } else {
                0.0
            };
            let x = input.read_f32_be()? * scale;
            let y = input.read_f32_be()? * scale;
            let scale_x = input.read_f32_be()?;
            let scale_y = input.read_f32_be()?;
            let width = input.read_f32_be()? * scale;
            let height = input.read_f32_be()? * scale;
            let _ = inherit_path;
            Ok(AttachmentData::Region(RegionAttachmentData {
                name,
                path,
                sequence,
                color,
                x,
                y,
                rotation,
                scale_x,
                scale_y,
                width,
                height,
            }))
        }
        1 => {
            // boundingbox
            let weighted = (flags & 16) != 0;
            let v = read_vertices(input, weighted, scale)?;
            if nonessential {
                let _ = input.read_color_rgba()?;
            }
            Ok(AttachmentData::BoundingBox(BoundingBoxAttachmentData {
                vertex_id: crate::ids::next_vertex_attachment_id(),
                name,
                vertices: v.vertices,
            }))
        }
        2 => {
            // mesh
            let trace_binary = std::env::var("SPINE2D_BINARY_TRACE")
                .ok()
                .is_some_and(|v| v == "1");
            let path = if (flags & 16) != 0 {
                input
                    .read_string_ref(strings)?
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            };
            let color = if (flags & 32) != 0 {
                input.read_color_rgba()?
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };
            let sequence = if (flags & 64) != 0 {
                Some(read_sequence(input)?)
            } else {
                None
            };

            let hull_length = input.read_varint(true)? as usize;
            let weighted = (flags & 128) != 0;
            let v = read_vertices(input, weighted, scale)?;

            let uvs_count = v.world_vertices_length;
            let mut uvs = Vec::with_capacity(uvs_count / 2);
            for _ in 0..(uvs_count / 2) {
                let u = input.read_f32_be()?;
                let vv = input.read_f32_be()?;
                uvs.push([u, vv]);
            }

            let triangle_index_count = {
                let base = v.world_vertices_length as isize - hull_length as isize - 2;
                if base < 0 {
                    return Err(Error::BinaryParse {
                        message: format!(
                            "invalid mesh triangle count: worldVerticesLength={} hullLength={}",
                            v.world_vertices_length, hull_length
                        ),
                    });
                }
                (base as usize) * 3
            };
            let mut triangles = Vec::with_capacity(triangle_index_count);
            for _ in 0..triangle_index_count {
                let idx = input.read_varint(true)?;
                if idx < 0 {
                    return Err(Error::BinaryParse {
                        message: format!("invalid mesh triangle index {idx}"),
                    });
                }
                triangles.push(idx as u32);
            }

            let timeline_slot_count = input.read_varint(true)? as usize;
            let mut timeline_slots = Vec::with_capacity(timeline_slot_count);
            for _ in 0..timeline_slot_count {
                timeline_slots.push(input.read_varint(true)? as usize);
            }

            if nonessential {
                let edges_count = input.read_varint(true)? as usize;
                if trace_binary {
                    eprintln!(
                        "    [binary] mesh {name:?} hull={hull_length} worldLen={} triIdx={} timelineSlots={timeline_slot_count} edges={edges_count} weighted={weighted} cursorAfterTriangles={}",
                        v.world_vertices_length, triangle_index_count, input.cursor
                    );
                }
                for _ in 0..edges_count {
                    let idx = input.read_varint(true)?;
                    if idx < 0 {
                        return Err(Error::BinaryParse {
                            message: format!("invalid mesh edge index {idx}"),
                        });
                    }
                }
                let _ = input.read_f32_be()?;
                let _ = input.read_f32_be()?;
            }

            Ok(AttachmentData::Mesh(MeshAttachmentData {
                vertex_id: crate::ids::next_vertex_attachment_id(),
                name,
                path,
                timeline_skin: skin_name.to_string(),
                timeline_attachment: attachment_key.to_string(),
                timeline_slots,
                sequence,
                color,
                vertices: v.vertices,
                uvs,
                triangles,
            }))
        }
        3 => {
            // linkedmesh
            let path = if (flags & 16) != 0 {
                input
                    .read_string_ref(strings)?
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            };
            let color = if (flags & 32) != 0 {
                input.read_color_rgba()?
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };
            let sequence = if (flags & 64) != 0 {
                Some(read_sequence(input)?)
            } else {
                None
            };
            let inherit_timelines = (flags & 128) != 0;
            let source_slot_index = input.read_varint(true)? as usize;
            let parent_skin_index = input.read_varint(true)? as usize;
            let parent_key = input
                .read_string_ref(strings)?
                .ok_or_else(|| Error::BinaryParse {
                    message: "linked mesh missing parent name".to_string(),
                })?;
            if nonessential {
                let _ = input.read_f32_be()? * scale;
                let _ = input.read_f32_be()? * scale;
            }

            pending_linked_meshes.push(PendingLinkedMesh {
                skin_name: skin_name.to_string(),
                slot_index,
                attachment_key: attachment_key.to_string(),
                source_slot_index,
                parent_skin_index,
                parent_key: parent_key.clone(),
                inherit_timelines,
            });

            Ok(AttachmentData::Mesh(MeshAttachmentData {
                vertex_id: crate::ids::next_vertex_attachment_id(),
                name,
                path,
                timeline_skin: skin_name.to_string(),
                timeline_attachment: attachment_key.to_string(),
                timeline_slots: Vec::new(),
                sequence,
                color,
                vertices: MeshVertices::Unweighted(Vec::new()),
                uvs: Vec::new(),
                triangles: Vec::new(),
            }))
        }
        4 => {
            // path
            let closed = (flags & 16) != 0;
            let constant_speed = (flags & 32) != 0;
            let weighted = (flags & 64) != 0;
            let v = read_vertices(input, weighted, scale)?;
            let lengths_len = v.world_vertices_length / 6;
            let mut lengths = Vec::with_capacity(lengths_len);
            for _ in 0..lengths_len {
                lengths.push(input.read_f32_be()? * scale);
            }
            if nonessential {
                let _ = input.read_color_rgba()?;
            }
            Ok(AttachmentData::Path(PathAttachmentData {
                vertex_id: crate::ids::next_vertex_attachment_id(),
                name,
                vertices: v.vertices,
                lengths,
                closed,
                constant_speed,
            }))
        }
        5 => {
            // point
            let rotation = input.read_f32_be()?;
            let x = input.read_f32_be()? * scale;
            let y = input.read_f32_be()? * scale;
            if nonessential {
                let _ = input.read_color_rgba()?;
            }
            Ok(AttachmentData::Point(PointAttachmentData {
                name,
                x,
                y,
                rotation,
            }))
        }
        6 => {
            // clipping
            let end_slot_index = input.read_varint(true)? as usize;
            let weighted = (flags & 16) != 0;
            let v = read_vertices(input, weighted, scale)?;
            if nonessential {
                let _ = input.read_color_rgba()?;
            }
            Ok(AttachmentData::Clipping(ClippingAttachmentData {
                vertex_id: crate::ids::next_vertex_attachment_id(),
                name,
                vertices: v.vertices,
                end_slot: Some(end_slot_index),
                convex: (flags & 32) != 0,
                inverse: (flags & 64) != 0,
            }))
        }
        _ => Err(Error::BinaryParse {
            message: format!("unsupported attachment type {ty}"),
        }),
    }
}

fn read_bezier(input: &mut BinaryInput<'_>, scale: f32) -> Result<Curve, Error> {
    let cx1 = input.read_f32_be()?;
    let cy1 = input.read_f32_be()? * scale;
    let cx2 = input.read_f32_be()?;
    let cy2 = input.read_f32_be()? * scale;
    Ok(Curve::Bezier { cx1, cy1, cx2, cy2 })
}

fn read_curve_1(input: &mut BinaryInput<'_>, scale: f32) -> Result<Curve, Error> {
    match input.read_i8()? {
        CURVE_LINEAR => Ok(Curve::Linear),
        CURVE_STEPPED => Ok(Curve::Stepped),
        CURVE_BEZIER => read_bezier(input, scale),
        other => Err(Error::BinaryParse {
            message: format!("invalid curve type {other}"),
        }),
    }
}

fn read_curve_timeline1(
    input: &mut BinaryInput<'_>,
    frame_count: usize,
    value_scale: f32,
) -> Result<Vec<FloatFrame>, Error> {
    if frame_count == 0 {
        return Ok(Vec::new());
    }
    let mut frames = Vec::with_capacity(frame_count);
    let mut time = input.read_f32_be()?;
    let mut value = input.read_f32_be()? * value_scale;
    for frame in 0..frame_count {
        let curve = if frame + 1 == frame_count {
            Curve::Linear
        } else {
            let time2 = input.read_f32_be()?;
            let value2 = input.read_f32_be()? * value_scale;
            let curve = read_curve_1(input, value_scale)?;
            frames.push(FloatFrame { time, value, curve });
            time = time2;
            value = value2;
            continue;
        };
        frames.push(FloatFrame { time, value, curve });
    }
    Ok(frames)
}

fn read_curve_timeline2(
    input: &mut BinaryInput<'_>,
    frame_count: usize,
    value_scale: f32,
) -> Result<Vec<Vec2Frame>, Error> {
    if frame_count == 0 {
        return Ok(Vec::new());
    }
    let mut frames = Vec::with_capacity(frame_count);
    let mut time = input.read_f32_be()?;
    let mut x = input.read_f32_be()? * value_scale;
    let mut y = input.read_f32_be()? * value_scale;
    for frame in 0..frame_count {
        if frame + 1 == frame_count {
            frames.push(Vec2Frame {
                time,
                x,
                y,
                curve: [Curve::Linear; 2],
            });
            break;
        }

        let time2 = input.read_f32_be()?;
        let x2 = input.read_f32_be()? * value_scale;
        let y2 = input.read_f32_be()? * value_scale;
        let curve_type = input.read_i8()?;
        let curve = match curve_type {
            CURVE_LINEAR => [Curve::Linear; 2],
            CURVE_STEPPED => [Curve::Stepped; 2],
            CURVE_BEZIER => [
                read_bezier(input, value_scale)?,
                read_bezier(input, value_scale)?,
            ],
            other => {
                return Err(Error::BinaryParse {
                    message: format!("invalid curve type {other}"),
                });
            }
        };
        frames.push(Vec2Frame { time, x, y, curve });
        time = time2;
        x = x2;
        y = y2;
    }
    Ok(frames)
}

impl crate::SkeletonData {
    pub fn from_skel_bytes(bytes: &[u8]) -> Result<Arc<Self>, Error> {
        Self::from_skel_bytes_with_scale(bytes, 1.0)
    }

    pub fn from_skel_bytes_with_scale(bytes: &[u8], scale: f32) -> Result<Arc<Self>, Error> {
        let scale = if scale.is_finite() { scale } else { 1.0 };
        let mut input = BinaryInput::new(bytes);

        let hash = match input.read_i64_be()? {
            0 => String::new(),
            value => value.to_string(),
        };

        let spine_version = input.read_string()?;
        validate_spine_version(spine_version.as_deref().unwrap_or(""))?;

        // x, y, width, height, referenceScale
        let x = input.read_f32_be()?;
        let y = input.read_f32_be()?;
        let width = input.read_f32_be()?;
        let height = input.read_f32_be()?;
        let reference_scale = input.read_f32_be()? * scale;

        let nonessential = input.read_bool()?;
        let (fps, images_path, audio_path) = if nonessential {
            (
                input.read_f32_be()?,
                input.read_string()?.unwrap_or_default(),
                input.read_string()?.unwrap_or_default(),
            )
        } else {
            (
                crate::SkeletonData::DEFAULT_FPS,
                String::new(),
                String::new(),
            )
        };

        let strings_count = input.read_varint(true)? as usize;
        let mut strings = Vec::with_capacity(strings_count);
        for _ in 0..strings_count {
            strings.push(input.read_string()?.unwrap_or_default());
        }

        // Bones
        let bones_count = input.read_varint(true)? as usize;
        let mut bones = Vec::with_capacity(bones_count);
        for i in 0..bones_count {
            let name = input.read_string()?.unwrap_or_default();
            let parent = if i == 0 {
                None
            } else {
                Some(input.read_varint(true)? as usize)
            };
            let rotation = input.read_f32_be()?;
            let x = input.read_f32_be()? * scale;
            let y = input.read_f32_be()? * scale;
            let scale_x = input.read_f32_be()?;
            let scale_y = input.read_f32_be()?;
            let shear_x = input.read_f32_be()?;
            let shear_y = input.read_f32_be()?;
            // Matches upstream binary layout: inherit (byte) comes before length (float).
            // Reading it as a varint is incorrect (it can consume extra bytes if the next float's
            // last byte has the continuation bit set).
            let inherit = map_inherit(input.read_u8()? as i32);
            let length = input.read_f32_be()? * scale;
            let skin_required = input.read_bool()?;
            let mut color = [0.61, 0.61, 0.61, 1.0];
            let mut icon = String::new();
            let mut icon_size = 1.0f32;
            let mut icon_rotation = 0.0f32;
            let mut visible = true;
            if nonessential {
                color = input.read_color_rgba()?;
                icon = input.read_string()?.unwrap_or_default();
                icon_size = input.read_f32_be()?;
                icon_rotation = input.read_f32_be()?;
                visible = input.read_bool()?;
            }
            bones.push(BoneData {
                name,
                parent,
                length,
                x,
                y,
                rotation,
                scale_x,
                scale_y,
                shear_x,
                shear_y,
                inherit,
                skin_required,
                color,
                icon,
                icon_size,
                icon_rotation,
                visible,
            });
        }

        // Slots
        let slots_count = input.read_varint(true)? as usize;
        let mut slots = Vec::with_capacity(slots_count);
        for _ in 0..slots_count {
            let name = input.read_string()?.unwrap_or_default();
            let bone = input.read_varint(true)? as usize;
            let color = input.read_color_rgba()?;

            let a = input.read_u8()?;
            let r = input.read_u8()?;
            let g = input.read_u8()?;
            let b = input.read_u8()?;
            let has_dark = !(r == 0xff && g == 0xff && b == 0xff && a == 0xff);
            let dark_color = if has_dark {
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
            } else {
                [0.0, 0.0, 0.0]
            };

            let attachment = input.read_string_ref(&strings)?;
            let blend = map_blend(input.read_varint(true)?);
            let mut visible = true;
            if nonessential {
                visible = input.read_bool()?;
            }
            slots.push(SlotData {
                name,
                bone,
                attachment,
                color,
                has_dark,
                dark_color,
                blend,
                visible,
            });
        }

        let constraint_count = input.read_varint(true)? as usize;
        let mut constraint_refs: Vec<ConstraintRef> = Vec::with_capacity(constraint_count);

        let mut ik_constraints = Vec::new();
        let mut transform_constraints = Vec::new();
        let mut path_constraints = Vec::new();
        let mut physics_constraints = Vec::new();
        let mut slider_constraints = Vec::new();

        for order in 0..constraint_count {
            let name = input.read_string()?.unwrap_or_default();
            let kind = input.read_u8()?;
            match kind {
                CONSTRAINT_IK => {
                    let bones_count = input.read_varint(true)? as usize;
                    let mut bones_in_constraint = Vec::with_capacity(bones_count);
                    for _ in 0..bones_count {
                        bones_in_constraint.push(input.read_varint(true)? as usize);
                    }
                    let target = input.read_varint(true)? as usize;
                    let flags = input.read_u8()?;
                    let skin_required = (flags & 1) != 0;
                    let scale_y_mode = if (flags & 2) != 0 {
                        crate::ScaleYMode::from_binary(input.read_u8()?)
                    } else {
                        crate::ScaleYMode::None
                    };
                    let bend_direction = if (flags & 4) != 0 { -1 } else { 1 };
                    let compress = (flags & 8) != 0;
                    let stretch = (flags & 16) != 0;
                    let mix = if (flags & 32) != 0 {
                        if (flags & 64) != 0 {
                            input.read_f32_be()?
                        } else {
                            1.0
                        }
                    } else {
                        0.0
                    };
                    let softness = if (flags & 128) != 0 {
                        input.read_f32_be()? * scale
                    } else {
                        0.0
                    };

                    let idx = ik_constraints.len();
                    ik_constraints.push(IkConstraintData {
                        name,
                        order: order as i32,
                        skin_required,
                        bones: bones_in_constraint,
                        target,
                        mix,
                        softness,
                        compress,
                        stretch,
                        scale_y_mode,
                        bend_direction,
                    });
                    constraint_refs.push(ConstraintRef::Ik(idx));
                }
                CONSTRAINT_TRANSFORM => {
                    let bones_count = input.read_varint(true)? as usize;
                    let mut bones_in_constraint = Vec::with_capacity(bones_count);
                    for _ in 0..bones_count {
                        bones_in_constraint.push(input.read_varint(true)? as usize);
                    }
                    let target = input.read_varint(true)? as usize; // `source` bone in 4.3

                    let flags = input.read_u8()?;
                    let skin_required = (flags & 1) != 0;
                    let local_source = (flags & 2) != 0;
                    let local_target = (flags & 4) != 0;
                    let additive = (flags & 8) != 0;
                    let clamp = (flags & 16) != 0;
                    let properties_count = (flags >> 5) as usize;

                    let mut properties = Vec::<crate::TransformFromProperty>::new();
                    for _ in 0..properties_count {
                        let from_kind = input.read_u8()?;
                        let from_prop = crate::TransformProperty::from_binary_kind(from_kind)
                            .ok_or_else(|| Error::BinaryParse {
                                message: format!(
                                    "transform constraint property kind out of range: {from_kind}"
                                ),
                            })?;
                        let from_scale = if matches!(from_kind, 1 | 2) {
                            scale
                        } else {
                            1.0
                        };
                        let from_offset = input.read_f32_be()? * from_scale; // from.offset

                        let to_count = input.read_u8()? as usize;
                        let mut to = Vec::<crate::TransformToProperty>::with_capacity(to_count);
                        for _ in 0..to_count {
                            let to_kind = input.read_u8()?;
                            let to_prop = crate::TransformProperty::from_binary_kind(to_kind)
                                .ok_or_else(|| Error::BinaryParse {
                                    message: format!(
                                        "transform constraint property kind out of range: {to_kind}"
                                    ),
                                })?;
                            let to_scale = if matches!(to_kind, 1 | 2) { scale } else { 1.0 };
                            let offset = input.read_f32_be()? * to_scale; // to.offset
                            let max = input.read_f32_be()? * to_scale; // to.max
                            let scale = input.read_f32_be()? * to_scale / from_scale; // to.scale
                            to.push(crate::TransformToProperty {
                                property: to_prop,
                                offset,
                                max,
                                scale,
                            });
                        }
                        if !to.is_empty() {
                            properties.push(crate::TransformFromProperty {
                                property: from_prop,
                                offset: from_offset,
                                to,
                            });
                        }
                    }

                    let mut offset_rotation = 0.0f32;
                    let mut offset_x = 0.0f32;
                    let mut offset_y = 0.0f32;
                    let mut offset_scale_x = 0.0f32;
                    let mut offset_scale_y = 0.0f32;
                    let mut offset_shear_y = 0.0f32;

                    let flags = input.read_u8()?;
                    if (flags & 1) != 0 {
                        offset_rotation = input.read_f32_be()?;
                    }
                    if (flags & 2) != 0 {
                        offset_x = input.read_f32_be()? * scale;
                    }
                    if (flags & 4) != 0 {
                        offset_y = input.read_f32_be()? * scale;
                    }
                    if (flags & 8) != 0 {
                        offset_scale_x = input.read_f32_be()?;
                    }
                    if (flags & 16) != 0 {
                        offset_scale_y = input.read_f32_be()?;
                    }
                    if (flags & 32) != 0 {
                        offset_shear_y = input.read_f32_be()?;
                    }

                    // Matches upstream: mix values default to 0 and are only present when the flag bit is set.
                    let mut mix_rotate = 0.0f32;
                    let mut mix_x = 0.0f32;
                    let mut mix_y = 0.0f32;
                    let mut mix_scale_x = 0.0f32;
                    let mut mix_scale_y = 0.0f32;
                    let mut mix_shear_y = 0.0f32;

                    let flags = input.read_u8()?;
                    if (flags & 1) != 0 {
                        mix_rotate = input.read_f32_be()?;
                    }
                    if (flags & 2) != 0 {
                        mix_x = input.read_f32_be()?;
                    }
                    if (flags & 4) != 0 {
                        mix_y = input.read_f32_be()?;
                    }
                    if (flags & 8) != 0 {
                        mix_scale_x = input.read_f32_be()?;
                    }
                    if (flags & 16) != 0 {
                        mix_scale_y = input.read_f32_be()?;
                    }
                    if (flags & 32) != 0 {
                        mix_shear_y = input.read_f32_be()?;
                    }

                    let idx = transform_constraints.len();
                    transform_constraints.push(TransformConstraintData {
                        name,
                        order: order as i32,
                        skin_required,
                        bones: bones_in_constraint,
                        source: target,
                        local_source,
                        local_target,
                        additive,
                        clamp,
                        offsets: [
                            offset_rotation,
                            offset_x,
                            offset_y,
                            offset_scale_x,
                            offset_scale_y,
                            offset_shear_y,
                        ],
                        properties,
                        mix_rotate,
                        mix_x,
                        mix_y,
                        mix_scale_x,
                        mix_scale_y,
                        mix_shear_y,
                    });
                    constraint_refs.push(ConstraintRef::Transform(idx));
                }
                CONSTRAINT_PATH => {
                    let bones_count = input.read_varint(true)? as usize;
                    let mut bones_in_constraint = Vec::with_capacity(bones_count);
                    for _ in 0..bones_count {
                        bones_in_constraint.push(input.read_varint(true)? as usize);
                    }
                    let target = input.read_varint(true)? as usize; // slot index
                    let flags = input.read_u8()?;
                    let skin_required = (flags & 1) != 0;
                    let position_mode = decode_path_constraint_position_mode(flags);
                    let spacing_mode = match (flags >> 2) & 3 {
                        0 => SpacingMode::Length,
                        1 => SpacingMode::Fixed,
                        2 => SpacingMode::Percent,
                        3 => SpacingMode::Proportional,
                        _ => SpacingMode::Length,
                    };
                    let rotate_mode = match (flags >> 4) & 3 {
                        0 => RotateMode::Tangent,
                        1 => RotateMode::Chain,
                        2 => RotateMode::ChainScale,
                        _ => RotateMode::Tangent,
                    };
                    let offset_rotation = if (flags & 128) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let mut position = input.read_f32_be()?;
                    if position_mode == PositionMode::Fixed {
                        position *= scale;
                    }
                    let mut spacing = input.read_f32_be()?;
                    if matches!(spacing_mode, SpacingMode::Length | SpacingMode::Fixed) {
                        spacing *= scale;
                    }
                    let mix_rotate = input.read_f32_be()?;
                    let mix_x = input.read_f32_be()?;
                    let mix_y = input.read_f32_be()?;

                    let idx = path_constraints.len();
                    path_constraints.push(PathConstraintData {
                        name,
                        order: order as i32,
                        bones: bones_in_constraint,
                        target,
                        position_mode,
                        spacing_mode,
                        rotate_mode,
                        offset_rotation,
                        position,
                        spacing,
                        mix_rotate,
                        mix_x,
                        mix_y,
                        skin_required,
                    });
                    constraint_refs.push(ConstraintRef::Path(idx));
                }
                CONSTRAINT_PHYSICS => {
                    let bone = input.read_varint(true)? as usize;
                    let mut flags = input.read_u8()?;
                    let skin_required = (flags & 1) != 0;
                    let x = if (flags & 2) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let y = if (flags & 4) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let rotate = if (flags & 8) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let mut scale_x = if (flags & 16) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let scale_y_mode = if scale_x < -2.0 {
                        scale_x = -2.0 - scale_x;
                        crate::ScaleYMode::Volume
                    } else if scale_x < 0.0 {
                        scale_x = -1.0 - scale_x;
                        crate::ScaleYMode::Uniform
                    } else {
                        crate::ScaleYMode::None
                    };
                    let shear_x = if (flags & 32) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let limit = if (flags & 64) != 0 {
                        input.read_f32_be()? * scale
                    } else {
                        5000.0 * scale
                    };
                    let step_div = input.read_u8()? as f32;
                    let step = if step_div > 0.0 { 1.0 / step_div } else { 1.0 };
                    let inertia = input.read_f32_be()?;
                    let strength = input.read_f32_be()?;
                    let damping = input.read_f32_be()?;
                    let mass_inverse = if (flags & 128) != 0 {
                        input.read_f32_be()?
                    } else {
                        1.0
                    };
                    let wind = input.read_f32_be()?;
                    let gravity = input.read_f32_be()?;

                    flags = input.read_u8()?;
                    let inertia_global = (flags & 1) != 0;
                    let strength_global = (flags & 2) != 0;
                    let damping_global = (flags & 4) != 0;
                    let mass_global = (flags & 8) != 0;
                    let wind_global = (flags & 16) != 0;
                    let gravity_global = (flags & 32) != 0;
                    let mix_global = (flags & 64) != 0;
                    let mix = if (flags & 128) != 0 {
                        input.read_f32_be()?
                    } else {
                        1.0
                    };

                    let idx = physics_constraints.len();
                    physics_constraints.push(crate::PhysicsConstraintData {
                        name,
                        order: order as i32,
                        skin_required,
                        bone,
                        x,
                        y,
                        rotate,
                        scale_x,
                        scale_y_mode,
                        shear_x,
                        limit,
                        step,
                        inertia,
                        strength,
                        damping,
                        mass_inverse,
                        wind,
                        gravity,
                        mix,
                        inertia_global,
                        strength_global,
                        damping_global,
                        mass_global,
                        wind_global,
                        gravity_global,
                        mix_global,
                    });
                    constraint_refs.push(ConstraintRef::Physics(idx));
                }
                CONSTRAINT_SLIDER => {
                    let flags = input.read_u8()?;
                    let skin_required = (flags & 1) != 0;
                    let looped = (flags & 2) != 0;
                    let additive = (flags & 4) != 0;

                    let setup_time = if (flags & 8) != 0 {
                        input.read_f32_be()?
                    } else {
                        0.0
                    };
                    let setup_mix = if (flags & 16) != 0 {
                        if (flags & 32) != 0 {
                            input.read_f32_be()?
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    let (bone, property, property_from, to, slider_scale, local) = if (flags & 64)
                        != 0
                    {
                        let local = (flags & 128) != 0;
                        let bone = input.read_varint(true)? as usize;
                        let from = input.read_f32_be()?;

                        let property_kind = input.read_u8()?;
                        let property = crate::TransformProperty::from_binary_kind(property_kind);
                        let property_scale = match property {
                            Some(crate::TransformProperty::X | crate::TransformProperty::Y) => {
                                scale
                            }
                            _ => 1.0,
                        };
                        let property_from = if property.is_some() {
                            from * property_scale
                        } else {
                            0.0
                        };
                        let to = input.read_f32_be()?;
                        let slider_scale = input.read_f32_be()? / property_scale;
                        (Some(bone), property, property_from, to, slider_scale, local)
                    } else {
                        (None, None, 0.0, 0.0, 0.0, false)
                    };

                    let idx = slider_constraints.len();
                    slider_constraints.push(crate::SliderConstraintData {
                        name,
                        order: order as i32,
                        skin_required,
                        setup_time,
                        setup_mix,
                        additive,
                        looped,
                        bone,
                        property,
                        property_from,
                        to,
                        scale: slider_scale,
                        local,
                        animation: None,
                    });
                    constraint_refs.push(ConstraintRef::Slider(idx));
                }
                other => {
                    return Err(Error::BinaryParse {
                        message: format!("unknown constraint type {other}"),
                    });
                }
            }
        }

        // Skins (default + named)
        let mut pending_linked_meshes = Vec::<PendingLinkedMesh>::new();
        let mut skins_map = HashMap::<String, SkinData>::new();
        let mut skin_order = Vec::<String>::new();

        // Default skin
        let trace_binary = std::env::var("SPINE2D_BINARY_TRACE")
            .ok()
            .is_some_and(|v| v == "1");
        let default_slot_count = input.read_varint(true)? as usize;
        if default_slot_count != 0 {
            let mut attachments = vec![IndexMap::new(); slots.len()];
            for _ in 0..default_slot_count {
                let slot_entry_offset = input.cursor;
                let slot_index = input.read_varint(true)? as usize;
                let count = input.read_varint(true)? as usize;
                if trace_binary {
                    eprintln!(
                        "[binary] default skin slotEntry offset={slot_entry_offset} slotIndex={slot_index} attachmentCount={count}"
                    );
                }
                for _ in 0..count {
                    let key_offset = input.cursor;
                    let key = input
                        .read_string_ref(&strings)
                        .map_err(|e| Error::BinaryParse {
                            message: format!(
                                "failed to read attachment key stringRef (skin=default slotIndex={slot_index}) at offset {key_offset}: {e}"
                            ),
                        })?
                        .unwrap_or_default();
                    let attachment_offset = input.cursor;
                    let att = read_attachment(
                        &mut input,
                        &strings,
                        nonessential,
                        scale,
                        "default",
                        slot_index,
                        &key,
                        &mut pending_linked_meshes,
                    )
                    .map_err(|e| Error::BinaryParse {
                        message: format!(
                            "failed to read attachment (skin=default slotIndex={slot_index} key={key:?}) at offset {attachment_offset}: {e}",
                        ),
                    })?;
                    if trace_binary {
                        eprintln!(
                            "  [binary] attachment key={key:?} keyOffset={key_offset} attachmentOffset={attachment_offset} endOffset={}",
                            input.cursor
                        );
                    }
                    attachments[slot_index].insert(key, att);
                }
            }
            let skin = SkinData {
                name: "default".to_string(),
                color: SkinData::DEFAULT_COLOR,
                attachments,
                bones: Vec::new(),
                ik_constraints: Vec::new(),
                transform_constraints: Vec::new(),
                path_constraints: Vec::new(),
                physics_constraints: Vec::new(),
                slider_constraints: Vec::new(),
            };
            skins_map.insert("default".to_string(), skin);
            skin_order.push("default".to_string());
        }

        let named_skins_count = input.read_varint(true)? as usize;
        for _ in 0..named_skins_count {
            let skin_name = input.read_string()?.unwrap_or_default();
            let skin_color = if nonessential {
                input.read_color_rgba()?
            } else {
                SkinData::DEFAULT_COLOR
            };
            let mut bones_in_skin = Vec::new();
            for _ in 0..(input.read_varint(true)? as usize) {
                bones_in_skin.push(input.read_varint(true)? as usize);
            }
            let mut ik_in_skin = Vec::new();
            let mut transform_in_skin = Vec::new();
            let mut path_in_skin = Vec::new();
            let mut physics_in_skin = Vec::new();
            let mut slider_in_skin = Vec::new();
            for _ in 0..(input.read_varint(true)? as usize) {
                let constraint_index = input.read_varint(true)? as usize;
                match constraint_refs.get(constraint_index) {
                    Some(ConstraintRef::Ik(i)) => ik_in_skin.push(*i),
                    Some(ConstraintRef::Transform(i)) => transform_in_skin.push(*i),
                    Some(ConstraintRef::Path(i)) => path_in_skin.push(*i),
                    Some(ConstraintRef::Physics(i)) => physics_in_skin.push(*i),
                    Some(ConstraintRef::Slider(i)) => slider_in_skin.push(*i),
                    None => {
                        return Err(Error::BinaryParse {
                            message: format!(
                                "skin references out-of-range constraint index {constraint_index} (len={})",
                                constraint_refs.len()
                            ),
                        });
                    }
                }
            }

            let slot_count = input.read_varint(true)? as usize;
            let mut attachments = vec![IndexMap::new(); slots.len()];
            for _ in 0..slot_count {
                let slot_index = input.read_varint(true)? as usize;
                let count = input.read_varint(true)? as usize;
                for _ in 0..count {
                    let key_offset = input.cursor;
                    let key = input
                        .read_string_ref(&strings)
                        .map_err(|e| Error::BinaryParse {
                            message: format!(
                                "failed to read attachment key stringRef (skin={skin_name} slotIndex={slot_index}) at offset {key_offset}: {e}"
                            ),
                        })?
                        .unwrap_or_default();
                    let attachment_offset = input.cursor;
                    let att = read_attachment(
                        &mut input,
                        &strings,
                        nonessential,
                        scale,
                        &skin_name,
                        slot_index,
                        &key,
                        &mut pending_linked_meshes,
                    )
                    .map_err(|e| Error::BinaryParse {
                        message: format!(
                            "failed to read attachment (skin={skin_name} slotIndex={slot_index} key={key:?}) at offset {attachment_offset}: {e}",
                        ),
                    })?;
                    attachments[slot_index].insert(key, att);
                }
            }

            let skin = SkinData {
                name: skin_name.clone(),
                color: skin_color,
                attachments,
                bones: bones_in_skin,
                ik_constraints: ik_in_skin,
                transform_constraints: transform_in_skin,
                path_constraints: path_in_skin,
                physics_constraints: physics_in_skin,
                slider_constraints: slider_in_skin,
            };
            skins_map.insert(skin_name.clone(), skin);
            skin_order.push(skin_name);
        }

        // Resolve linked meshes (may depend on other linked meshes).
        let mut remaining = pending_linked_meshes;
        while !remaining.is_empty() {
            let mut next = Vec::new();
            let mut resolved_any = false;

            for pending in remaining {
                let parent_skin_name = skin_order
                    .get(pending.parent_skin_index)
                    .ok_or_else(|| Error::BinaryParse {
                        message: format!(
                            "linked mesh parent skin index {} out of range (len={})",
                            pending.parent_skin_index,
                            skin_order.len()
                        ),
                    })?
                    .clone();
                let Some(parent_skin) = skins_map.get(&parent_skin_name) else {
                    return Err(Error::BinaryParse {
                        message: "linked mesh parent skin not found".to_string(),
                    });
                };
                let Some(parent_attachment) =
                    parent_skin.attachment(pending.source_slot_index, pending.parent_key.as_str())
                else {
                    return Err(Error::BinaryParse {
                        message: format!(
                            "linked mesh parent attachment not found: {}",
                            pending.parent_key
                        ),
                    });
                };
                let AttachmentData::Mesh(parent_mesh) = parent_attachment else {
                    return Err(Error::BinaryParse {
                        message: "linked mesh parent attachment is not a mesh".to_string(),
                    });
                };
                if parent_mesh.triangles.is_empty() {
                    next.push(pending);
                    continue;
                }

                let parent_vertices = parent_mesh.vertices.clone();
                let parent_uvs = parent_mesh.uvs.clone();
                let parent_triangles = parent_mesh.triangles.clone();

                let timeline_skin = if pending.inherit_timelines {
                    parent_skin_name.clone()
                } else {
                    pending.skin_name.clone()
                };
                let timeline_attachment = if pending.inherit_timelines {
                    pending.parent_key.clone()
                } else {
                    pending.attachment_key.clone()
                };

                {
                    let Some(linked_skin) = skins_map.get_mut(&pending.skin_name) else {
                        continue;
                    };
                    let Some(slot_map) = linked_skin.attachments.get_mut(pending.slot_index) else {
                        continue;
                    };
                    let Some(linked_attachment) = slot_map.get_mut(&pending.attachment_key) else {
                        continue;
                    };
                    let AttachmentData::Mesh(linked_mesh) = linked_attachment else {
                        continue;
                    };
                    linked_mesh.vertices = parent_vertices;
                    linked_mesh.uvs = parent_uvs;
                    linked_mesh.triangles = parent_triangles;
                    linked_mesh.timeline_skin = timeline_skin.clone();
                    linked_mesh.timeline_attachment = timeline_attachment.clone();
                }

                if pending.inherit_timelines && pending.slot_index != pending.source_slot_index {
                    let Some(parent_skin) = skins_map.get_mut(&timeline_skin) else {
                        continue;
                    };
                    let Some(parent_slot_map) =
                        parent_skin.attachments.get_mut(pending.source_slot_index)
                    else {
                        continue;
                    };
                    let Some(parent_attachment) = parent_slot_map.get_mut(&timeline_attachment)
                    else {
                        continue;
                    };
                    if let AttachmentData::Mesh(parent_mesh) = parent_attachment
                        && !parent_mesh.timeline_slots.contains(&pending.slot_index)
                    {
                        parent_mesh.timeline_slots.push(pending.slot_index);
                    }
                }
                resolved_any = true;
            }

            if !resolved_any && !next.is_empty() {
                let p = &next[0];
                return Err(Error::BinaryParse {
                    message: format!(
                        "linked mesh resolution stalled: skin={}, slot={}, attachment={}",
                        p.skin_name, p.slot_index, p.attachment_key
                    ),
                });
            }

            remaining = next;
        }

        // Events
        let events_count = input.read_varint(true)? as usize;
        let mut events = HashMap::<String, crate::EventData>::new();
        let mut event_defs = Vec::<crate::EventData>::with_capacity(events_count);
        for _ in 0..events_count {
            let name = input.read_string()?.unwrap_or_default();
            let int_value = input.read_varint(false)?; // intValue
            let float_value = input.read_f32_be()?; // floatValue
            let string_value = input.read_string()?.unwrap_or_default(); // stringValue
            let audio = input.read_string()?;
            let (audio_path, volume, balance) = match audio {
                Some(audio_path) if !audio_path.is_empty() => {
                    let volume = input.read_f32_be()?;
                    let balance = input.read_f32_be()?;
                    (audio_path, volume, balance)
                }
                Some(audio_path) => (audio_path, 1.0, 0.0),
                None => ("".to_string(), 1.0, 0.0),
            };
            let data = crate::EventData {
                name: name.clone(),
                int_value,
                float_value,
                string: string_value,
                audio_path,
                volume,
                balance,
            };
            event_defs.push(data.clone());
            events.insert(name.clone(), data);
        }

        // Animations
        let animations_count = input.read_varint(true)? as usize;
        let mut animations = Vec::with_capacity(animations_count);
        let mut animation_index = HashMap::<String, usize>::new();
        let trace_anim = std::env::var("SPINE2D_BINARY_TRACE_ANIM")
            .ok()
            .is_some_and(|v| v == "1");

        for ai in 0..animations_count {
            let name_offset = input.cursor;
            let name = input.read_string()?.unwrap_or_default();
            if trace_anim {
                eprintln!(
                    "[binary] animation[{ai}] name={name:?} nameOffset={name_offset} bodyOffset={}",
                    input.cursor
                );
            }
            let anim = read_animation(
                &mut input,
                &name,
                &strings,
                &skin_order,
                &skins_map,
                &slots,
                &constraint_refs,
                &path_constraints,
                &event_defs,
                scale,
                nonessential,
            )?;
            if trace_anim {
                eprintln!("[binary] animation[{ai}] endOffset={}", input.cursor);
            }
            animation_index.insert(name, ai);
            animations.push(anim);
        }

        for cref in &constraint_refs {
            if let ConstraintRef::Slider(idx) = *cref {
                let animation = input.read_varint(true)? as usize;
                if let Some(c) = slider_constraints.get_mut(idx) {
                    c.animation = Some(animation);
                }
            }
        }

        Ok(Arc::new(crate::SkeletonData {
            name: String::new(),
            spine_version,
            hash,
            x,
            y,
            width,
            height,
            reference_scale,
            fps,
            images_path,
            audio_path,
            bones,
            slots,
            skins: skins_map,
            events,
            animations,
            animation_index,
            ik_constraints,
            transform_constraints,
            path_constraints,
            physics_constraints,
            slider_constraints,
        }))
    }
}

#[allow(clippy::too_many_arguments)]
fn read_animation(
    input: &mut BinaryInput<'_>,
    name: &str,
    strings: &[String],
    skin_order: &[String],
    skins: &HashMap<String, SkinData>,
    slots: &[SlotData],
    constraint_refs: &[ConstraintRef],
    path_constraints: &[PathConstraintData],
    event_defs: &[crate::EventData],
    scale: f32,
    nonessential: bool,
) -> Result<Animation, Error> {
    let trace_anim = std::env::var("SPINE2D_BINARY_TRACE_ANIM")
        .ok()
        .is_some_and(|v| v == "1");
    if trace_anim {
        eprintln!(
            "[binary] read_animation name={name:?} startOffset={}",
            input.cursor
        );
    }
    let timeline_count_hint = input.read_varint(true)? as usize;

    let mut duration = 0.0f32;
    let mut timeline_order = TimelineOrderBuilder::with_capacity(timeline_count_hint);

    // Slot timelines
    let slot_timeline_slot_count = input.read_varint(true)? as usize;
    let mut slot_attachment_timelines = Vec::new();
    let mut slot_color_timelines = Vec::new();
    let mut slot_rgb_timelines = Vec::new();
    let mut slot_alpha_timelines = Vec::new();
    let mut slot_rgba2_timelines = Vec::new();
    let mut slot_rgb2_timelines = Vec::new();

    for _ in 0..slot_timeline_slot_count {
        let slot_index = input.read_varint(true)? as usize;
        let timeline_count = input.read_varint(true)? as usize;
        for _ in 0..timeline_count {
            let timeline_type = input.read_u8()?;
            let frame_count = input.read_varint(true)? as usize;
            let frame_last = frame_count.saturating_sub(1);
            match timeline_type {
                SLOT_ATTACHMENT => {
                    let mut frames = Vec::with_capacity(frame_count);
                    for _ in 0..frame_count {
                        let time = input.read_f32_be()?;
                        duration = duration.max(time);
                        let name = input.read_string_ref(strings)?;
                        frames.push(AttachmentFrame { time, name });
                    }
                    slot_attachment_timelines.push(AttachmentTimeline { slot_index, frames });
                    timeline_order.slot_attachment(&slot_attachment_timelines);
                }
                SLOT_RGBA => {
                    let _bezier_count = input.read_varint(true)? as usize;
                    let mut time = input.read_f32_be()?;
                    let mut r = input.read_u8()? as f32 / 255.0;
                    let mut g = input.read_u8()? as f32 / 255.0;
                    let mut b = input.read_u8()? as f32 / 255.0;
                    let mut a = input.read_u8()? as f32 / 255.0;

                    let mut frames = Vec::with_capacity(frame_count);
                    for frame in 0..frame_count {
                        duration = duration.max(time);
                        if frame == frame_last {
                            frames.push(ColorFrame {
                                time,
                                color: [r, g, b, a],
                                curve: [Curve::Linear; 4],
                            });
                            break;
                        }
                        let time2 = input.read_f32_be()?;
                        let r2 = input.read_u8()? as f32 / 255.0;
                        let g2 = input.read_u8()? as f32 / 255.0;
                        let b2 = input.read_u8()? as f32 / 255.0;
                        let a2 = input.read_u8()? as f32 / 255.0;
                        let curve_type = input.read_i8()?;
                        let curve = match curve_type {
                            CURVE_LINEAR => [Curve::Linear; 4],
                            CURVE_STEPPED => [Curve::Stepped; 4],
                            CURVE_BEZIER => [
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                            ],
                            other => {
                                return Err(Error::BinaryParse {
                                    message: format!("invalid curve type {other}"),
                                });
                            }
                        };
                        frames.push(ColorFrame {
                            time,
                            color: [r, g, b, a],
                            curve,
                        });
                        time = time2;
                        r = r2;
                        g = g2;
                        b = b2;
                        a = a2;
                    }
                    slot_color_timelines.push(ColorTimeline { slot_index, frames });
                    timeline_order.slot_color(&slot_color_timelines);
                }
                SLOT_RGB => {
                    let _bezier_count = input.read_varint(true)? as usize;
                    let mut time = input.read_f32_be()?;
                    let mut r = input.read_u8()? as f32 / 255.0;
                    let mut g = input.read_u8()? as f32 / 255.0;
                    let mut b = input.read_u8()? as f32 / 255.0;

                    let mut frames = Vec::with_capacity(frame_count);
                    for frame in 0..frame_count {
                        duration = duration.max(time);
                        if frame == frame_last {
                            frames.push(RgbFrame {
                                time,
                                color: [r, g, b],
                                curve: [Curve::Linear; 3],
                            });
                            break;
                        }
                        let time2 = input.read_f32_be()?;
                        let r2 = input.read_u8()? as f32 / 255.0;
                        let g2 = input.read_u8()? as f32 / 255.0;
                        let b2 = input.read_u8()? as f32 / 255.0;
                        let curve_type = input.read_i8()?;
                        let curve = match curve_type {
                            CURVE_LINEAR => [Curve::Linear; 3],
                            CURVE_STEPPED => [Curve::Stepped; 3],
                            CURVE_BEZIER => [
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                            ],
                            other => {
                                return Err(Error::BinaryParse {
                                    message: format!("invalid curve type {other}"),
                                });
                            }
                        };
                        frames.push(RgbFrame {
                            time,
                            color: [r, g, b],
                            curve,
                        });
                        time = time2;
                        r = r2;
                        g = g2;
                        b = b2;
                    }
                    slot_rgb_timelines.push(RgbTimeline { slot_index, frames });
                    timeline_order.slot_rgb(&slot_rgb_timelines);
                }
                SLOT_RGBA2 => {
                    let _bezier_count = input.read_varint(true)? as usize;
                    let mut time = input.read_f32_be()?;
                    let mut r = input.read_u8()? as f32 / 255.0;
                    let mut g = input.read_u8()? as f32 / 255.0;
                    let mut b = input.read_u8()? as f32 / 255.0;
                    let mut a = input.read_u8()? as f32 / 255.0;
                    let mut r2 = input.read_u8()? as f32 / 255.0;
                    let mut g2 = input.read_u8()? as f32 / 255.0;
                    let mut b2 = input.read_u8()? as f32 / 255.0;

                    let mut frames = Vec::with_capacity(frame_count);
                    for frame in 0..frame_count {
                        duration = duration.max(time);
                        if frame == frame_last {
                            frames.push(Rgba2Frame {
                                time,
                                light: [r, g, b, a],
                                dark: [r2, g2, b2],
                                curve: [Curve::Linear; 7],
                            });
                            break;
                        }
                        let time2 = input.read_f32_be()?;
                        let nr = input.read_u8()? as f32 / 255.0;
                        let ng = input.read_u8()? as f32 / 255.0;
                        let nb = input.read_u8()? as f32 / 255.0;
                        let na = input.read_u8()? as f32 / 255.0;
                        let nr2 = input.read_u8()? as f32 / 255.0;
                        let ng2 = input.read_u8()? as f32 / 255.0;
                        let nb2 = input.read_u8()? as f32 / 255.0;
                        let curve_type = input.read_i8()?;
                        let curve = match curve_type {
                            CURVE_LINEAR => [Curve::Linear; 7],
                            CURVE_STEPPED => [Curve::Stepped; 7],
                            CURVE_BEZIER => {
                                let mut curves = [Curve::Linear; 7];
                                for curve in &mut curves {
                                    *curve = read_bezier(input, 1.0)?;
                                }
                                curves
                            }
                            other => {
                                return Err(Error::BinaryParse {
                                    message: format!("invalid curve type {other}"),
                                });
                            }
                        };
                        frames.push(Rgba2Frame {
                            time,
                            light: [r, g, b, a],
                            dark: [r2, g2, b2],
                            curve,
                        });
                        time = time2;
                        r = nr;
                        g = ng;
                        b = nb;
                        a = na;
                        r2 = nr2;
                        g2 = ng2;
                        b2 = nb2;
                    }
                    slot_rgba2_timelines.push(Rgba2Timeline { slot_index, frames });
                    timeline_order.slot_rgba2(&slot_rgba2_timelines);
                }
                SLOT_RGB2 => {
                    let _bezier_count = input.read_varint(true)? as usize;
                    let mut time = input.read_f32_be()?;
                    let mut r = input.read_u8()? as f32 / 255.0;
                    let mut g = input.read_u8()? as f32 / 255.0;
                    let mut b = input.read_u8()? as f32 / 255.0;
                    let mut r2 = input.read_u8()? as f32 / 255.0;
                    let mut g2 = input.read_u8()? as f32 / 255.0;
                    let mut b2 = input.read_u8()? as f32 / 255.0;

                    let mut frames = Vec::with_capacity(frame_count);
                    for frame in 0..frame_count {
                        duration = duration.max(time);
                        if frame == frame_last {
                            frames.push(Rgb2Frame {
                                time,
                                light: [r, g, b],
                                dark: [r2, g2, b2],
                                curve: [Curve::Linear; 6],
                            });
                            break;
                        }
                        let time2 = input.read_f32_be()?;
                        let nr = input.read_u8()? as f32 / 255.0;
                        let ng = input.read_u8()? as f32 / 255.0;
                        let nb = input.read_u8()? as f32 / 255.0;
                        let nr2 = input.read_u8()? as f32 / 255.0;
                        let ng2 = input.read_u8()? as f32 / 255.0;
                        let nb2 = input.read_u8()? as f32 / 255.0;
                        let curve_type = input.read_i8()?;
                        let curve = match curve_type {
                            CURVE_LINEAR => [Curve::Linear; 6],
                            CURVE_STEPPED => [Curve::Stepped; 6],
                            CURVE_BEZIER => {
                                let mut curves = [Curve::Linear; 6];
                                for curve in &mut curves {
                                    *curve = read_bezier(input, 1.0)?;
                                }
                                curves
                            }
                            other => {
                                return Err(Error::BinaryParse {
                                    message: format!("invalid curve type {other}"),
                                });
                            }
                        };
                        frames.push(Rgb2Frame {
                            time,
                            light: [r, g, b],
                            dark: [r2, g2, b2],
                            curve,
                        });
                        time = time2;
                        r = nr;
                        g = ng;
                        b = nb;
                        r2 = nr2;
                        g2 = ng2;
                        b2 = nb2;
                    }
                    slot_rgb2_timelines.push(Rgb2Timeline { slot_index, frames });
                    timeline_order.slot_rgb2(&slot_rgb2_timelines);
                }
                SLOT_ALPHA => {
                    let _bezier_count = input.read_varint(true)? as usize;
                    let mut time = input.read_f32_be()?;
                    let mut a = input.read_u8()? as f32 / 255.0;
                    let mut frames = Vec::with_capacity(frame_count);
                    for frame in 0..frame_count {
                        duration = duration.max(time);
                        if frame == frame_last {
                            frames.push(AlphaFrame {
                                time,
                                alpha: a,
                                curve: Curve::Linear,
                            });
                            break;
                        }
                        let time2 = input.read_f32_be()?;
                        let a2 = input.read_u8()? as f32 / 255.0;
                        let curve = read_curve_1(input, 1.0)?;
                        frames.push(AlphaFrame {
                            time,
                            alpha: a,
                            curve,
                        });
                        time = time2;
                        a = a2;
                    }
                    slot_alpha_timelines.push(AlphaTimeline { slot_index, frames });
                    timeline_order.slot_alpha(&slot_alpha_timelines);
                }
                other => {
                    let type_offset = input.cursor.saturating_sub(1);
                    let preview = {
                        let bytes = input.bytes.get(type_offset..).unwrap_or(&[]);
                        let n = bytes.len().min(24);
                        let mut out = String::new();
                        for (i, b) in bytes[..n].iter().copied().enumerate() {
                            if i > 0 {
                                out.push(' ');
                            }
                            out.push_str(&format!("{b:02x}"));
                        }
                        if bytes.len() > n {
                            out.push_str(" …");
                        }
                        out
                    };
                    return Err(Error::BinaryParse {
                        message: format!(
                            "unsupported slot timeline type {other} at offset {type_offset} (anim={name} slotIndex={slot_index}); next=[{preview}]"
                        ),
                    });
                }
            }
        }
    }

    // Bone timelines
    let bone_timeline_bone_count = input.read_varint(true)? as usize;
    let mut bone_timelines = Vec::new();
    for _ in 0..bone_timeline_bone_count {
        let bone_index = input.read_varint(true)? as usize;
        let timeline_count = input.read_varint(true)? as usize;
        for _ in 0..timeline_count {
            let timeline_type = input.read_u8()?;
            let frame_count = input.read_varint(true)? as usize;
            if timeline_type == BONE_INHERIT {
                let mut frames = Vec::with_capacity(frame_count);
                for _ in 0..frame_count {
                    let time = input.read_f32_be()?;
                    duration = duration.max(time);
                    let inherit = map_inherit(input.read_u8()? as i32);
                    frames.push(InheritFrame { time, inherit });
                }
                bone_timelines.push(BoneTimeline::Inherit(InheritTimeline {
                    bone_index,
                    frames,
                }));
                timeline_order.bone(&bone_timelines);
                continue;
            }
            let _bezier_count = input.read_varint(true)? as usize;
            match timeline_type {
                BONE_ROTATE => {
                    let frames = read_rotate_timeline(input, frame_count)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines
                        .push(BoneTimeline::Rotate(RotateTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_TRANSLATE => {
                    let frames = read_curve_timeline2(input, frame_count, scale)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Translate(TranslateTimeline {
                        bone_index,
                        frames,
                    }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_TRANSLATEX => {
                    let frames = read_curve_timeline1(input, frame_count, scale)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::TranslateX(TranslateXTimeline {
                        bone_index,
                        frames,
                    }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_TRANSLATEY => {
                    let frames = read_curve_timeline1(input, frame_count, scale)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::TranslateY(TranslateYTimeline {
                        bone_index,
                        frames,
                    }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_SCALE => {
                    let frames = read_curve_timeline2(input, frame_count, 1.0)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Scale(ScaleTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_SCALEX => {
                    let frames = read_curve_timeline1(input, frame_count, 1.0)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines
                        .push(BoneTimeline::ScaleX(ScaleXTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_SCALEY => {
                    let frames = read_curve_timeline1(input, frame_count, 1.0)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines
                        .push(BoneTimeline::ScaleY(ScaleYTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_SHEAR => {
                    let frames = read_curve_timeline2(input, frame_count, 1.0)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines.push(BoneTimeline::Shear(ShearTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_SHEARX => {
                    let frames = read_curve_timeline1(input, frame_count, 1.0)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines
                        .push(BoneTimeline::ShearX(ShearXTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                BONE_SHEARY => {
                    let frames = read_curve_timeline1(input, frame_count, 1.0)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    bone_timelines
                        .push(BoneTimeline::ShearY(ShearYTimeline { bone_index, frames }));
                    timeline_order.bone(&bone_timelines);
                }
                other => {
                    return Err(Error::BinaryParse {
                        message: format!("unsupported bone timeline type {other}"),
                    });
                }
            }
        }
    }

    // IK constraint timelines
    let ik_timeline_count = input.read_varint(true)? as usize;
    let mut ik_constraint_timelines = Vec::new();
    for _ in 0..ik_timeline_count {
        let combined_index = input.read_varint(true)? as usize;
        let constraint_index = match constraint_refs.get(combined_index).copied() {
            Some(ConstraintRef::Ik(i)) => i,
            Some(other) => {
                return Err(Error::BinaryParse {
                    message: format!(
                        "ik constraint timeline index {combined_index} points to non-ik constraint: {other:?}"
                    ),
                });
            }
            None => {
                return Err(Error::BinaryParse {
                    message: format!("ik constraint timeline index {combined_index} out of range"),
                });
            }
        };
        let frame_count = input.read_varint(true)? as usize;
        let frame_last = frame_count.saturating_sub(1);
        let _bezier_count = input.read_varint(true)? as usize;

        let mut frames = Vec::with_capacity(frame_count);
        let mut flags = input.read_u8()?;
        let mut time = input.read_f32_be()?;
        let mut mix = if (flags & 1) != 0 {
            if (flags & 2) != 0 {
                input.read_f32_be()?
            } else {
                1.0
            }
        } else {
            0.0
        };
        let mut softness = if (flags & 4) != 0 {
            input.read_f32_be()? * scale
        } else {
            0.0
        };
        for frame in 0..frame_count {
            duration = duration.max(time);
            let bend_direction = if (flags & 8) != 0 { 1 } else { -1 };
            let compress = (flags & 16) != 0;
            let stretch = (flags & 32) != 0;
            if frame == frame_last {
                frames.push(IkFrame {
                    time,
                    mix,
                    softness,
                    bend_direction,
                    compress,
                    stretch,
                    curve: [Curve::Linear; 2],
                });
                break;
            }

            let next_flags = input.read_u8()?;
            let time2 = input.read_f32_be()?;
            let mix2 = if (next_flags & 1) != 0 {
                if (next_flags & 2) != 0 {
                    input.read_f32_be()?
                } else {
                    1.0
                }
            } else {
                0.0
            };
            let softness2 = if (next_flags & 4) != 0 {
                input.read_f32_be()? * scale
            } else {
                0.0
            };

            let curve = if (next_flags & 64) != 0 {
                [Curve::Stepped; 2]
            } else if (next_flags & 128) != 0 {
                [read_bezier(input, 1.0)?, read_bezier(input, scale)?]
            } else {
                [Curve::Linear; 2]
            };

            frames.push(IkFrame {
                time,
                mix,
                softness,
                bend_direction,
                compress,
                stretch,
                curve,
            });

            flags = next_flags;
            time = time2;
            mix = mix2;
            softness = softness2;
        }
        ik_constraint_timelines.push(IkConstraintTimeline {
            constraint_index,
            frames,
        });
        timeline_order.ik_constraint(&ik_constraint_timelines);
    }

    // Transform constraint timelines
    let transform_timeline_count = input.read_varint(true)? as usize;
    let mut transform_constraint_timelines = Vec::new();
    for _ in 0..transform_timeline_count {
        let combined_index = input.read_varint(true)? as usize;
        let constraint_index = match constraint_refs.get(combined_index).copied() {
            Some(ConstraintRef::Transform(i)) => i,
            Some(other) => {
                return Err(Error::BinaryParse {
                    message: format!(
                        "transform constraint timeline index {combined_index} points to non-transform constraint: {other:?}"
                    ),
                });
            }
            None => {
                return Err(Error::BinaryParse {
                    message: format!(
                        "transform constraint timeline index {combined_index} out of range"
                    ),
                });
            }
        };
        let frame_count = input.read_varint(true)? as usize;
        let frame_last = frame_count.saturating_sub(1);
        let _bezier_count = input.read_varint(true)? as usize;

        let mut frames = Vec::with_capacity(frame_count);
        let mut time = input.read_f32_be()?;
        let mut mix_rotate = input.read_f32_be()?;
        let mut mix_x = input.read_f32_be()?;
        let mut mix_y = input.read_f32_be()?;
        let mut mix_scale_x = input.read_f32_be()?;
        let mut mix_scale_y = input.read_f32_be()?;
        let mut mix_shear_y = input.read_f32_be()?;
        for frame in 0..frame_count {
            duration = duration.max(time);
            if frame == frame_last {
                frames.push(TransformFrame {
                    time,
                    mix_rotate,
                    mix_x,
                    mix_y,
                    mix_scale_x,
                    mix_scale_y,
                    mix_shear_y,
                    curve: [Curve::Linear; 6],
                });
                break;
            }
            let time2 = input.read_f32_be()?;
            let mix_rotate2 = input.read_f32_be()?;
            let mix_x2 = input.read_f32_be()?;
            let mix_y2 = input.read_f32_be()?;
            let mix_scale_x2 = input.read_f32_be()?;
            let mix_scale_y2 = input.read_f32_be()?;
            let mix_shear_y2 = input.read_f32_be()?;
            let curve_type = input.read_i8()?;
            let curve = match curve_type {
                CURVE_LINEAR => [Curve::Linear; 6],
                CURVE_STEPPED => [Curve::Stepped; 6],
                CURVE_BEZIER => {
                    let mut curves = [Curve::Linear; 6];
                    for curve in &mut curves {
                        *curve = read_bezier(input, 1.0)?;
                    }
                    curves
                }
                other => {
                    return Err(Error::BinaryParse {
                        message: format!("invalid curve type {other}"),
                    });
                }
            };
            frames.push(TransformFrame {
                time,
                mix_rotate,
                mix_x,
                mix_y,
                mix_scale_x,
                mix_scale_y,
                mix_shear_y,
                curve,
            });
            time = time2;
            mix_rotate = mix_rotate2;
            mix_x = mix_x2;
            mix_y = mix_y2;
            mix_scale_x = mix_scale_x2;
            mix_scale_y = mix_scale_y2;
            mix_shear_y = mix_shear_y2;
        }
        transform_constraint_timelines.push(TransformConstraintTimeline {
            constraint_index,
            frames,
        });
        timeline_order.transform_constraint(&transform_constraint_timelines);
    }

    // Path constraint timelines
    let path_timeline_count = input.read_varint(true)? as usize;
    let mut path_constraint_timelines = Vec::new();
    for _ in 0..path_timeline_count {
        let combined_index = input.read_varint(true)? as usize;
        let constraint_index = match constraint_refs.get(combined_index).copied() {
            Some(ConstraintRef::Path(i)) => i,
            Some(other) => {
                return Err(Error::BinaryParse {
                    message: format!(
                        "path constraint timeline index {combined_index} points to non-path constraint: {other:?}"
                    ),
                });
            }
            None => {
                return Err(Error::BinaryParse {
                    message: format!(
                        "path constraint timeline index {combined_index} out of range"
                    ),
                });
            }
        };
        let data = path_constraints
            .get(constraint_index)
            .ok_or_else(|| Error::BinaryParse {
                message: format!("path constraint index {constraint_index} out of range"),
            })?;
        let timeline_count = input.read_varint(true)? as usize;
        for _ in 0..timeline_count {
            let ty = input.read_u8()?;
            let frame_count = input.read_varint(true)? as usize;
            let frame_last = frame_count.saturating_sub(1);
            let _bezier_count = input.read_varint(true)? as usize;
            match ty {
                PATH_POSITION => {
                    let value_scale = if data.position_mode == PositionMode::Fixed {
                        scale
                    } else {
                        1.0
                    };
                    let frames = read_curve_timeline1(input, frame_count, value_scale)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    path_constraint_timelines.push(PathConstraintTimeline::Position(
                        PathConstraintPositionTimeline {
                            constraint_index,
                            frames,
                        },
                    ));
                    timeline_order.path_constraint(&path_constraint_timelines);
                }
                PATH_SPACING => {
                    let value_scale =
                        if matches!(data.spacing_mode, SpacingMode::Length | SpacingMode::Fixed) {
                            scale
                        } else {
                            1.0
                        };
                    let frames = read_curve_timeline1(input, frame_count, value_scale)?;
                    if let Some(last) = frames.last() {
                        duration = duration.max(last.time);
                    }
                    path_constraint_timelines.push(PathConstraintTimeline::Spacing(
                        PathConstraintSpacingTimeline {
                            constraint_index,
                            frames,
                        },
                    ));
                    timeline_order.path_constraint(&path_constraint_timelines);
                }
                PATH_MIX => {
                    let mut time = input.read_f32_be()?;
                    let mut mix_rotate = input.read_f32_be()?;
                    let mut mix_x = input.read_f32_be()?;
                    let mut mix_y = input.read_f32_be()?;
                    let mut frames = Vec::with_capacity(frame_count);
                    for frame in 0..frame_count {
                        duration = duration.max(time);
                        if frame == frame_last {
                            frames.push(PathMixFrame {
                                time,
                                mix_rotate,
                                mix_x,
                                mix_y,
                                curve: [Curve::Linear; 3],
                            });
                            break;
                        }
                        let time2 = input.read_f32_be()?;
                        let mix_rotate2 = input.read_f32_be()?;
                        let mix_x2 = input.read_f32_be()?;
                        let mix_y2 = input.read_f32_be()?;
                        let curve_type = input.read_i8()?;
                        let curve = match curve_type {
                            CURVE_LINEAR => [Curve::Linear; 3],
                            CURVE_STEPPED => [Curve::Stepped; 3],
                            CURVE_BEZIER => [
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                                read_bezier(input, 1.0)?,
                            ],
                            other => {
                                return Err(Error::BinaryParse {
                                    message: format!("invalid curve type {other}"),
                                });
                            }
                        };
                        frames.push(PathMixFrame {
                            time,
                            mix_rotate,
                            mix_x,
                            mix_y,
                            curve,
                        });
                        time = time2;
                        mix_rotate = mix_rotate2;
                        mix_x = mix_x2;
                        mix_y = mix_y2;
                    }
                    path_constraint_timelines.push(PathConstraintTimeline::Mix(
                        PathConstraintMixTimeline {
                            constraint_index,
                            frames,
                        },
                    ));
                    timeline_order.path_constraint(&path_constraint_timelines);
                }
                other => {
                    return Err(Error::BinaryParse {
                        message: format!("unsupported path timeline type {other}"),
                    });
                }
            }
        }
    }

    let mut physics_constraint_timelines = Vec::new();
    let mut physics_reset_timelines = Vec::new();
    let physics_timeline_count = input.read_varint(true)? as usize;
    for _ in 0..physics_timeline_count {
        // -1 means global timeline.
        let combined_index = input.read_varint(true)? - 1;
        let constraint_index = if combined_index < 0 {
            -1
        } else {
            let combined_index = combined_index as usize;
            match constraint_refs.get(combined_index).copied() {
                Some(ConstraintRef::Physics(i)) => i as i32,
                Some(other) => {
                    return Err(Error::BinaryParse {
                        message: format!(
                            "physics constraint timeline index {combined_index} points to non-physics constraint: {other:?}"
                        ),
                    });
                }
                None => {
                    return Err(Error::BinaryParse {
                        message: format!(
                            "physics constraint timeline index {combined_index} out of range"
                        ),
                    });
                }
            }
        };
        let timeline_count = input.read_varint(true)? as usize;
        for _ in 0..timeline_count {
            let ty = input.read_u8()?;
            let frame_count = input.read_varint(true)? as usize;
            if ty == PHYSICS_RESET {
                let mut frames = Vec::with_capacity(frame_count);
                for _ in 0..frame_count {
                    let time = input.read_f32_be()?;
                    duration = duration.max(time);
                    frames.push(time);
                }
                physics_reset_timelines.push(crate::PhysicsConstraintResetTimeline {
                    constraint_index,
                    frames,
                });
                timeline_order.physics_reset(&physics_reset_timelines);
                continue;
            }

            let _bezier_count = input.read_varint(true)? as usize;

            // All physics timelines are CurveTimeline1 in units (unscaled).
            let mut frames = Vec::with_capacity(frame_count);
            if frame_count == 0 {
                continue;
            }
            let mut time = input.read_f32_be()?;
            let mut value = input.read_f32_be()?;
            for frame in 0..frame_count {
                duration = duration.max(time);
                let is_last = frame + 1 == frame_count;
                if is_last {
                    frames.push(FloatFrame {
                        time,
                        value,
                        curve: Curve::Linear,
                    });
                    break;
                }

                let time2 = input.read_f32_be()?;
                let value2 = input.read_f32_be()?;
                let curve = match input.read_i8()? {
                    CURVE_LINEAR => Curve::Linear,
                    CURVE_STEPPED => Curve::Stepped,
                    CURVE_BEZIER => read_bezier(input, 1.0)?,
                    other => {
                        return Err(Error::BinaryParse {
                            message: format!("invalid curve type {other}"),
                        });
                    }
                };
                frames.push(FloatFrame { time, value, curve });
                time = time2;
                value = value2;
            }

            let timeline = crate::PhysicsConstraintFloatTimeline {
                constraint_index,
                frames,
            };
            let wrapped = match ty {
                PHYSICS_INERTIA => crate::PhysicsConstraintTimeline::Inertia(timeline),
                PHYSICS_STRENGTH => crate::PhysicsConstraintTimeline::Strength(timeline),
                PHYSICS_DAMPING => crate::PhysicsConstraintTimeline::Damping(timeline),
                PHYSICS_MASS => crate::PhysicsConstraintTimeline::Mass(timeline),
                PHYSICS_WIND => crate::PhysicsConstraintTimeline::Wind(timeline),
                PHYSICS_GRAVITY => crate::PhysicsConstraintTimeline::Gravity(timeline),
                PHYSICS_MIX => crate::PhysicsConstraintTimeline::Mix(timeline),
                other => {
                    return Err(Error::BinaryParse {
                        message: format!("unsupported physics timeline type {other}"),
                    });
                }
            };
            physics_constraint_timelines.push(wrapped);
            timeline_order.physics_constraint(&physics_constraint_timelines);
        }
    }

    let slider_timeline_count = input.read_varint(true)? as usize;
    let mut slider_time_timelines = Vec::new();
    let mut slider_mix_timelines = Vec::new();
    for _ in 0..slider_timeline_count {
        let combined_index = input.read_varint(true)? as usize;
        let constraint_index = match constraint_refs.get(combined_index).copied() {
            Some(ConstraintRef::Slider(i)) => i,
            Some(other) => {
                return Err(Error::BinaryParse {
                    message: format!(
                        "slider timeline index {combined_index} points to non-slider constraint: {other:?}"
                    ),
                });
            }
            None => {
                return Err(Error::BinaryParse {
                    message: format!("slider timeline index {combined_index} out of range"),
                });
            }
        };
        let timeline_count = input.read_varint(true)? as usize;
        for _ in 0..timeline_count {
            let ty = input.read_u8()?;
            let frame_count = input.read_varint(true)? as usize;
            let _bezier_count = input.read_varint(true)? as usize;
            let frames = read_curve_timeline1(input, frame_count, 1.0)?;
            if let Some(last) = frames.last() {
                duration = duration.max(last.time);
            }
            let timeline = crate::SliderConstraintTimeline {
                constraint_index,
                frames,
            };
            match ty {
                SLIDER_TIME => {
                    slider_time_timelines.push(timeline);
                    timeline_order.slider_time(&slider_time_timelines);
                }
                SLIDER_MIX => {
                    slider_mix_timelines.push(timeline);
                    timeline_order.slider_mix(&slider_mix_timelines);
                }
                other => {
                    return Err(Error::BinaryParse {
                        message: format!("unsupported slider timeline type {other}"),
                    });
                }
            }
        }
    }

    // Attachment timelines (deform/sequence)
    let attachment_skin_count = input.read_varint(true)? as usize;
    let mut deform_timelines = Vec::new();
    let mut sequence_timelines = Vec::new();
    for _ in 0..attachment_skin_count {
        let skin_index = input.read_varint(true)? as usize;
        let skin_name = skin_order
            .get(skin_index)
            .ok_or_else(|| Error::BinaryParse {
                message: format!("attachment timeline skin index {skin_index} out of range"),
            })?;
        let skin = skins.get(skin_name).ok_or_else(|| Error::BinaryParse {
            message: format!("skin '{skin_name}' not found"),
        })?;
        let slot_count = input.read_varint(true)? as usize;
        for _ in 0..slot_count {
            let slot_index = input.read_varint(true)? as usize;
            let attachment_count = input.read_varint(true)? as usize;
            for _ in 0..attachment_count {
                let attachment_key =
                    input
                        .read_string_ref(strings)?
                        .ok_or_else(|| Error::BinaryParse {
                            message: "missing attachment name in attachment timeline".to_string(),
                        })?;
                let attachment = skin
                    .attachment(slot_index, attachment_key.as_str())
                    .ok_or_else(|| Error::BinaryParse {
                        message: format!("attachment not found: {attachment_key}"),
                    })?;

                let timeline_type = input.read_u8()?;
                let frame_count = input.read_varint(true)? as usize;
                let frame_last = frame_count.saturating_sub(1);
                match timeline_type {
                    ATTACHMENT_DEFORM => {
                        let attachment_vertices = match attachment {
                            AttachmentData::Mesh(m) => &m.vertices,
                            AttachmentData::Path(p) => &p.vertices,
                            AttachmentData::BoundingBox(b) => &b.vertices,
                            AttachmentData::Clipping(c) => &c.vertices,
                            _ => {
                                return Err(Error::BinaryParse {
                                    message: "unsupported attachment type for deform timeline"
                                        .to_string(),
                                });
                            }
                        };
                        let (vertex_count, setup_vertices) =
                            attachment_deform_setup(attachment_vertices);
                        let _bezier_count = input.read_varint(true)? as usize;
                        let mut frames = Vec::with_capacity(frame_count);
                        let mut time = input.read_f32_be()?;
                        for frame in 0..frame_count {
                            duration = duration.max(time);
                            let end = input.read_varint(true)? as usize;
                            let vertices = if end == 0 {
                                if setup_vertices.is_some() {
                                    setup_vertices.clone().unwrap_or_default()
                                } else {
                                    vec![0.0; vertex_count]
                                }
                            } else {
                                let start = input.read_varint(true)? as usize;
                                let mut out = vec![0.0f32; vertex_count];
                                let end = start + end;
                                for o in out.iter_mut().take(end).skip(start) {
                                    *o = input.read_f32_be()? * scale;
                                }
                                if let Some(setup) = setup_vertices.as_ref() {
                                    for (o, s) in out.iter_mut().zip(setup) {
                                        *o += *s;
                                    }
                                }
                                out
                            };
                            if frame == frame_last {
                                frames.push(DeformFrame {
                                    time,
                                    vertices,
                                    curve: Curve::Linear,
                                });
                                break;
                            }
                            let time2 = input.read_f32_be()?;
                            let curve_type = input.read_i8()?;
                            let curve = match curve_type {
                                CURVE_LINEAR => Curve::Linear,
                                CURVE_STEPPED => Curve::Stepped,
                                CURVE_BEZIER => read_bezier(input, 1.0)?,
                                other => {
                                    return Err(Error::BinaryParse {
                                        message: format!("invalid curve type {other}"),
                                    });
                                }
                            };
                            frames.push(DeformFrame {
                                time,
                                vertices,
                                curve,
                            });
                            time = time2;
                        }
                        deform_timelines.push(DeformTimeline {
                            skin: skin_name.clone(),
                            slot_index,
                            attachment: attachment_key,
                            vertex_count,
                            setup_vertices,
                            frames,
                        });
                        timeline_order.deform(&deform_timelines);
                    }
                    ATTACHMENT_SEQUENCE => {
                        let mut frames = Vec::with_capacity(frame_count);
                        for _ in 0..frame_count {
                            let time = input.read_f32_be()?;
                            duration = duration.max(time);
                            let mode_and_index = input.read_i32_be()?;
                            let delay = input.read_f32_be()?;
                            let mode = map_sequence_mode(mode_and_index & 0xF)?;
                            let index = mode_and_index >> 4;
                            frames.push(SequenceFrame {
                                time,
                                mode,
                                index,
                                delay,
                            });
                        }
                        sequence_timelines.push(SequenceTimeline {
                            skin: skin_name.clone(),
                            slot_index,
                            attachment: attachment_key,
                            frames,
                        });
                        timeline_order.sequence(&sequence_timelines);
                    }
                    other => {
                        return Err(Error::BinaryParse {
                            message: format!("unsupported attachment timeline type {other}"),
                        });
                    }
                }
            }
        }
    }

    // Draw order timeline
    if trace_anim {
        eprintln!(
            "[binary] read_animation name={name:?} beforeDrawOrder offset={}",
            input.cursor
        );
    }
    let draw_order_count = input.read_varint(true)? as usize;
    if trace_anim {
        eprintln!(
            "[binary] read_animation name={name:?} drawOrderCount={draw_order_count} offset={}",
            input.cursor
        );
    }
    let draw_order_timeline = if draw_order_count == 0 {
        None
    } else {
        let mut frames = Vec::with_capacity(draw_order_count);
        for _ in 0..draw_order_count {
            let time = input.read_f32_be()?;
            duration = duration.max(time);
            let draw_order = read_draw_order_offsets(input, slots.len())?;
            frames.push(crate::DrawOrderFrame {
                time,
                draw_order_to_setup_index: draw_order,
            });
        }
        Some(crate::DrawOrderTimeline { frames })
    };
    if draw_order_timeline.is_some() {
        timeline_order.draw_order();
    }

    let draw_order_folder_count = input.read_varint(true)? as usize;
    let mut draw_order_folder_timelines = Vec::with_capacity(draw_order_folder_count);
    for _ in 0..draw_order_folder_count {
        let folder_slot_count = input.read_varint(true)? as usize;
        let mut folder_slots = Vec::with_capacity(folder_slot_count);
        for _ in 0..folder_slot_count {
            folder_slots.push(input.read_varint(true)? as usize);
        }

        let key_count = input.read_varint(true)? as usize;
        let mut frames = Vec::with_capacity(key_count);
        for _ in 0..key_count {
            let time = input.read_f32_be()?;
            duration = duration.max(time);
            let folder_draw_order = read_draw_order_offsets(input, folder_slot_count)?;
            frames.push(DrawOrderFolderFrame {
                time,
                folder_draw_order,
            });
        }
        draw_order_folder_timelines.push(DrawOrderFolderTimeline {
            slots: folder_slots,
            frames,
        });
        timeline_order.draw_order_folder(&draw_order_folder_timelines);
    }

    // Event timeline
    let event_timeline = read_event_timeline(input, event_defs, &mut duration, trace_anim, name)?;
    if nonessential {
        let _ = input.read_color_rgba()?;
    }

    let animation = Animation {
        name: name.to_string(),
        duration,
        event_timeline,
        bone_timelines,
        deform_timelines,
        sequence_timelines,
        slot_attachment_timelines,
        slot_color_timelines,
        slot_rgb_timelines,
        slot_alpha_timelines,
        slot_rgba2_timelines,
        slot_rgb2_timelines,
        ik_constraint_timelines,
        transform_constraint_timelines,
        path_constraint_timelines,
        physics_constraint_timelines,
        physics_reset_timelines,
        slider_time_timelines,
        slider_mix_timelines,
        draw_order_timeline,
        draw_order_folder_timelines,
        timeline_order: timeline_order.finish(),
    };
    Ok(crate::runtime::finalize_animation(animation))
}

fn read_event_timeline(
    input: &mut BinaryInput<'_>,
    event_defs: &[crate::EventData],
    duration: &mut f32,
    trace_anim: bool,
    animation_name: &str,
) -> Result<Option<EventTimeline>, Error> {
    if trace_anim {
        eprintln!(
            "[binary] read_animation name={animation_name:?} beforeEvents offset={}",
            input.cursor
        );
    }
    let event_count = input.read_varint(true)? as usize;
    if trace_anim {
        eprintln!(
            "[binary] read_animation name={animation_name:?} eventCount={event_count} offset={}",
            input.cursor
        );
    }
    if event_count == 0 {
        return Ok(None);
    }

    let mut events = Vec::with_capacity(event_count);
    for _ in 0..event_count {
        let time = input.read_f32_be()?;
        *duration = duration.max(time);
        let event_data_index = input.read_varint(true)? as usize;
        let event_data = event_defs
            .get(event_data_index)
            .ok_or_else(|| Error::BinaryParse {
                message: format!("event data index out of range: {event_data_index}"),
            })?;
        let int_value = input.read_varint(false)?; // intValue
        let float_value = input.read_f32_be()?; // floatValue
        // Match upstream runtimes: when the key's stringValue is null, fall back to EventData.stringValue.
        let string_value = input
            .read_string()?
            .unwrap_or_else(|| event_data.string.clone());
        let has_audio = !event_data.audio_path.is_empty();
        let (volume, balance) = if has_audio {
            (input.read_f32_be()?, input.read_f32_be()?)
        } else {
            (1.0, 0.0)
        };
        events.push(Event {
            time,
            name: event_data.name.clone(),
            int_value,
            float_value,
            string: string_value,
            audio_path: event_data.audio_path.clone(),
            volume,
            balance,
        });
    }
    Ok(Some(EventTimeline { events }))
}

fn read_draw_order_offsets(
    input: &mut BinaryInput<'_>,
    slot_count: usize,
) -> Result<Option<Vec<usize>>, Error> {
    let offset_count = input.read_varint(true)? as usize;
    if offset_count == 0 {
        return Ok(None);
    }

    let mut draw_order = vec![usize::MAX; slot_count];
    let mut unchanged = Vec::with_capacity(slot_count.saturating_sub(offset_count));
    let mut original_index = 0usize;
    for _ in 0..offset_count {
        let slot_index = input.read_varint(true)? as usize;
        while original_index != slot_index {
            if original_index >= slot_count {
                return Err(Error::BinaryParse {
                    message: "drawOrder offset slot index out of range".to_string(),
                });
            }
            unchanged.push(original_index);
            original_index += 1;
        }
        let offset = input.read_varint(true)? as isize;
        let dst = original_index as isize + offset;
        if dst < 0 || dst >= slot_count as isize {
            return Err(Error::BinaryParse {
                message: "drawOrder offset target out of range".to_string(),
            });
        }
        draw_order[dst as usize] = original_index;
        original_index += 1;
    }
    while original_index < slot_count {
        unchanged.push(original_index);
        original_index += 1;
    }
    let mut unchanged_index = unchanged.len();
    for i in (0..slot_count).rev() {
        if draw_order[i] == usize::MAX {
            unchanged_index = unchanged_index
                .checked_sub(1)
                .ok_or_else(|| Error::BinaryParse {
                    message: "drawOrder failed to fill unchanged slots".to_string(),
                })?;
            draw_order[i] = unchanged[unchanged_index];
        }
    }
    Ok(Some(draw_order))
}

fn read_rotate_timeline(
    input: &mut BinaryInput<'_>,
    frame_count: usize,
) -> Result<Vec<RotateFrame>, Error> {
    if frame_count == 0 {
        return Ok(Vec::new());
    }
    let mut frames = Vec::with_capacity(frame_count);
    let mut time = input.read_f32_be()?;
    let mut angle = input.read_f32_be()?;
    for frame in 0..frame_count {
        let curve = if frame + 1 == frame_count {
            Curve::Linear
        } else {
            let time2 = input.read_f32_be()?;
            let angle2 = input.read_f32_be()?;
            let curve = read_curve_1(input, 1.0)?;
            frames.push(RotateFrame { time, angle, curve });
            time = time2;
            angle = angle2;
            continue;
        };
        frames.push(RotateFrame { time, angle, curve });
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::{BinaryInput, read_event_timeline};
    use crate::{EventData, EventTimeline};

    fn push_varint(out: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut b = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                b |= 0x80;
            }
            out.push(b);
            if value == 0 {
                break;
            }
        }
    }

    fn push_f32_be(out: &mut Vec<u8>, v: f32) {
        out.extend_from_slice(&v.to_be_bytes());
    }

    fn push_string(out: &mut Vec<u8>, s: Option<&str>) {
        match s {
            None => push_varint(out, 0),
            Some("") => push_varint(out, 1),
            Some(s) => {
                let bytes = s.as_bytes();
                push_varint(out, (bytes.len() as u32) + 1);
                out.extend_from_slice(bytes);
            }
        }
    }

    fn read_only_event_timeline(bytes: &[u8], event_defs: &[EventData]) -> Option<EventTimeline> {
        let mut input = BinaryInput::new(bytes);
        let mut duration = 0.0f32;
        read_event_timeline(&mut input, event_defs, &mut duration, false, "<test>")
            .expect("read_event_timeline")
    }

    #[test]
    fn binary_rotate_timeline_preserves_curve_data() {
        let mut bytes = Vec::new();

        // frame 0
        push_f32_be(&mut bytes, 0.0);
        push_f32_be(&mut bytes, 10.0);
        // frame 1
        push_f32_be(&mut bytes, 0.5);
        push_f32_be(&mut bytes, 30.0);
        bytes.push(2);
        push_f32_be(&mut bytes, 0.25);
        push_f32_be(&mut bytes, 5.0);
        push_f32_be(&mut bytes, 0.75);
        push_f32_be(&mut bytes, 20.0);

        let mut input = BinaryInput::new(&bytes);
        let frames = super::read_rotate_timeline(&mut input, 2).expect("read rotate timeline");

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].time, 0.0);
        assert_eq!(frames[0].angle, 10.0);
        assert_eq!(frames[1].time, 0.5);
        assert_eq!(frames[1].angle, 30.0);
        match frames[0].curve {
            crate::Curve::Bezier { cx1, cy1, cx2, cy2 } => {
                assert_eq!(cx1, 0.25);
                assert_eq!(cy1, 5.0);
                assert_eq!(cx2, 0.75);
                assert_eq!(cy2, 20.0);
            }
            other => panic!("expected bezier curve, got {other:?}"),
        }
        assert_eq!(frames[1].curve, crate::Curve::Linear);
    }

    #[test]
    fn binary_event_timeline_null_string_falls_back_to_event_data() {
        let event_defs = vec![EventData {
            name: "evt".to_string(),
            int_value: 0,
            float_value: 0.0,
            string: "DEFAULT".to_string(),
            audio_path: String::new(),
            volume: 1.0,
            balance: 0.0,
        }];

        let mut bytes = Vec::new();

        // eventCount = 2
        push_varint(&mut bytes, 2);

        // event0: time, eventDataIndex=0, intValue=0, floatValue=0, stringValue=null
        push_f32_be(&mut bytes, 0.1);
        push_varint(&mut bytes, 0);
        push_varint(&mut bytes, 0); // zigzag-encoded 0
        push_f32_be(&mut bytes, 0.0);
        push_string(&mut bytes, None);

        // event1: same, but override stringValue
        push_f32_be(&mut bytes, 0.2);
        push_varint(&mut bytes, 0);
        push_varint(&mut bytes, 0);
        push_f32_be(&mut bytes, 0.0);
        push_string(&mut bytes, Some("OVERRIDE"));

        let timeline = read_only_event_timeline(&bytes, &event_defs).expect("timeline");
        assert_eq!(timeline.events.len(), 2);
        assert_eq!(timeline.events[0].string, "DEFAULT");
        assert_eq!(timeline.events[1].string, "OVERRIDE");
    }
}
