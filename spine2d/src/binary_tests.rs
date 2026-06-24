use crate::runtime::MixBlend;
use crate::{PositionMode, Skeleton, SkeletonData, apply_animation};
use std::path::PathBuf;
use std::sync::Arc;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn load_bytes(rel: &str) -> Vec<u8> {
    std::fs::read(repo_root().join(rel)).expect(rel)
}

#[cfg(feature = "upstream-smoke")]
fn upstream_examples_root() -> PathBuf {
    if let Ok(dir) = std::env::var("SPINE2D_UPSTREAM_EXAMPLES_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }

    let root = repo_root();
    let candidates = [
        root.join(".cache/spine-runtimes/examples"),
        root.join("assets/spine-runtimes/examples"),
        root.join("third_party/spine-runtimes/examples"),
    ];
    for p in candidates {
        if p.is_dir() {
            return p;
        }
    }

    panic!(
        "Upstream Spine examples not found. Run `python3 ./scripts/fetch_spine_runtimes_examples.py --mode export --scope tests` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

#[cfg(feature = "upstream-smoke")]
fn load_example_bytes(rel: &str) -> Vec<u8> {
    let path = upstream_examples_root().join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn binary_path_constraint_position_mode_uses_latest_flag_bits() {
    assert_eq!(
        crate::binary::decode_path_constraint_position_mode_for_test(0b0000_0010),
        PositionMode::Percent
    );
    assert_eq!(
        crate::binary::decode_path_constraint_position_mode_for_test(0b0000_0100),
        PositionMode::Fixed
    );
}

#[test]
fn binary_weighted_vertices_reads_latest_packed_bone_length() {
    fn varint(value: i32) -> Vec<u8> {
        let mut value = value as u32;
        let mut out = Vec::new();
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
        out
    }

    fn f32_be(value: f32) -> [u8; 4] {
        value.to_be_bytes()
    }

    let mut bytes = Vec::new();
    bytes.extend(varint(2)); // vertexCount
    bytes.extend(varint(5)); // bones array length: [2, 0, 1, 1, 2]
    bytes.extend(varint(2)); // vertex 0 has two bones
    bytes.extend(varint(0));
    bytes.extend(f32_be(1.0));
    bytes.extend(f32_be(2.0));
    bytes.extend(f32_be(0.25));
    bytes.extend(varint(1));
    bytes.extend(f32_be(3.0));
    bytes.extend(f32_be(4.0));
    bytes.extend(f32_be(0.75));
    bytes.extend(varint(1)); // vertex 1 has one bone
    bytes.extend(varint(2));
    bytes.extend(f32_be(5.0));
    bytes.extend(f32_be(6.0));
    bytes.extend(f32_be(1.0));

    let (vertices, world_vertices_length, cursor) =
        crate::binary::read_vertices_for_test(&bytes, true, 2.0).expect("read vertices");

    assert_eq!(world_vertices_length, 4);
    assert_eq!(cursor, bytes.len());
    let crate::MeshVertices::Weighted(weights) = vertices else {
        panic!("expected weighted vertices");
    };
    assert_eq!(weights.len(), 2);
    assert_eq!(weights[0].len(), 2);
    assert_eq!(weights[0][0].bone, 0);
    assert_approx(weights[0][0].x, 2.0, 1.0e-6, "v0 w0 x");
    assert_approx(weights[0][0].y, 4.0, 1.0e-6, "v0 w0 y");
    assert_approx(weights[0][0].weight, 0.25, 1.0e-6, "v0 w0 weight");
    assert_eq!(weights[1][0].bone, 2);
    assert_approx(weights[1][0].x, 10.0, 1.0e-6, "v1 w0 x");
    assert_approx(weights[1][0].y, 12.0, 1.0e-6, "v1 w0 y");
    assert_approx(weights[1][0].weight, 1.0, 1.0e-6, "v1 w0 weight");
}

#[test]
fn binary_nonessential_bone_slot_and_animation_fields_are_preserved() {
    fn varint(value: i32) -> Vec<u8> {
        let mut value = value as u32;
        let mut out = Vec::new();
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
        out
    }

    fn f32_be(value: f32) -> [u8; 4] {
        value.to_be_bytes()
    }

    fn push_string(out: &mut Vec<u8>, s: Option<&str>) {
        match s {
            None => out.push(0),
            Some("") => out.push(1),
            Some(s) => {
                out.push((s.len() + 1) as u8);
                out.extend_from_slice(s.as_bytes());
            }
        }
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&42_i64.to_be_bytes()); // hash
    push_string(&mut bytes, Some("4.3.00"));
    bytes.extend(f32_be(1.0)); // x
    bytes.extend(f32_be(2.0)); // y
    bytes.extend(f32_be(3.0)); // width
    bytes.extend(f32_be(4.0)); // height
    bytes.extend(f32_be(50.0)); // referenceScale
    bytes.push(1); // nonessential = true
    bytes.extend(f32_be(24.0)); // fps
    push_string(&mut bytes, Some("images/")); // imagesPath
    push_string(&mut bytes, Some("audio/")); // audioPath
    bytes.extend(varint(0)); // strings
    bytes.extend(varint(1)); // bones
    push_string(&mut bytes, Some("root"));
    bytes.extend(f32_be(0.0)); // rotation
    bytes.extend(f32_be(0.0)); // x
    bytes.extend(f32_be(0.0)); // y
    bytes.extend(f32_be(1.0)); // scaleX
    bytes.extend(f32_be(1.0)); // scaleY
    bytes.extend(f32_be(0.0)); // shearX
    bytes.extend(f32_be(0.0)); // shearY
    bytes.push(0); // inherit normal
    bytes.extend(f32_be(0.0)); // length
    bytes.push(0); // skinRequired
    bytes.extend_from_slice(&[0x11, 0x22, 0x33, 0x44]); // color
    push_string(&mut bytes, Some("root-icon"));
    bytes.extend(f32_be(2.5)); // iconSize
    bytes.extend(f32_be(45.0)); // iconRotation
    bytes.push(0); // visible=false

    bytes.extend(varint(1)); // slots
    push_string(&mut bytes, Some("slot0"));
    bytes.extend(varint(0)); // bone
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // slot color
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // slot dark
    push_string(&mut bytes, None); // attachment
    bytes.extend(varint(0)); // blend normal
    bytes.push(0); // nonessential visible=false? false => 0

    bytes.extend(varint(0)); // constraints
    bytes.extend(varint(0)); // default skins
    bytes.extend(varint(0)); // named skins
    bytes.extend(varint(0)); // events
    bytes.extend(varint(1)); // animations
    push_string(&mut bytes, Some("anim"));
    bytes.extend(varint(0)); // timeline count hint
    bytes.extend(varint(0)); // slot timelines
    bytes.extend(varint(0)); // bone timelines
    bytes.extend(varint(0)); // ik timelines
    bytes.extend(varint(0)); // transform timelines
    bytes.extend(varint(0)); // path timelines
    bytes.extend(varint(0)); // physics timelines
    bytes.extend(varint(0)); // slider timelines
    bytes.extend(varint(0)); // attachment timelines
    bytes.extend(varint(0)); // draw order timeline
    bytes.extend(varint(0)); // draw order folder timelines
    bytes.extend(varint(0)); // event timeline
    bytes.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]); // animation color

    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
    assert_eq!(data.name, "");
    assert_eq!(data.spine_version.as_deref(), Some("4.3.00"));
    assert_eq!(data.hash, "42");
    assert_eq!(data.x, 1.0);
    assert_eq!(data.y, 2.0);
    assert_eq!(data.width, 3.0);
    assert_eq!(data.height, 4.0);
    assert_eq!(data.reference_scale, 50.0);
    assert_eq!(data.fps, 24.0);
    assert_eq!(data.images_path, "images/");
    assert_eq!(data.audio_path, "audio/");

    let bone = &data.bones[0];
    assert_eq!(
        bone.color,
        [
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            0x44 as f32 / 255.0
        ]
    );
    assert_eq!(bone.icon, "root-icon");
    assert_eq!(bone.icon_size, 2.5);
    assert_eq!(bone.icon_rotation, 45.0);
    assert!(!bone.visible);
    assert!(!data.slots[0].visible);

    let animation = data.find_animation("anim").unwrap();
    assert_eq!(
        animation.color,
        [
            0xaa as f32 / 255.0,
            0xbb as f32 / 255.0,
            0xcc as f32 / 255.0,
            0xdd as f32 / 255.0
        ]
    );
}

#[test]
fn binary_nonessential_attachment_colors_are_preserved() {
    fn varint(value: i32) -> Vec<u8> {
        let mut value = value as u32;
        let mut out = Vec::new();
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
        out
    }

    fn f32_be(value: f32) -> [u8; 4] {
        value.to_be_bytes()
    }

    fn push_string(out: &mut Vec<u8>, s: Option<&str>) {
        match s {
            None => out.push(0),
            Some("") => out.push(1),
            Some(s) => {
                out.push((s.len() + 1) as u8);
                out.extend_from_slice(s.as_bytes());
            }
        }
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&42_i64.to_be_bytes()); // hash
    push_string(&mut bytes, Some("4.3.00"));
    bytes.extend(f32_be(1.0)); // x
    bytes.extend(f32_be(2.0)); // y
    bytes.extend(f32_be(3.0)); // width
    bytes.extend(f32_be(4.0)); // height
    bytes.extend(f32_be(50.0)); // referenceScale
    bytes.push(1); // nonessential = true
    bytes.extend(f32_be(24.0)); // fps
    push_string(&mut bytes, Some("images/")); // imagesPath
    push_string(&mut bytes, Some("audio/")); // audioPath
    bytes.extend(varint(4)); // strings
    push_string(&mut bytes, Some("bbox"));
    push_string(&mut bytes, Some("path"));
    push_string(&mut bytes, Some("point"));
    push_string(&mut bytes, Some("clip"));

    bytes.extend(varint(1)); // bones
    push_string(&mut bytes, Some("root"));
    bytes.extend(f32_be(0.0)); // rotation
    bytes.extend(f32_be(0.0)); // x
    bytes.extend(f32_be(0.0)); // y
    bytes.extend(f32_be(1.0)); // scaleX
    bytes.extend(f32_be(1.0)); // scaleY
    bytes.extend(f32_be(0.0)); // shearX
    bytes.extend(f32_be(0.0)); // shearY
    bytes.push(0); // inherit normal
    bytes.extend(f32_be(0.0)); // length
    bytes.push(0); // skinRequired
    bytes.extend_from_slice(&[0x11, 0x22, 0x33, 0x44]); // bone color
    push_string(&mut bytes, Some("root-icon"));
    bytes.extend(f32_be(2.5)); // iconSize
    bytes.extend(f32_be(45.0)); // iconRotation
    bytes.push(0); // visible=false

    bytes.extend(varint(1)); // slots
    push_string(&mut bytes, Some("slot0"));
    bytes.extend(varint(0)); // bone
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // slot color
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]); // slot dark
    push_string(&mut bytes, None); // attachment
    bytes.extend(varint(0)); // blend normal
    bytes.push(0); // visible=false

    bytes.extend(varint(0)); // constraints
    bytes.extend(varint(1)); // default skin slot entries
    bytes.extend(varint(0)); // slot index
    bytes.extend(varint(4)); // attachment count

    bytes.extend(varint(1)); // bbox key ref
    bytes.push(1); // boundingbox
    bytes.extend(varint(0)); // vertex count
    bytes.extend_from_slice(&[0x10, 0x20, 0x30, 0x40]);

    bytes.extend(varint(2)); // path key ref
    bytes.push(4); // path
    bytes.extend(varint(0)); // vertex count
    bytes.extend_from_slice(&[0x11, 0x21, 0x31, 0x41]);

    bytes.extend(varint(3)); // point key ref
    bytes.push(5); // point
    bytes.extend(f32_be(12.5)); // rotation
    bytes.extend(f32_be(1.5)); // x
    bytes.extend(f32_be(2.5)); // y
    bytes.extend_from_slice(&[0x12, 0x22, 0x32, 0x42]);

    bytes.extend(varint(4)); // clip key ref
    bytes.push(6); // clipping
    bytes.extend(varint(0)); // end slot index
    bytes.extend(varint(0)); // vertex count
    bytes.extend_from_slice(&[0x13, 0x23, 0x33, 0x43]);

    bytes.extend(varint(0)); // named skins
    bytes.extend(varint(0)); // events
    bytes.extend(varint(0)); // animations

    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
    let skin = data.find_skin("default").expect("default skin");
    let attachments = &skin.attachments[0];

    match attachments.get("bbox").unwrap() {
        crate::AttachmentData::BoundingBox(bb) => assert_eq!(
            bb.color,
            [
                0x10 as f32 / 255.0,
                0x20 as f32 / 255.0,
                0x30 as f32 / 255.0,
                0x40 as f32 / 255.0
            ]
        ),
        other => panic!("expected bounding box, got {other:?}"),
    }
    match attachments.get("path").unwrap() {
        crate::AttachmentData::Path(path) => assert_eq!(
            path.color,
            [
                0x11 as f32 / 255.0,
                0x21 as f32 / 255.0,
                0x31 as f32 / 255.0,
                0x41 as f32 / 255.0
            ]
        ),
        other => panic!("expected path, got {other:?}"),
    }
    match attachments.get("point").unwrap() {
        crate::AttachmentData::Point(point) => assert_eq!(
            point.color,
            [
                0x12 as f32 / 255.0,
                0x22 as f32 / 255.0,
                0x32 as f32 / 255.0,
                0x42 as f32 / 255.0
            ]
        ),
        other => panic!("expected point, got {other:?}"),
    }
    match attachments.get("clip").unwrap() {
        crate::AttachmentData::Clipping(clip) => assert_eq!(
            clip.color,
            [
                0x13 as f32 / 255.0,
                0x23 as f32 / 255.0,
                0x33 as f32 / 255.0,
                0x43 as f32 / 255.0
            ]
        ),
        other => panic!("expected clipping, got {other:?}"),
    }
}

