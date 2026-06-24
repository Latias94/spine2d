use crate::runtime::MixBlend;
use crate::{PositionMode, Skeleton, SkeletonData, UpdateCacheItem, apply_animation};
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

fn update_cache_debug_labels(skeleton: &Skeleton) -> Vec<String> {
    fn bone_name(skeleton: &Skeleton, index: usize) -> &str {
        skeleton
            .data()
            .bones
            .get(index)
            .map(|b| b.name.as_str())
            .unwrap_or("<unknown>")
    }

    skeleton
        .update_cache_items()
        .iter()
        .map(|item| match *item {
            UpdateCacheItem::Bone(index) => format!("bone {}", bone_name(skeleton, index)),
            UpdateCacheItem::Ik(index) => {
                let name = skeleton
                    .ik_constraints()
                    .get(index)
                    .and_then(|c| skeleton.data().ik_constraints.get(c.data_index()))
                    .map(|d| d.name.as_str())
                    .unwrap_or("<unknown>");
                format!("ik {}", name)
            }
            UpdateCacheItem::Transform(index) => {
                let name = skeleton
                    .transform_constraints()
                    .get(index)
                    .and_then(|c| skeleton.data().transform_constraints.get(c.data_index()))
                    .map(|d| d.name.as_str())
                    .unwrap_or("<unknown>");
                format!("transform {}", name)
            }
            UpdateCacheItem::Path(index) => {
                let name = skeleton
                    .path_constraints()
                    .get(index)
                    .and_then(|c| skeleton.data().path_constraints.get(c.data_index()))
                    .map(|d| d.name.as_str())
                    .unwrap_or("<unknown>");
                format!("path {}", name)
            }
            UpdateCacheItem::Physics(index) => {
                let name = skeleton
                    .physics_constraints()
                    .get(index)
                    .and_then(|c| skeleton.data().physics_constraints.get(c.data_index()))
                    .map(|d| d.name.as_str())
                    .unwrap_or("<unknown>");
                format!("physics {}", name)
            }
            UpdateCacheItem::Slider(index) => {
                let name = skeleton
                    .slider_constraints()
                    .get(index)
                    .and_then(|c| skeleton.data().slider_constraints.get(c.data_index()))
                    .map(|d| d.name.as_str())
                    .unwrap_or("<unknown>");
                format!("slider {}", name)
            }
        })
        .collect()
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
fn binary_nonessential_bone_and_slot_fields_are_preserved() {
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
    bytes.extend_from_slice(&[0; 8]); // hash
    push_string(&mut bytes, Some("4.3.00"));
    bytes.extend(f32_be(0.0)); // x
    bytes.extend(f32_be(0.0)); // y
    bytes.extend(f32_be(0.0)); // width
    bytes.extend(f32_be(0.0)); // height
    bytes.extend(f32_be(1.0)); // referenceScale
    bytes.push(1); // nonessential = true
    bytes.extend(f32_be(30.0)); // fps
    push_string(&mut bytes, None); // imagesPath
    push_string(&mut bytes, None); // audioPath
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
    bytes.extend(varint(0)); // animations

    let data = SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
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
}

#[test]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn binary_animation_preserves_parse_order_in_timeline_order() {
    let skel = load_example_bytes("tank/export/tank-pro.skel");
    let json = load_example_string("tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let (_, animation_skel) = data_skel.animation("shoot").expect("shoot animation");
    let (_, animation_json) = data_json.animation("shoot").expect("shoot animation");

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
    let (_, anim) = data.animation(animation_name).expect("animation exists");
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
    assert!(data.animation("run").is_some(), "missing 'run' animation");
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

    let skin_skel = data_skel.skin("default").expect("default skin (skel)");
    let skin_json = data_json.skin("default").expect("default skin (json)");

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

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_run_t0_skel_vs_json() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let a = pose_at(data_skel, "run", 0.0);
    let b = pose_at(data_json, "run", 0.0);

    for (i, (ba, bb)) in a.bones.iter().zip(&b.bones).enumerate() {
        let name = bone_name(&a, ba.data_index());
        let da = (ba.a - bb.a).abs();
        let dwx = (ba.world_x - bb.world_x).abs();
        let dwy = (ba.world_y - bb.world_y).abs();
        println!(
            "bone[{i:02}] {name:20} a {:+.6} vs {:+.6} (Δ{:.6}) wx {:+.3} vs {:+.3} (Δ{:.3}) wy {:+.3} vs {:+.3} (Δ{:.3}) rot {:+.3} vs {:+.3} (Δ{:.3})",
            ba.a,
            bb.a,
            da,
            ba.world_x,
            bb.world_x,
            dwx,
            ba.world_y,
            bb.world_y,
            dwy,
            ba.rotation,
            bb.rotation,
            (ba.rotation - bb.rotation).abs(),
        );
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_skel_vs_json_constraints() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    println!(
        "IK constraints: skel={} json={}",
        data_skel.ik_constraints.len(),
        data_json.ik_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .ik_constraints
        .iter()
        .zip(&data_json.ik_constraints)
        .enumerate()
    {
        println!(
            "ik[{i}] name skel='{}' json='{}' order {} vs {} bones {:?} vs {:?} target {} vs {} mix {:.3} vs {:.3} softness {:.3} vs {:.3} compress {} vs {} stretch {} vs {} scale_y_mode {:?} vs {:?} bend {} vs {} skin_required {} vs {}",
            a.name,
            b.name,
            a.order,
            b.order,
            a.bones,
            b.bones,
            a.target,
            b.target,
            a.mix,
            b.mix,
            a.softness,
            b.softness,
            a.compress,
            b.compress,
            a.stretch,
            b.stretch,
            a.scale_y_mode,
            b.scale_y_mode,
            a.bend_direction,
            b.bend_direction,
            a.skin_required,
            b.skin_required,
        );
    }

    println!(
        "Transform constraints: skel={} json={}",
        data_skel.transform_constraints.len(),
        data_json.transform_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .transform_constraints
        .iter()
        .zip(&data_json.transform_constraints)
        .enumerate()
    {
        println!(
            "transform[{i}] name skel='{}' json='{}' order {} vs {} bones {:?} vs {:?} source {} vs {} local_source {} vs {} local_target {} vs {} additive {} vs {} clamp {} vs {} mix_rotate {:.3} vs {:.3}",
            a.name,
            b.name,
            a.order,
            b.order,
            a.bones,
            b.bones,
            a.source,
            b.source,
            a.local_source,
            b.local_source,
            a.local_target,
            b.local_target,
            a.additive,
            b.additive,
            a.clamp,
            b.clamp,
            a.mix_rotate,
            b.mix_rotate
        );
    }

    println!(
        "Path constraints: skel={} json={}",
        data_skel.path_constraints.len(),
        data_json.path_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .path_constraints
        .iter()
        .zip(&data_json.path_constraints)
        .enumerate()
    {
        println!(
            "path[{i}] name skel='{}' json='{}' order {} vs {} bones {:?} vs {:?} target {} vs {} rotate_mode {:?} vs {:?}",
            a.name,
            b.name,
            a.order,
            b.order,
            a.bones,
            b.bones,
            a.target,
            b.target,
            a.rotate_mode,
            b.rotate_mode
        );
    }

    for anim_name in ["run", "walk"] {
        let (_, a) = data_skel.animation(anim_name).expect("skel animation");
        let (_, b) = data_json.animation(anim_name).expect("json animation");
        println!(
            "animation[{anim_name}] duration {:.3} vs {:.3}, order_len {} vs {}",
            a.duration,
            b.duration,
            a.timeline_order.len(),
            b.timeline_order.len()
        );
        println!("  skel order: {:?}", a.timeline_order);
        println!("  json order: {:?}", b.timeline_order);
    }

    println!(
        "Bones: skel={} json={}",
        data_skel.bones.len(),
        data_json.bones.len()
    );
    for (i, (a, b)) in data_skel.bones.iter().zip(&data_json.bones).enumerate() {
        if a.parent != b.parent
            || a.length != b.length
            || a.x != b.x
            || a.y != b.y
            || a.rotation != b.rotation
            || a.scale_x != b.scale_x
            || a.scale_y != b.scale_y
            || a.shear_x != b.shear_x
            || a.shear_y != b.shear_y
            || a.inherit != b.inherit
            || a.skin_required != b.skin_required
        {
            println!(
                "bone[{i}] skel='{}' json='{}' parent {:?} vs {:?} length {:.6} vs {:.6} x {:.6} vs {:.6} y {:.6} vs {:.6} rot {:.6} vs {:.6} scaleX {:.6} vs {:.6} scaleY {:.6} vs {:.6} shearX {:.6} vs {:.6} shearY {:.6} vs {:.6} inherit {:?} vs {:?} skin_required {} vs {}",
                a.name,
                b.name,
                a.parent,
                b.parent,
                a.length,
                b.length,
                a.x,
                b.x,
                a.y,
                b.y,
                a.rotation,
                b.rotation,
                a.scale_x,
                b.scale_x,
                a.scale_y,
                b.scale_y,
                a.shear_x,
                b.shear_x,
                a.shear_y,
                b.shear_y,
                a.inherit,
                b.inherit,
                a.skin_required,
                b.skin_required,
            );
        }
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_run_to_walk_t04_skel_vs_json() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let mut state_data_skel = crate::runtime::AnimationStateData::new(data_skel.clone());
    state_data_skel.set_mix("run", "walk", 0.2);
    let mut state_data_json = crate::runtime::AnimationStateData::new(data_json.clone());
    state_data_json.set_mix("run", "walk", 0.2);

    let mut skeleton_skel = Skeleton::new(data_skel.clone());
    let mut state_skel = crate::runtime::AnimationState::new(state_data_skel);
    skeleton_skel.setup_pose();
    state_skel.set_animation(0, "run", true);
    state_skel.update(0.3);
    state_skel.apply(&mut skeleton_skel);
    skeleton_skel.update_world_transform_with_physics(crate::Physics::None);

    let mut skeleton_json = Skeleton::new(data_json.clone());
    let mut state_json = crate::runtime::AnimationState::new(state_data_json);
    skeleton_json.setup_pose();
    state_json.set_animation(0, "run", true);
    state_json.update(0.3);
    state_json.apply(&mut skeleton_json);
    skeleton_json.update_world_transform_with_physics(crate::Physics::None);

    println!("--- after run only ---");
    for bone_name in ["rear-thigh", "rear-shin", "rear-foot"] {
        let i = bone_index(&data_skel, bone_name);
        let ba = &skeleton_skel.bones[i];
        let bb = &skeleton_json.bones[i];
        println!(
            "{bone_name}: arot {:.6} vs {:.6}; world_x {:.6} vs {:.6}; world_y {:.6} vs {:.6}",
            ba.arotation, bb.arotation, ba.world_x, bb.world_x, ba.world_y, bb.world_y
        );
    }

    state_json.set_animation(0, "walk", true);
    state_json.update(0.1);
    state_json.apply(&mut skeleton_json);

    state_skel.set_animation(0, "walk", true);
    state_skel.update(0.1);
    state_skel.apply(&mut skeleton_skel);

    println!("--- after walk mix apply, before world ---");
    for bone_name in ["rear-thigh", "rear-shin", "rear-foot"] {
        let i = bone_index(&data_skel, bone_name);
        let ba = &skeleton_skel.bones[i];
        let bb = &skeleton_json.bones[i];
        println!(
            "{bone_name}: local x {:.6} vs {:.6}; y {:.6} vs {:.6}; rot {:.6} vs {:.6}; ax {:.6} vs {:.6}; ay {:.6} vs {:.6}; arot {:.6} vs {:.6}; sx {:.6} vs {:.6}; sy {:.6} vs {:.6}; shx {:.6} vs {:.6}; shy {:.6} vs {:.6}",
            ba.x,
            bb.x,
            ba.y,
            bb.y,
            ba.rotation,
            bb.rotation,
            ba.ax,
            bb.ax,
            ba.ay,
            bb.ay,
            ba.arotation,
            bb.arotation,
            ba.ascale_x,
            bb.ascale_x,
            ba.ascale_y,
            bb.ascale_y,
            ba.ashear_x,
            bb.ashear_x,
            ba.ashear_y,
            bb.ashear_y,
        );
    }
    for (i, (ia, ib)) in skeleton_skel
        .ik_constraints
        .iter()
        .zip(&skeleton_json.ik_constraints)
        .enumerate()
    {
        let name = data_skel
            .ik_constraints
            .get(i)
            .map(|c| c.name.as_str())
            .unwrap_or("?");
        if matches!(name, "rear-leg-ik" | "rear-foot-ik") {
            println!(
                "{name}: mix {:.9} vs {:.9}; softness {:.9} vs {:.9}; bend {} vs {}; compress {} vs {}; stretch {} vs {}",
                ia.mix,
                ib.mix,
                ia.softness,
                ib.softness,
                ia.bend_direction,
                ib.bend_direction,
                ia.compress,
                ib.compress,
                ia.stretch,
                ib.stretch,
            );
        }
    }

    skeleton_json.update_world_transform_with_physics(crate::Physics::None);
    skeleton_skel.update_world_transform_with_physics(crate::Physics::None);

    println!("--- after walk mix ---");

    for (i, (ba, bb)) in skeleton_skel
        .bones
        .iter()
        .zip(&skeleton_json.bones)
        .enumerate()
    {
        let name = bone_name(&skeleton_skel, ba.data_index());
        let mut printed = false;
        for (label, a, b) in [
            ("ax", ba.ax, bb.ax),
            ("ay", ba.ay, bb.ay),
            ("arotation", ba.arotation, bb.arotation),
            ("ascale_x", ba.ascale_x, bb.ascale_x),
            ("ascale_y", ba.ascale_y, bb.ascale_y),
            ("ashear_x", ba.ashear_x, bb.ashear_x),
            ("ashear_y", ba.ashear_y, bb.ashear_y),
            ("a", ba.a, bb.a),
            ("b", ba.b, bb.b),
            ("c", ba.c, bb.c),
            ("d", ba.d, bb.d),
            ("world_x", ba.world_x, bb.world_x),
            ("world_y", ba.world_y, bb.world_y),
        ] {
            if (a - b).abs() > 1.0e-4 {
                if !printed {
                    println!("bone[{i}] {name}");
                    printed = true;
                }
                println!("  {label}: {:.6} vs {:.6} (Δ{:.6})", a, b, (a - b).abs());
            }
        }
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_walk_t01_rear_samples_skel_vs_json() {
    use crate::{BoneTimeline, Curve};

    fn bezier_value(
        time: f32,
        time1: f32,
        value1: f32,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        time2: f32,
        value2: f32,
    ) -> f32 {
        if time <= time1 {
            return value1;
        }
        if time >= time2 {
            return value2;
        }

        let mut low = 0.0f32;
        let mut high = 1.0f32;
        let mut t = 0.5f32;
        for _ in 0..24 {
            t = (low + high) * 0.5;
            let mt = 1.0 - t;
            let x = mt * mt * mt * time1
                + 3.0 * mt * mt * t * cx1
                + 3.0 * mt * t * t * cx2
                + t * t * t * time2;
            if x < time {
                low = t;
            } else {
                high = t;
            }
        }

        let mt = 1.0 - t;
        mt * mt * mt * value1
            + 3.0 * mt * mt * t * cy1
            + 3.0 * mt * t * t * cy2
            + t * t * t * value2
    }

    fn curve_value(
        curve: Curve,
        time: f32,
        time1: f32,
        value1: f32,
        time2: f32,
        value2: f32,
    ) -> f32 {
        match curve {
            Curve::Linear => {
                let t = (time - time1) / (time2 - time1);
                value1 + (value2 - value1) * t
            }
            Curve::Stepped => value1,
            Curve::Bezier { cx1, cy1, cx2, cy2 } => {
                bezier_value(time, time1, value1, cx1, cy1, cx2, cy2, time2, value2)
            }
        }
    }

    fn sample_rotate(frames: &[crate::RotateFrame], time: f32) -> f32 {
        let index = frames.partition_point(|f| f.time <= time);
        if index == 0 {
            return frames[0].angle;
        }
        if index >= frames.len() {
            return frames[frames.len() - 1].angle;
        }
        let prev = &frames[index - 1];
        let next = &frames[index];
        curve_value(
            prev.curve, time, prev.time, prev.angle, next.time, next.angle,
        )
    }

    fn sample_vec2(frames: &[crate::Vec2Frame], time: f32) -> (f32, f32) {
        let index = frames.partition_point(|f| f.time <= time);
        if index == 0 {
            let f = &frames[0];
            return (f.x, f.y);
        }
        if index >= frames.len() {
            let f = &frames[frames.len() - 1];
            return (f.x, f.y);
        }
        let prev = &frames[index - 1];
        let next = &frames[index];
        (
            curve_value(prev.curve[0], time, prev.time, prev.x, next.time, next.x),
            curve_value(prev.curve[1], time, prev.time, prev.y, next.time, next.y),
        )
    }

    fn sample_ik_softness(frames: &[crate::IkFrame], time: f32) -> f32 {
        let index = frames.partition_point(|f| f.time <= time);
        if index == 0 {
            return frames[0].softness;
        }
        if index >= frames.len() {
            return frames[frames.len() - 1].softness;
        }
        let prev = &frames[index - 1];
        let next = &frames[index];
        curve_value(
            prev.curve[1],
            time,
            prev.time,
            prev.softness,
            next.time,
            next.softness,
        )
    }

    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");
    for (anim_name, time) in [("walk", 0.1f32), ("run", 0.4f32)] {
        let (_, anim_skel) = data_skel.animation(anim_name).expect("skel animation");
        let (_, anim_json) = data_json.animation(anim_name).expect("json animation");

        println!(
            "[DEBUG-rear-samples] anim={anim_name} time={time:.9} bits={:08x}",
            time.to_bits()
        );
        for (i, (a, b)) in anim_skel
            .bone_timelines
            .iter()
            .zip(&anim_json.bone_timelines)
            .enumerate()
        {
            match (a, b) {
                (BoneTimeline::Rotate(ta), BoneTimeline::Rotate(tb)) => {
                    let name = &data_skel.bones[ta.bone_index].name;
                    if matches!(
                        name.as_str(),
                        "rear-foot-target"
                            | "rear-leg-target"
                            | "rear-thigh"
                            | "rear-shin"
                            | "rear-foot"
                            | "back-foot-tip"
                    ) {
                        let va = sample_rotate(&ta.frames, time);
                        let vb = sample_rotate(&tb.frames, time);
                        println!(
                            "[DEBUG-rear-samples] bone_tl[{i}] rotate {name}: sample {:.9} vs {:.9} diff {:.9}; frame_bits {:?} vs {:?}",
                            va,
                            vb,
                            va - vb,
                            ta.frames
                                .iter()
                                .map(|f| (f.time, f.time.to_bits(), f.angle))
                                .collect::<Vec<_>>(),
                            tb.frames
                                .iter()
                                .map(|f| (f.time, f.time.to_bits(), f.angle))
                                .collect::<Vec<_>>()
                        );
                    }
                }
                (BoneTimeline::Translate(ta), BoneTimeline::Translate(tb)) => {
                    let name = &data_skel.bones[ta.bone_index].name;
                    if matches!(
                        name.as_str(),
                        "rear-foot-target"
                            | "rear-leg-target"
                            | "rear-thigh"
                            | "rear-shin"
                            | "rear-foot"
                            | "back-foot-tip"
                    ) {
                        let va = sample_vec2(&ta.frames, time);
                        let vb = sample_vec2(&tb.frames, time);
                        println!(
                            "[DEBUG-rear-samples] bone_tl[{i}] translate {name}: sample ({:.9},{:.9}) vs ({:.9},{:.9}) diff ({:.9},{:.9}); frame_bits {:?} vs {:?}",
                            va.0,
                            va.1,
                            vb.0,
                            vb.1,
                            va.0 - vb.0,
                            va.1 - vb.1,
                            ta.frames
                                .iter()
                                .map(|f| (f.time, f.time.to_bits(), f.x, f.y))
                                .collect::<Vec<_>>(),
                            tb.frames
                                .iter()
                                .map(|f| (f.time, f.time.to_bits(), f.x, f.y))
                                .collect::<Vec<_>>()
                        );
                    }
                }
                _ => {}
            }
        }

        for (i, (a, b)) in anim_skel
            .ik_constraint_timelines
            .iter()
            .zip(&anim_json.ik_constraint_timelines)
            .enumerate()
        {
            let name = &data_skel.ik_constraints[a.constraint_index].name;
            if matches!(name.as_str(), "rear-leg-ik" | "rear-foot-ik" | "board-ik") {
                let va = sample_ik_softness(&a.frames, time);
                let vb = sample_ik_softness(&b.frames, time);
                println!(
                    "[DEBUG-rear-samples] ik_tl[{i}] {name}: softness {:.9} vs {:.9} diff {:.9}; frame_bits {:?} vs {:?}",
                    va,
                    vb,
                    va - vb,
                    a.frames
                        .iter()
                        .map(|f| (f.time, f.time.to_bits(), f.softness, f.bend_direction))
                        .collect::<Vec<_>>(),
                    b.frames
                        .iter()
                        .map(|f| (f.time, f.time.to_bits(), f.softness, f.bend_direction))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_run_to_walk_after_state_apply_before_world_skel_vs_json() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let mut state_data_skel = crate::runtime::AnimationStateData::new(data_skel.clone());
    state_data_skel.set_mix("run", "walk", 0.2);
    let mut state_data_json = crate::runtime::AnimationStateData::new(data_json.clone());
    state_data_json.set_mix("run", "walk", 0.2);

    let mut skeleton_skel = Skeleton::new(data_skel.clone());
    let mut state_skel = crate::runtime::AnimationState::new(state_data_skel);
    skeleton_skel.setup_pose();
    state_skel.set_animation(0, "run", true);
    state_skel.update(0.3);
    state_skel.apply(&mut skeleton_skel);
    skeleton_skel.update_world_transform_with_physics(crate::Physics::None);
    state_skel.set_animation(0, "walk", true);
    state_skel.update(0.1);
    state_skel.apply(&mut skeleton_skel);

    let mut skeleton_json = Skeleton::new(data_json.clone());
    let mut state_json = crate::runtime::AnimationState::new(state_data_json);
    skeleton_json.setup_pose();
    state_json.set_animation(0, "run", true);
    state_json.update(0.3);
    state_json.apply(&mut skeleton_json);
    skeleton_json.update_world_transform_with_physics(crate::Physics::None);
    state_json.set_animation(0, "walk", true);
    state_json.update(0.1);
    state_json.apply(&mut skeleton_json);

    for name in [
        "root",
        "hip",
        "rear-foot-target",
        "rear-leg-target",
        "rear-thigh",
        "rear-shin",
        "rear-foot",
        "back-foot-tip",
    ] {
        let i = bone_index(&data_skel, name);
        let a = &skeleton_skel.bones[i];
        let b = &skeleton_json.bones[i];
        println!(
            "[DEBUG-state-boundary] {name}: local rot {:.9} vs {:.9} diff {:.9}; x {:.9} vs {:.9} diff {:.9}; y {:.9} vs {:.9} diff {:.9}; applied rot {:.9} vs {:.9} diff {:.9}; ax {:.9} vs {:.9} diff {:.9}; ay {:.9} vs {:.9} diff {:.9}",
            a.rotation,
            b.rotation,
            a.rotation - b.rotation,
            a.x,
            b.x,
            a.x - b.x,
            a.y,
            b.y,
            a.y - b.y,
            a.arotation,
            b.arotation,
            a.arotation - b.arotation,
            a.ax,
            b.ax,
            a.ax - b.ax,
            a.ay,
            b.ay,
            a.ay - b.ay,
        );
    }
    skeleton_json.update_world_transform_with_physics(crate::Physics::None);
    skeleton_skel.update_world_transform_with_physics(crate::Physics::None);
    println!("--- after world ---");
    println!(
        "rust cache: {:?}",
        update_cache_debug_labels(&skeleton_skel)
    );
    println!(
        "json cache: {:?}",
        update_cache_debug_labels(&skeleton_json)
    );
    for name in [
        "root",
        "hip",
        "rear-foot-target",
        "rear-leg-target",
        "rear-thigh",
        "rear-shin",
        "rear-foot",
        "back-foot-tip",
    ] {
        let i = bone_index(&data_skel, name);
        let a = &skeleton_skel.bones[i];
        let b = &skeleton_json.bones[i];
        println!(
            "[DEBUG-state-boundary] {name}: local rot {:.9} vs {:.9} diff {:.9}; applied rot {:.9} vs {:.9} diff {:.9}; ax {:.9} vs {:.9} diff {:.9}; ay {:.9} vs {:.9} diff {:.9}; world a {:.9} vs {:.9} diff {:.9}; b {:.9} vs {:.9} diff {:.9}; c {:.9} vs {:.9} diff {:.9}; d {:.9} vs {:.9} diff {:.9}; wx {:.9} vs {:.9} diff {:.9}; wy {:.9} vs {:.9} diff {:.9}",
            a.rotation,
            b.rotation,
            a.rotation - b.rotation,
            a.arotation,
            b.arotation,
            a.arotation - b.arotation,
            a.ax,
            b.ax,
            a.ax - b.ax,
            a.ay,
            b.ay,
            a.ay - b.ay,
            a.a,
            b.a,
            a.a - b.a,
            a.b,
            b.b,
            a.b - b.b,
            a.c,
            b.c,
            a.c - b.c,
            a.d,
            b.d,
            a.d - b.d,
            a.world_x,
            b.world_x,
            a.world_x - b.world_x,
            a.world_y,
            b.world_y,
            a.world_y - b.world_y,
        );
    }
    for name in ["board-ik", "rear-leg-ik", "rear-foot-ik"] {
        let i = data_skel
            .ik_constraints
            .iter()
            .position(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing ik {name}"));
        let a = &skeleton_skel.ik_constraints[i];
        let b = &skeleton_json.ik_constraints[i];
        println!(
            "[DEBUG-state-boundary] ik {name}: mix {:.9} vs {:.9}; softness {:.9} vs {:.9}; bend {} vs {}",
            a.mix, b.mix, a.softness, b.softness, a.bend_direction, b.bend_direction
        );
    }
}

fn bone_index(data: &SkeletonData, name: &str) -> usize {
    data.bones
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| panic!("missing bone: {name}"))
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_spineboy_walk_animation_skel_vs_json() {
    let skel = load_example_bytes("spineboy/export/spineboy-pro.skel");
    let json = load_example_string("spineboy/export/spineboy-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let (_, walk_skel) = data_skel.animation("walk").expect("walk skel");
    let (_, walk_json) = data_json.animation("walk").expect("walk json");

    println!(
        "walk duration: {:.6} vs {:.6}; bone {} vs {}; deform {} vs {}; sequence {} vs {}; slotAttachment {} vs {}; ik {} vs {}; transform {} vs {}; path {} vs {}; physics {} vs {}; sliderTime {} vs {}; sliderMix {} vs {}",
        walk_skel.duration,
        walk_json.duration,
        walk_skel.bone_timelines.len(),
        walk_json.bone_timelines.len(),
        walk_skel.deform_timelines.len(),
        walk_json.deform_timelines.len(),
        walk_skel.sequence_timelines.len(),
        walk_json.sequence_timelines.len(),
        walk_skel.slot_attachment_timelines.len(),
        walk_json.slot_attachment_timelines.len(),
        walk_skel.ik_constraint_timelines.len(),
        walk_json.ik_constraint_timelines.len(),
        walk_skel.transform_constraint_timelines.len(),
        walk_json.transform_constraint_timelines.len(),
        walk_skel.path_constraint_timelines.len(),
        walk_json.path_constraint_timelines.len(),
        walk_skel.physics_constraint_timelines.len(),
        walk_json.physics_constraint_timelines.len(),
        walk_skel.slider_time_timelines.len(),
        walk_json.slider_time_timelines.len(),
        walk_skel.slider_mix_timelines.len(),
        walk_json.slider_mix_timelines.len(),
    );

    for (i, (a, b)) in walk_skel
        .bone_timelines
        .iter()
        .zip(&walk_json.bone_timelines)
        .enumerate()
    {
        if format!("{a:#?}") != format!("{b:#?}") {
            println!("bone timeline[{i}] skel = {a:#?}");
            println!("bone timeline[{i}] json = {b:#?}");
        }
    }

    for (i, (a, b)) in walk_skel
        .ik_constraint_timelines
        .iter()
        .zip(&walk_json.ik_constraint_timelines)
        .enumerate()
    {
        let name_skel = data_skel
            .ik_constraints
            .get(a.constraint_index)
            .map(|c| c.name.as_str())
            .unwrap_or("<unknown>");
        if matches!(name_skel, "rear-leg-ik" | "rear-foot-ik") {
            let name_json = data_json
                .ik_constraints
                .get(b.constraint_index)
                .map(|c| c.name.as_str())
                .unwrap_or("<unknown>");
            println!("ik timeline[{i}] skel name={name_skel} json name={name_json}");
            println!("ik timeline[{i}] skel = {a:#?}");
            println!("ik timeline[{i}] json = {b:#?}");
        }
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_tank_skel_vs_json_constraints() {
    let skel = load_example_bytes("tank/export/tank-pro.skel");
    let json = load_example_string("tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    println!(
        "IK constraints: skel={} json={}",
        data_skel.ik_constraints.len(),
        data_json.ik_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .ik_constraints
        .iter()
        .zip(&data_json.ik_constraints)
        .enumerate()
    {
        println!(
            "ik[{i}] name skel='{}' json='{}' mix {:.3} vs {:.3} bend {} vs {} target {} vs {} bones {:?} vs {:?}",
            a.name,
            b.name,
            a.mix,
            b.mix,
            a.bend_direction,
            b.bend_direction,
            a.target,
            b.target,
            a.bones,
            b.bones
        );
    }

    println!(
        "Transform constraints: skel={} json={}",
        data_skel.transform_constraints.len(),
        data_json.transform_constraints.len()
    );
    for (i, (a, b)) in data_skel
        .transform_constraints
        .iter()
        .zip(&data_json.transform_constraints)
        .enumerate()
    {
        println!(
            "transform[{i}] name skel='{}' json='{}' mix_rotate {:.3} vs {:.3} mix_x {:.3} vs {:.3} mix_y {:.3} vs {:.3}",
            a.name, b.name, a.mix_rotate, b.mix_rotate, a.mix_x, b.mix_x, a.mix_y, b.mix_y
        );
    }
}

#[test]
#[ignore]
#[cfg(all(feature = "json", feature = "upstream-smoke"))]
fn debug_dump_tank_shoot_t01_skel_vs_json() {
    let skel = load_example_bytes("tank/export/tank-pro.skel");
    let json = load_example_string("tank/export/tank-pro.json");

    let data_skel = SkeletonData::from_skel_bytes(&skel).expect("parse skel");
    let data_json = SkeletonData::from_json_str(&json).expect("parse json");

    let t = 0.1;
    let a = pose_at(data_skel, "shoot", t);
    let b = pose_at(data_json, "shoot", t);

    for (i, (ba, bb)) in a.bones.iter().zip(&b.bones).enumerate() {
        let name = bone_name(&a, ba.data_index());
        let dwx = (ba.world_x - bb.world_x).abs();
        let dwy = (ba.world_y - bb.world_y).abs();
        if dwx > 0.01 || dwy > 0.01 {
            println!(
                "bone[{i:02}] {name:16} wx {:+.5} vs {:+.5} (Δ{:.5}) wy {:+.5} vs {:+.5} (Δ{:.5}) a {:+.6} vs {:+.6} (Δ{:.6})",
                ba.world_x,
                bb.world_x,
                dwx,
                ba.world_y,
                bb.world_y,
                dwy,
                ba.a,
                bb.a,
                (ba.a - bb.a).abs(),
            );
        }
    }
}