#[test]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn binary_animation_preserves_parse_order_in_timeline_order() {
    let skel = load_example_bytes("tank/export/tank-pro.skel");
    let json = load_example_string("tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let animation_skel = data_skel.find_animation("shoot").expect("shoot animation");
    let animation_json = data_json.find_animation("shoot").expect("shoot animation");

    assert_eq!(
        animation_skel.timeline_order, animation_json.timeline_order,
        "binary and json should preserve the same parse order"
    );
}

#[cfg(feature = "json")]
fn load_string(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).expect(rel)
}

#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn load_example_string(rel: &str) -> String {
    let path = upstream_examples_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn pose_at(data: Arc<SkeletonData>, animation_name: &str, time: f32) -> Skeleton {
    let anim = data
        .find_animation(animation_name)
        .expect("animation exists");
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, time, true, 1.0, MixBlend::Replace);
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    skeleton
}

#[cfg(all(feature = "json", feature = "binary"))]
fn slot_index(data: &SkeletonData, name: &str) -> usize {
    data.slots
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("missing slot {name:?}"))
}

fn assert_approx(a: f32, b: f32, eps: f32, ctx: &str) {
    if (a - b).abs() > eps {
        panic!("{ctx}: expected {b}, got {a} (diff {})", (a - b).abs());
    }
}

#[cfg(feature = "json")]
fn bone_name(s: &Skeleton, data_index: usize) -> &str {
    s.data
        .bones
        .get(data_index)
        .map(|d| d.name.as_str())
        .unwrap_or("?")
}

#[cfg(feature = "json")]
fn assert_pose_close(a: &Skeleton, b: &Skeleton, eps: f32, ctx: &str) {
    assert_eq!(a.bones.len(), b.bones.len(), "bones length");
    assert_eq!(a.slots.len(), b.slots.len(), "slots length");
    assert_eq!(a.draw_order, b.draw_order, "draw order");

    for (i, (ba, bb)) in a.bones.iter().zip(&b.bones).enumerate() {
        let name_a = bone_name(a, ba.data_index());
        let name_b = bone_name(b, bb.data_index());
        assert_eq!(
            ba.data_index(),
            bb.data_index(),
            "{ctx}: bone[{i}] data_index"
        );
        assert_eq!(name_a, name_b, "{ctx}: bone[{i}] name");
        assert_eq!(ba.active, bb.active, "{ctx}: bone[{i}]({name_a}).active");
        assert_eq!(ba.inherit, bb.inherit, "{ctx}: bone[{i}]({name_a}).inherit");
        assert_eq!(
            ba.parent_index(),
            bb.parent_index(),
            "{ctx}: bone[{i}]({name_a}).parent_index"
        );

        assert_approx(ba.x, bb.x, eps, &format!("{ctx}: bone[{i}]({name_a}).x"));
        assert_approx(ba.y, bb.y, eps, &format!("{ctx}: bone[{i}]({name_a}).y"));
        assert_approx(
            ba.rotation,
            bb.rotation,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).rotation"),
        );
        assert_approx(
            ba.scale_x,
            bb.scale_x,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).scale_x"),
        );
        assert_approx(
            ba.scale_y,
            bb.scale_y,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).scale_y"),
        );
        assert_approx(
            ba.shear_x,
            bb.shear_x,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).shear_x"),
        );
        assert_approx(
            ba.shear_y,
            bb.shear_y,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).shear_y"),
        );
        assert_approx(ba.a, bb.a, eps, &format!("{ctx}: bone[{i}]({name_a}).a"));
        assert_approx(ba.b, bb.b, eps, &format!("{ctx}: bone[{i}]({name_a}).b"));
        assert_approx(ba.c, bb.c, eps, &format!("{ctx}: bone[{i}]({name_a}).c"));
        assert_approx(ba.d, bb.d, eps, &format!("{ctx}: bone[{i}]({name_a}).d"));
        assert_approx(
            ba.world_x,
            bb.world_x,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).world_x"),
        );
        assert_approx(
            ba.world_y,
            bb.world_y,
            eps,
            &format!("{ctx}: bone[{i}]({name_a}).world_y"),
        );
    }

    for (i, (sa, sb)) in a.slots.iter().zip(&b.slots).enumerate() {
        assert_eq!(sa.attachment, sb.attachment, "slot[{i}].attachment");
        assert_eq!(
            sa.sequence_index, sb.sequence_index,
            "slot[{i}].sequence_index"
        );
        assert_eq!(sa.deform.len(), sb.deform.len(), "slot[{i}].deform.len");
        for (j, (&da, &db)) in sa.deform.iter().zip(&sb.deform).enumerate() {
            assert_approx(da, db, eps, &format!("slot[{i}].deform[{j}]"));
        }
        for k in 0..4 {
            assert_approx(
                sa.color[k],
                sb.color[k],
                eps,
                &format!("slot[{i}].color[{k}]"),
            );
        }
        assert_eq!(sa.has_dark, sb.has_dark, "slot[{i}].has_dark");
        for k in 0..3 {
            assert_approx(
                sa.dark_color[k],
                sb.dark_color[k],
                eps,
                &format!("slot[{i}].dark_color[{k}]"),
            );
        }
    }

    assert_eq!(
        a.ik_constraints.len(),
        b.ik_constraints.len(),
        "ik constraints length"
    );
    for (i, (ca, cb)) in a.ik_constraints.iter().zip(&b.ik_constraints).enumerate() {
        assert_approx(ca.mix, cb.mix, eps, &format!("ik[{i}].mix"));
        assert_approx(ca.softness, cb.softness, eps, &format!("ik[{i}].softness"));
        // `.skel` and `.json` exports may differ in `bend_direction` for single-bone IK
        // constraints (it is ignored by the solver). Only enforce it for two-bone IK.
        if ca.bones.len() == 2 || cb.bones.len() == 2 {
            assert_eq!(
                ca.bend_direction, cb.bend_direction,
                "ik[{i}].bend_direction"
            );
        }
    }

    assert_eq!(
        a.transform_constraints.len(),
        b.transform_constraints.len(),
        "transform constraints length"
    );
    for (i, (ca, cb)) in a
        .transform_constraints
        .iter()
        .zip(&b.transform_constraints)
        .enumerate()
    {
        assert_approx(
            ca.mix_rotate,
            cb.mix_rotate,
            eps,
            &format!("transform[{i}].mix_rotate"),
        );
        assert_approx(ca.mix_x, cb.mix_x, eps, &format!("transform[{i}].mix_x"));
        assert_approx(ca.mix_y, cb.mix_y, eps, &format!("transform[{i}].mix_y"));
        assert_approx(
            ca.mix_scale_x,
            cb.mix_scale_x,
            eps,
            &format!("transform[{i}].mix_scale_x"),
        );
        assert_approx(
            ca.mix_scale_y,
            cb.mix_scale_y,
            eps,
            &format!("transform[{i}].mix_scale_y"),
        );
        assert_approx(
            ca.mix_shear_y,
            cb.mix_shear_y,
            eps,
            &format!("transform[{i}].mix_shear_y"),
        );
    }

    assert_eq!(
        a.path_constraints.len(),
        b.path_constraints.len(),
        "path constraints length"
    );
    for (i, (ca, cb)) in a
        .path_constraints
        .iter()
        .zip(&b.path_constraints)
        .enumerate()
    {
        assert_approx(
            ca.position,
            cb.position,
            eps,
            &format!("path[{i}].position"),
        );
        assert_approx(ca.spacing, cb.spacing, eps, &format!("path[{i}].spacing"));
        assert_approx(
            ca.mix_rotate,
            cb.mix_rotate,
            eps,
            &format!("path[{i}].mix_rotate"),
        );
        assert_approx(ca.mix_x, cb.mix_x, eps, &format!("path[{i}].mix_x"));
        assert_approx(ca.mix_y, cb.mix_y, eps, &format!("path[{i}].mix_y"));
    }
}

#[test]
#[cfg(feature = "upstream-smoke")]
fn skel_smoke_loads_spineboy_pro() {
    let bytes = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
    assert!(
        data.find_animation("run").is_some(),
        "missing 'run' animation"
    );
    let _ = pose_at(data, "run", 0.2);
}

#[test]
#[cfg(feature = "upstream-smoke")]
fn skel_spineboy_constraints_match_spine_cpp_lite_reference() {
    // Expected values are dumped from the official C++ runtime (oracle) loading
    // `spineboy-pro.skel` (see `scripts/run_spine_cpp_lite_dump_constraints.zsh`).
    let bytes = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");

    let ik = |name: &str| {
        data.ik_constraints
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing ik constraint {name:?}"))
    };
    let tr = |name: &str| {
        data.transform_constraints
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing transform constraint {name:?}"))
    };

    assert_approx(ik("aim-ik").mix, 0.0, 1.0e-6, "aim-ik mix");
    assert_eq!(ik("aim-ik").bend_direction, -1, "aim-ik bend");

    assert_approx(ik("aim-torso-ik").mix, 1.0, 1.0e-6, "aim-torso-ik mix");
    assert_eq!(ik("aim-torso-ik").bend_direction, -1, "aim-torso-ik bend");

    assert_approx(ik("front-leg-ik").mix, 1.0, 1.0e-6, "front-leg-ik mix");
    assert_eq!(ik("front-leg-ik").bend_direction, -1, "front-leg-ik bend");

    assert_approx(ik("rear-leg-ik").mix, 1.0, 1.0e-6, "rear-leg-ik mix");
    assert_eq!(ik("rear-leg-ik").bend_direction, -1, "rear-leg-ik bend");
    assert_approx(ik("rear-foot-ik").mix, 1.0, 1.0e-6, "rear-foot-ik mix");
    assert_eq!(ik("rear-foot-ik").bend_direction, -1, "rear-foot-ik bend");

    assert_approx(
        tr("aim-front-arm-transform").mix_rotate,
        0.0,
        1.0e-6,
        "aim-front-arm-transform mix_rotate",
    );
    assert_approx(
        tr("aim-front-arm-transform").mix_x,
        0.0,
        1.0e-6,
        "aim-front-arm-transform mix_x",
    );
    assert_approx(
        tr("aim-front-arm-transform").mix_y,
        0.0,
        1.0e-6,
        "aim-front-arm-transform mix_y",
    );

    assert_approx(
        tr("shoulder").mix_rotate,
        0.0,
        1.0e-6,
        "shoulder mix_rotate",
    );
    assert_approx(tr("shoulder").mix_x, -1.0, 1.0e-6, "shoulder mix_x");
    assert_approx(tr("shoulder").mix_y, -1.0, 1.0e-6, "shoulder mix_y");
}

#[test]
#[cfg(all(feature = "json", feature = "binary", feature = "upstream-smoke"))]
fn skel_spineboy_ik_constraints_match_json_parse() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    assert_eq!(
        data_skel.ik_constraints.len(),
        data_json.ik_constraints.len(),
        "ik constraints length",
    );

    for (i, (a, b)) in data_skel
        .ik_constraints
        .iter()
        .zip(&data_json.ik_constraints)
        .enumerate()
    {
        assert_eq!(a.name, b.name, "ik[{i}].name");
        assert_eq!(a.order, b.order, "ik[{i}].order");
        assert_eq!(a.skin_required, b.skin_required, "ik[{i}].skin_required");
        assert_eq!(a.bones, b.bones, "ik[{i}].bones");
        assert_eq!(a.target, b.target, "ik[{i}].target");
        assert_eq!(a.compress, b.compress, "ik[{i}].compress");
        assert_eq!(a.stretch, b.stretch, "ik[{i}].stretch");
        assert_eq!(a.scale_y_mode, b.scale_y_mode, "ik[{i}].scale_y_mode");
        if a.bones.len() > 1 {
            assert_eq!(a.bend_direction, b.bend_direction, "ik[{i}].bend_direction");
        }
        assert_approx(a.mix, b.mix, 1.0e-6, &format!("ik[{i}].mix"));
        assert_approx(a.softness, b.softness, 1.0e-6, &format!("ik[{i}].softness"));
    }
}

#[test]
#[cfg(all(feature = "json", feature = "binary", feature = "upstream-smoke"))]
fn skel_tank_treads_path_attachment_matches_json() {
    let skel = load_example_bytes("tank/export/tank-pro.skel");
    let json = load_example_string("tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let slot_name = "treads-path";
    let slot_skel = slot_index(&data_skel, slot_name);
    let slot_json = slot_index(&data_json, slot_name);
    assert_eq!(slot_skel, slot_json, "slot index");

    let skin_skel = data_skel.find_skin("default").expect("default skin (skel)");
    let skin_json = data_json.find_skin("default").expect("default skin (json)");

    let att_skel = skin_skel
        .attachments
        .get(slot_skel)
        .and_then(|m| m.get(slot_name))
        .unwrap_or_else(|| panic!("missing {slot_name:?} attachment in skel default skin"));
    let att_json = skin_json
        .attachments
        .get(slot_json)
        .and_then(|m| m.get(slot_name))
        .unwrap_or_else(|| panic!("missing {slot_name:?} attachment in json default skin"));

    let (p_skel, p_json) = match (att_skel, att_json) {
        (crate::AttachmentData::Path(a), crate::AttachmentData::Path(b)) => (a, b),
        _ => panic!("treads-path attachment must be Path"),
    };

    assert_eq!(p_skel.closed, p_json.closed, "closed");
    assert_eq!(
        p_skel.constant_speed, p_json.constant_speed,
        "constant_speed"
    );
    assert_eq!(p_skel.lengths.len(), p_json.lengths.len(), "lengths.len");
    for (i, (&a, &b)) in p_skel.lengths.iter().zip(&p_json.lengths).enumerate() {
        assert_approx(a, b, 1.0e-3, &format!("lengths[{i}]"));
    }
}

#[test]
#[cfg(all(feature = "json", feature = "binary", feature = "upstream-smoke"))]
fn skel_tank_treads_path_constraint_matches_json() {
    let skel = load_bytes("assets/spine-runtimes/examples/tank/export/tank-pro.skel");
    let json = load_string("assets/spine-runtimes/examples/tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let path_skel = data_skel
        .path_constraints
        .iter()
        .find(|c| c.name == "treads-path")
        .expect("treads-path constraint (skel)");
    let path_json = data_json
        .path_constraints
        .iter()
        .find(|c| c.name == "treads-path")
        .expect("treads-path constraint (json)");

    assert_eq!(path_skel.position_mode, path_json.position_mode);
    assert_eq!(path_skel.spacing_mode, path_json.spacing_mode);
    assert_eq!(path_skel.rotate_mode, path_json.rotate_mode);
    assert_eq!(
        path_skel.position_mode,
        crate::PositionMode::Percent,
        "binary path flags must decode percent position mode"
    );
    assert_approx(path_skel.position, path_json.position, 1.0e-6, "position");
    assert_approx(path_skel.spacing, path_json.spacing, 1.0e-6, "spacing");
    assert_approx(
        path_skel.mix_rotate,
        path_json.mix_rotate,
        1.0e-6,
        "mix_rotate",
    );
    assert_approx(path_skel.mix_x, path_json.mix_x, 1.0e-6, "mix_x");
    assert_approx(path_skel.mix_y, path_json.mix_y, 1.0e-6, "mix_y");
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn skel_matches_json_pose_spineboy_run() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    for &t in &[0.0, 0.1, 0.2, 0.4, 0.6] {
        let a = pose_at(data_skel.clone(), "run", t);
        let b = pose_at(data_json.clone(), "run", t);
        // `.skel` stores binary `f32`s while JSON stores decimals; small export/parse drift is
        // expected and can accumulate through constraints.
        assert_pose_close(&a, &b, 2.5e-1, &format!("spineboy.run t={t}"));
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn skel_matches_json_pose_tank_shoot() {
    let skel = load_example_bytes("tank/export/tank-pro.skel");
    let json = load_example_string("tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    for &t in &[0.1, 0.3, 0.5] {
        let a = pose_at(data_skel.clone(), "shoot", t);
        let b = pose_at(data_json.clone(), "shoot", t);
        assert_pose_close(&a, &b, 2.5e-1, &format!("tank.shoot t={t}"));
    }
}
