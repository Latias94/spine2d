use crate::runtime::{AnimationState, AnimationStateData};
use crate::{Skeleton, SkeletonData};
use serde::Deserialize;
use serde::de::Deserializer;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn upstream_examples_root() -> PathBuf {
    if let Ok(dir) = std::env::var("SPINE2D_UPSTREAM_EXAMPLES_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../assets/spine-runtimes/examples"),
        manifest_dir.join("../third_party/spine-runtimes/examples"),
        manifest_dir.join("../.cache/spine-runtimes/examples"),
    ];
    for p in candidates {
        if p.is_dir() {
            return p;
        }
    }

    panic!(
        "Upstream Spine examples not found. Run `./scripts/import_spine_runtimes_examples.zsh --mode json` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn example_json_path(relative: &str) -> PathBuf {
    let candidate = upstream_examples_root().join(relative);
    if candidate.is_file() {
        return candidate;
    }

    let cache = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.cache/spine-runtimes/examples")
        .join(relative);
    if cache.is_file() {
        return cache;
    }

    candidate
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/oracle_scenarios")
        .join(name)
}

fn golden_skel_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/oracle_scenarios_skel")
        .join(name)
}

fn assert_approx(label: &str, actual: f32, expected: f32, abs_eps: f32) {
    let diff = (actual - expected).abs();
    let rel_eps = 2e-7_f32;
    let tol = abs_eps + rel_eps * expected.abs().max(actual.abs());
    assert!(
        diff <= tol,
        "{label}: expected {expected}, got {actual} (diff {diff}, abs_eps {abs_eps}, rel_eps {rel_eps}, tol {tol})"
    );
}

#[derive(Clone, Debug, Deserialize)]
struct WorldDump {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    #[serde(rename = "x")]
    x: f32,
    #[serde(rename = "y")]
    y: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct AppliedDump {
    #[serde(rename = "x")]
    x: f32,
    #[serde(rename = "y")]
    y: f32,
    rotation: f32,
    #[serde(rename = "scaleX")]
    scale_x: f32,
    #[serde(rename = "scaleY")]
    scale_y: f32,
    #[serde(rename = "shearX")]
    shear_x: f32,
    #[serde(rename = "shearY")]
    shear_y: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct BoneDump {
    name: String,
    active: i32,
    world: WorldDump,
    applied: AppliedDump,
}

#[derive(Clone, Debug, Deserialize)]
struct AttachmentDump {
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SlotDump {
    name: String,
    color: [f32; 4],
    #[serde(rename = "hasDark")]
    has_dark: i32,
    #[serde(rename = "darkColor")]
    dark_color: [f32; 4],
    #[serde(default = "default_sequence_index", rename = "sequenceIndex")]
    sequence_index: i32,
    attachment: Option<AttachmentDump>,
}

fn default_sequence_index() -> i32 {
    -1
}

#[derive(Clone, Debug, Deserialize)]
struct IkConstraintDump {
    name: String,
    mix: f32,
    softness: f32,
    #[serde(rename = "bendDirection")]
    bend_direction: f32,
    active: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct TransformConstraintDump {
    name: String,
    #[serde(rename = "mixRotate")]
    mix_rotate: f32,
    #[serde(rename = "mixX")]
    mix_x: f32,
    #[serde(rename = "mixY")]
    mix_y: f32,
    #[serde(rename = "mixScaleX")]
    mix_scale_x: f32,
    #[serde(rename = "mixScaleY")]
    mix_scale_y: f32,
    #[serde(rename = "mixShearY")]
    mix_shear_y: f32,
    active: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct PathConstraintDump {
    name: String,
    position: f32,
    spacing: f32,
    #[serde(rename = "mixRotate")]
    mix_rotate: f32,
    #[serde(rename = "mixX")]
    mix_x: f32,
    #[serde(rename = "mixY")]
    mix_y: f32,
    active: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct PhysicsConstraintDump {
    name: String,
    inertia: f32,
    strength: f32,
    damping: f32,
    #[serde(rename = "massInverse")]
    mass_inverse: f32,
    wind: f32,
    gravity: f32,
    mix: f32,
    reset: i32,
    ux: f32,
    uy: f32,
    cx: f32,
    cy: f32,
    tx: f32,
    ty: f32,
    #[serde(rename = "xOffset")]
    x_offset: f32,
    #[serde(rename = "xVelocity")]
    x_velocity: f32,
    #[serde(rename = "yOffset")]
    y_offset: f32,
    #[serde(rename = "yVelocity")]
    y_velocity: f32,
    #[serde(rename = "rotateOffset")]
    rotate_offset: f32,
    #[serde(rename = "rotateVelocity")]
    rotate_velocity: f32,
    #[serde(rename = "scaleOffset")]
    scale_offset: f32,
    #[serde(rename = "scaleVelocity")]
    scale_velocity: f32,
    remaining: f32,
    #[serde(rename = "lastTime")]
    last_time: f32,
    active: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct DebugDump {
    slot: String,
    #[serde(
        rename = "worldVertices",
        deserialize_with = "deserialize_world_vertices_or_null"
    )]
    world_vertices: Vec<f32>,
}

fn deserialize_world_vertices_or_null<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Vec<f32>>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize)]
struct PoseDump {
    bones: Vec<BoneDump>,
    slots: Vec<SlotDump>,
    #[serde(rename = "drawOrder")]
    draw_order: Vec<i32>,
    #[serde(default, rename = "ikConstraints")]
    ik_constraints: Vec<IkConstraintDump>,
    #[serde(default, rename = "transformConstraints")]
    transform_constraints: Vec<TransformConstraintDump>,
    #[serde(default, rename = "pathConstraints")]
    path_constraints: Vec<PathConstraintDump>,
    #[serde(default, rename = "physicsConstraints")]
    physics_constraints: Vec<PhysicsConstraintDump>,
    debug: Option<DebugDump>,
}

fn read_pose(path: &Path) -> PoseDump {
    let json = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse json {path:?}: {e}"))
}

fn slot_index(data: &SkeletonData, name: &str) -> usize {
    data.slots
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("missing slot: {name}"))
}

fn dump_pose(skeleton: &Skeleton, _time: f32, debug_slot: Option<&str>) -> PoseDump {
    let bones = skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(i, bone)| {
            let name = skeleton
                .data
                .bones
                .get(i)
                .map(|b| b.name.as_str())
                .unwrap_or("<unknown>");
            BoneDump {
                name: name.to_string(),
                active: if bone.active { 1 } else { 0 },
                world: WorldDump {
                    a: bone.a,
                    b: bone.b,
                    c: bone.c,
                    d: bone.d,
                    x: bone.world_x,
                    y: bone.world_y,
                },
                applied: AppliedDump {
                    x: bone.ax,
                    y: bone.ay,
                    rotation: bone.arotation,
                    scale_x: bone.ascale_x,
                    scale_y: bone.ascale_y,
                    shear_x: bone.ashear_x,
                    shear_y: bone.ashear_y,
                },
            }
        })
        .collect();

    let slots = skeleton
        .slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let name = skeleton
                .data
                .slots
                .get(i)
                .map(|s| s.name.as_str())
                .unwrap_or("<unknown>");
            let attachment = slot
                .get_applied_attachment(skeleton)
                .map(|a| AttachmentDump {
                    name: a.get_name().to_string(),
                });
            let dark = slot.applied_dark_color();
            let dark_color = if slot.applied_has_dark_color() {
                [dark[0], dark[1], dark[2], 1.0]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            SlotDump {
                name: name.to_string(),
                color: slot.applied_color(),
                has_dark: if slot.applied_has_dark_color() { 1 } else { 0 },
                dark_color,
                sequence_index: slot.applied_sequence_index(),
                attachment,
            }
        })
        .collect();

    let draw_order = skeleton
        .draw_order()
        .iter()
        .copied()
        .map(|i| i as i32)
        .collect();

    let ik_constraints = skeleton
        .ik_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .ik_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            IkConstraintDump {
                name: name.to_string(),
                mix: c.mix,
                softness: c.softness,
                bend_direction: c.bend_direction as f32,
                active: if c.active { 1.0 } else { 0.0 },
            }
        })
        .collect();

    let transform_constraints = skeleton
        .transform_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .transform_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            TransformConstraintDump {
                name: name.to_string(),
                mix_rotate: c.mix_rotate,
                mix_x: c.mix_x,
                mix_y: c.mix_y,
                mix_scale_x: c.mix_scale_x,
                mix_scale_y: c.mix_scale_y,
                mix_shear_y: c.mix_shear_y,
                active: if c.active { 1.0 } else { 0.0 },
            }
        })
        .collect();

    let path_constraints = skeleton
        .path_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .path_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            PathConstraintDump {
                name: name.to_string(),
                position: c.position,
                spacing: c.spacing,
                mix_rotate: c.mix_rotate,
                mix_x: c.mix_x,
                mix_y: c.mix_y,
                active: if c.active { 1.0 } else { 0.0 },
            }
        })
        .collect();

    let physics_constraints = skeleton
        .physics_constraints
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data
                .physics_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            PhysicsConstraintDump {
                name: name.to_string(),
                inertia: c.inertia,
                strength: c.strength,
                damping: c.damping,
                mass_inverse: c.mass_inverse,
                wind: c.wind,
                gravity: c.gravity,
                mix: c.mix,
                reset: if c.reset { 1 } else { 0 },
                ux: c.ux,
                uy: c.uy,
                cx: c.cx,
                cy: c.cy,
                tx: c.tx,
                ty: c.ty,
                x_offset: c.x_offset,
                x_velocity: c.x_velocity,
                y_offset: c.y_offset,
                y_velocity: c.y_velocity,
                rotate_offset: c.rotate_offset,
                rotate_velocity: c.rotate_velocity,
                scale_offset: c.scale_offset,
                scale_velocity: c.scale_velocity,
                remaining: c.remaining,
                last_time: c.last_time,
                active: if c.active { 1 } else { 0 },
            }
        })
        .collect();

    let debug = debug_slot.map(|slot_name| {
        let i = slot_index(&skeleton.data, slot_name);
        DebugDump {
            slot: slot_name.to_string(),
            world_vertices: skeleton
                .slots
                .get(i)
                .and_then(|slot| slot.get_applied_attachment(skeleton))
                .and_then(|attachment| attachment.compute_world_vertices(skeleton, i))
                .unwrap_or_default(),
        }
    });

    PoseDump {
        bones,
        slots,
        draw_order,
        ik_constraints,
        transform_constraints,
        path_constraints,
        physics_constraints,
        debug,
    }
}

fn assert_pose_parity(rust: &PoseDump, cpp: &PoseDump, eps: f32) {
    let rust_bones: HashMap<_, _> = rust.bones.iter().map(|b| (b.name.as_str(), b)).collect();
    let cpp_bones: HashMap<_, _> = cpp.bones.iter().map(|b| (b.name.as_str(), b)).collect();
    let mut bone_names: BTreeSet<&str> = BTreeSet::new();
    bone_names.extend(rust_bones.keys().copied());
    bone_names.extend(cpp_bones.keys().copied());

    let missing_bones: Vec<_> = bone_names
        .iter()
        .copied()
        .filter(|n| !rust_bones.contains_key(n) || !cpp_bones.contains_key(n))
        .collect();
    assert!(missing_bones.is_empty(), "missing bones: {missing_bones:?}");

    for name in bone_names {
        let r = rust_bones[name];
        let c = cpp_bones[name];
        assert_eq!(r.active, c.active, "bone {name} active mismatch");
        if r.active == 0 && c.active == 0 {
            continue;
        }
        assert_approx(&format!("bone {name} world.a"), r.world.a, c.world.a, eps);
        assert_approx(&format!("bone {name} world.b"), r.world.b, c.world.b, eps);
        assert_approx(&format!("bone {name} world.c"), r.world.c, c.world.c, eps);
        assert_approx(&format!("bone {name} world.d"), r.world.d, c.world.d, eps);
        assert_approx(&format!("bone {name} world.x"), r.world.x, c.world.x, eps);
        assert_approx(&format!("bone {name} world.y"), r.world.y, c.world.y, eps);

        assert_approx(
            &format!("bone {name} applied.x"),
            r.applied.x,
            c.applied.x,
            eps,
        );
        assert_approx(
            &format!("bone {name} applied.y"),
            r.applied.y,
            c.applied.y,
            eps,
        );
        assert_approx(
            &format!("bone {name} applied.rotation"),
            r.applied.rotation,
            c.applied.rotation,
            eps,
        );
        assert_approx(
            &format!("bone {name} applied.scaleX"),
            r.applied.scale_x,
            c.applied.scale_x,
            eps,
        );
        assert_approx(
            &format!("bone {name} applied.scaleY"),
            r.applied.scale_y,
            c.applied.scale_y,
            eps,
        );
        assert_approx(
            &format!("bone {name} applied.shearX"),
            r.applied.shear_x,
            c.applied.shear_x,
            eps,
        );
        assert_approx(
            &format!("bone {name} applied.shearY"),
            r.applied.shear_y,
            c.applied.shear_y,
            eps,
        );
    }

    let rust_slots: HashMap<_, _> = rust.slots.iter().map(|s| (s.name.as_str(), s)).collect();
    let cpp_slots: HashMap<_, _> = cpp.slots.iter().map(|s| (s.name.as_str(), s)).collect();
    let mut slot_names: BTreeSet<&str> = BTreeSet::new();
    slot_names.extend(rust_slots.keys().copied());
    slot_names.extend(cpp_slots.keys().copied());

    let missing_slots: Vec<_> = slot_names
        .iter()
        .copied()
        .filter(|n| !rust_slots.contains_key(n) || !cpp_slots.contains_key(n))
        .collect();
    assert!(missing_slots.is_empty(), "missing slots: {missing_slots:?}");

    for name in slot_names {
        let r = rust_slots[name];
        let c = cpp_slots[name];
        for (i, label) in ["r", "g", "b", "a"].into_iter().enumerate() {
            assert_approx(
                &format!("slot {name} color.{label}"),
                r.color[i],
                c.color[i],
                eps,
            );
            assert_approx(
                &format!("slot {name} darkColor.{label}"),
                r.dark_color[i],
                c.dark_color[i],
                eps,
            );
        }
        assert_eq!(r.has_dark, c.has_dark, "slot {name} hasDark mismatch");
        assert_eq!(
            r.sequence_index, c.sequence_index,
            "slot {name} sequenceIndex mismatch"
        );

        let ra = r.attachment.as_ref().map(|a| a.name.as_str());
        let ca = c.attachment.as_ref().map(|a| a.name.as_str());
        assert_eq!(ra, ca, "slot {name} attachment mismatch");
    }

    assert_eq!(rust.draw_order, cpp.draw_order, "drawOrder mismatch");

    let rust_ik: HashMap<_, _> = rust
        .ik_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let cpp_ik: HashMap<_, _> = cpp
        .ik_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    for name in rust_ik.keys() {
        let r = rust_ik[name];
        let c = cpp_ik
            .get(name)
            .unwrap_or_else(|| panic!("missing ik constraint in cpp: {name}"));
        assert_approx(&format!("ik {name} mix"), r.mix, c.mix, eps);
        assert_approx(&format!("ik {name} softness"), r.softness, c.softness, eps);
        assert_approx(
            &format!("ik {name} bendDirection"),
            r.bend_direction,
            c.bend_direction,
            eps,
        );
        assert_approx(&format!("ik {name} active"), r.active, c.active, eps);
    }

    let rust_tc: HashMap<_, _> = rust
        .transform_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let cpp_tc: HashMap<_, _> = cpp
        .transform_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    for name in rust_tc.keys() {
        let r = rust_tc[name];
        let c = cpp_tc
            .get(name)
            .unwrap_or_else(|| panic!("missing transform constraint in cpp: {name}"));
        assert_approx(
            &format!("transform {name} mixRotate"),
            r.mix_rotate,
            c.mix_rotate,
            eps,
        );
        assert_approx(&format!("transform {name} mixX"), r.mix_x, c.mix_x, eps);
        assert_approx(&format!("transform {name} mixY"), r.mix_y, c.mix_y, eps);
        assert_approx(
            &format!("transform {name} mixScaleX"),
            r.mix_scale_x,
            c.mix_scale_x,
            eps,
        );
        assert_approx(
            &format!("transform {name} mixScaleY"),
            r.mix_scale_y,
            c.mix_scale_y,
            eps,
        );
        assert_approx(
            &format!("transform {name} mixShearY"),
            r.mix_shear_y,
            c.mix_shear_y,
            eps,
        );
        assert_approx(&format!("transform {name} active"), r.active, c.active, eps);
    }

    let rust_pc: HashMap<_, _> = rust
        .path_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let cpp_pc: HashMap<_, _> = cpp
        .path_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    for name in rust_pc.keys() {
        let r = rust_pc[name];
        let c = cpp_pc
            .get(name)
            .unwrap_or_else(|| panic!("missing path constraint in cpp: {name}"));
        assert_approx(
            &format!("path {name} position"),
            r.position,
            c.position,
            eps,
        );
        assert_approx(&format!("path {name} spacing"), r.spacing, c.spacing, eps);
        assert_approx(
            &format!("path {name} mixRotate"),
            r.mix_rotate,
            c.mix_rotate,
            eps,
        );
        assert_approx(&format!("path {name} mixX"), r.mix_x, c.mix_x, eps);
        assert_approx(&format!("path {name} mixY"), r.mix_y, c.mix_y, eps);
        assert_approx(&format!("path {name} active"), r.active, c.active, eps);
    }

    let rust_phys: HashMap<_, _> = rust
        .physics_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let cpp_phys: HashMap<_, _> = cpp
        .physics_constraints
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut phys_names: BTreeSet<&str> = BTreeSet::new();
    phys_names.extend(rust_phys.keys().copied());
    phys_names.extend(cpp_phys.keys().copied());

    let missing_phys: Vec<_> = phys_names
        .iter()
        .copied()
        .filter(|n| !rust_phys.contains_key(n) || !cpp_phys.contains_key(n))
        .collect();
    assert!(
        missing_phys.is_empty(),
        "missing physics constraints: {missing_phys:?}"
    );

    for name in phys_names {
        let r = rust_phys[name];
        let c = cpp_phys[name];
        assert_eq!(r.active, c.active, "physics {name} active mismatch");
        if r.active == 0 && c.active == 0 {
            continue;
        }

        assert_approx(
            &format!("physics {name} inertia"),
            r.inertia,
            c.inertia,
            eps,
        );
        assert_approx(
            &format!("physics {name} strength"),
            r.strength,
            c.strength,
            eps,
        );
        assert_approx(
            &format!("physics {name} damping"),
            r.damping,
            c.damping,
            eps,
        );
        assert_approx(
            &format!("physics {name} massInverse"),
            r.mass_inverse,
            c.mass_inverse,
            eps,
        );
        assert_approx(&format!("physics {name} wind"), r.wind, c.wind, eps);
        assert_approx(
            &format!("physics {name} gravity"),
            r.gravity,
            c.gravity,
            eps,
        );
        assert_approx(&format!("physics {name} mix"), r.mix, c.mix, eps);
        assert_eq!(r.reset, c.reset, "physics {name} reset mismatch");
        assert_approx(&format!("physics {name} ux"), r.ux, c.ux, eps);
        assert_approx(&format!("physics {name} uy"), r.uy, c.uy, eps);
        assert_approx(&format!("physics {name} cx"), r.cx, c.cx, eps);
        assert_approx(&format!("physics {name} cy"), r.cy, c.cy, eps);
        assert_approx(&format!("physics {name} tx"), r.tx, c.tx, eps);
        assert_approx(&format!("physics {name} ty"), r.ty, c.ty, eps);
        assert_approx(
            &format!("physics {name} xOffset"),
            r.x_offset,
            c.x_offset,
            eps,
        );
        assert_approx(
            &format!("physics {name} xVelocity"),
            r.x_velocity,
            c.x_velocity,
            eps,
        );
        assert_approx(
            &format!("physics {name} yOffset"),
            r.y_offset,
            c.y_offset,
            eps,
        );
        assert_approx(
            &format!("physics {name} yVelocity"),
            r.y_velocity,
            c.y_velocity,
            eps,
        );
        assert_approx(
            &format!("physics {name} rotateOffset"),
            r.rotate_offset,
            c.rotate_offset,
            eps,
        );
        assert_approx(
            &format!("physics {name} rotateVelocity"),
            r.rotate_velocity,
            c.rotate_velocity,
            eps,
        );
        assert_approx(
            &format!("physics {name} scaleOffset"),
            r.scale_offset,
            c.scale_offset,
            eps,
        );
        assert_approx(
            &format!("physics {name} scaleVelocity"),
            r.scale_velocity,
            c.scale_velocity,
            eps,
        );
        assert_approx(
            &format!("physics {name} remaining"),
            r.remaining,
            c.remaining,
            eps,
        );
        assert_approx(
            &format!("physics {name} lastTime"),
            r.last_time,
            c.last_time,
            eps,
        );
    }

    match (&rust.debug, &cpp.debug) {
        (None, None) => {}
        (Some(_), None) => panic!("rust debug exists but cpp debug is missing"),
        (None, Some(_)) => panic!("cpp debug exists but rust debug is missing"),
        (Some(r), Some(c)) => {
            assert_eq!(r.slot, c.slot, "debug.slot mismatch");
            assert_eq!(
                r.world_vertices.len(),
                c.world_vertices.len(),
                "debug.worldVertices length mismatch"
            );
            for (i, (rv, cv)) in r
                .world_vertices
                .iter()
                .copied()
                .zip(c.world_vertices.iter().copied())
                .enumerate()
            {
                assert_approx(&format!("debug.worldVertices[{i}]"), rv, cv, eps);
            }
        }
    }
}

fn load_data(path: &Path) -> Arc<SkeletonData> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("skel") {
        #[cfg(feature = "binary")]
        {
            let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
            return SkeletonData::from_skel_bytes(&bytes)
                .unwrap_or_else(|e| panic!("parse {path:?}: {e}"));
        }
        #[cfg(not(feature = "binary"))]
        {
            panic!("Input is .skel but spine2d was built without feature `binary`.");
        }
    }

    let json = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    SkeletonData::from_json_str(&json).unwrap_or_else(|e| panic!("parse {path:?}: {e}"))
}

fn step(state: &mut AnimationState, skeleton: &mut Skeleton, dt: f32) {
    step_with_physics(state, skeleton, dt, crate::Physics::None);
}

fn step_physics(state: &mut AnimationState, skeleton: &mut Skeleton, dt: f32) {
    step_with_physics(state, skeleton, dt, crate::Physics::Update);
}

fn step_with_physics(
    state: &mut AnimationState,
    skeleton: &mut Skeleton,
    dt: f32,
    physics: crate::Physics,
) {
    state.update(dt);
    state.apply(skeleton);
    skeleton.update(dt);
    skeleton.update_world_transform_with_physics(physics);
}

#[test]
fn oracle_mix_and_match_skin_switch_backpack_to_hat_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("accessories/backpack"));
    step(&mut state, &mut skeleton, 0.0);

    skeleton.set_skin(Some("accessories/hat-red-yellow"));
    step(&mut state, &mut skeleton, 0.0);

    let rust = dump_pose(&skeleton, 0.0, None);
    let cpp = read_pose(&golden_path(
        "mix_and_match_skin_switch_backpack_to_hat.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_mix_and_match_skin_switch_backpack_to_hat_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("accessories/backpack"));
    step(&mut state, &mut skeleton, 0.0);

    skeleton.set_skin(Some("accessories/hat-red-yellow"));
    step(&mut state, &mut skeleton, 0.0);

    let rust = dump_pose(&skeleton, 0.0, None);
    let cpp = read_pose(&golden_skel_path(
        "mix_and_match_skin_switch_backpack_to_hat.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_diamond_idle_rotating_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path("diamond_idle_rotating_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_diamond_idle_rotating_plus_rotation_add_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);
    let entry = state.set_animation(1, "rotation", true);
    entry.set_additive(&mut state, true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "diamond_idle_rotating_plus_rotation_add_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_diamond_idle_rotating_plus_idle_still_add_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);
    let entry = state.set_animation(1, "idle-still", true);
    entry.set_additive(&mut state, true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "diamond_idle_rotating_plus_idle_still_add_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);
    let entry = state.set_animation(1, "idle-still", true);
    entry.set_additive(&mut state, true);
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_diamond_idle_rotating_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path("diamond_idle_rotating_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_diamond_idle_rotating_plus_idle_still_add_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);
    let entry = state.set_animation(1, "idle-still", true);
    entry.set_additive(&mut state, true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "diamond_idle_rotating_plus_idle_still_add_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);
    let entry = state.set_animation(1, "idle-still", true);
    entry.set_additive(&mut state, true);
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "diamond_idle_rotating_plus_idle_still_add_to_empty_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_diamond_idle_rotating_plus_rotation_add_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle-rotating", true);
    let entry = state.set_animation(1, "rotation", true);
    entry.set_additive(&mut state, true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "diamond_idle_rotating_plus_rotation_add_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_diamond_disappear_t0_8_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "disappear", false);

    let dt = 0.8;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path("diamond_disappear_t0_8.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_diamond_size_changing_rotation_t3_0_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "size-changing-rotation", false);

    let dt = 3.0;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path("diamond_size_changing_rotation_t3_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_diamond_disappear_t0_8_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "disappear", false);

    let dt = 0.8;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path("diamond_disappear_t0_8.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_diamond_size_changing_rotation_t3_0_matches_cpp() {
    let data = load_data(&example_json_path("diamond/export/diamond-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "size-changing-rotation", false);

    let dt = 3.0;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "diamond_size_changing_rotation_t3_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_mix_and_match_skin_switch_hat_aware_t0_1667_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("accessories/backpack"));
    step(&mut state, &mut skeleton, 0.0);

    skeleton.set_skin(Some("accessories/hat-red-yellow"));
    state.set_animation(0, "aware", true);
    step(&mut state, &mut skeleton, 0.1667);

    let rust = dump_pose(&skeleton, 0.1667, None);
    let cpp = read_pose(&golden_path(
        "mix_and_match_skin_switch_hat_aware_t0_1667.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_mix_and_match_skin_switch_hat_aware_t0_1667_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("accessories/backpack"));
    step(&mut state, &mut skeleton, 0.0);

    skeleton.set_skin(Some("accessories/hat-red-yellow"));
    state.set_animation(0, "aware", true);
    step(&mut state, &mut skeleton, 0.1667);

    let rust = dump_pose(&skeleton, 0.1667, None);
    let cpp = read_pose(&golden_skel_path(
        "mix_and_match_skin_switch_hat_aware_t0_1667.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_mix_and_match_walk_plus_dress_up_add_t0_4_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("full-skins/boy"));
    step(&mut state, &mut skeleton, 0.0);

    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.1);

    let dress_up = state.set_animation(1, "dress-up", true);
    dress_up.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path(
        "mix_and_match_walk_plus_dress_up_add_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_mix_and_match_walk_plus_dress_up_add_t0_4_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("full-skins/boy"));
    step(&mut state, &mut skeleton, 0.0);

    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.1);

    let dress_up = state.set_animation(1, "dress-up", true);
    dress_up.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "mix_and_match_walk_plus_dress_up_add_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_goblins_walk_dagger_deform_vertices_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblin"));
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("right-hand-item"));
    let cpp = read_pose(&golden_path("goblins_walk_dagger_t0_3.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_goblins_walk_skin_goblin_left_foot_deform_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblin"));
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_path(
        "goblins_walk_skin_goblin_left_foot_deform_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_goblins_walk_skin_goblingirl_left_foot_deform_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblingirl"));
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_path(
        "goblins_walk_skin_goblingirl_left_foot_deform_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_goblins_walk_skin_goblin_left_foot_deform_jitter_dt_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblin"));
    state.set_animation(0, "walk", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_path(
        "goblins_walk_skin_goblin_left_foot_deform_jitter_dt_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_goblins_walk_skin_goblingirl_left_foot_deform_jitter_dt_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblingirl"));
    state.set_animation(0, "walk", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_path(
        "goblins_walk_skin_goblingirl_left_foot_deform_jitter_dt_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_goblins_walk_dagger_deform_vertices_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblin"));
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("right-hand-item"));
    let cpp = read_pose(&golden_skel_path("goblins_walk_dagger_t0_3.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_stretchyman_sneak_t1_366667_matches_cpp() {
    let data = load_data(&example_json_path(
        "stretchyman/export/stretchyman-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "sneak", true);
    step(&mut state, &mut skeleton, 1.366_667);

    let rust = dump_pose(&skeleton, 1.366_667, Some("back-leg-path"));
    let cpp = read_pose(&golden_path("stretchyman_sneak_t1_366667.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_stretchyman_sneak_t1_366667_matches_cpp() {
    let data = load_data(&example_json_path(
        "stretchyman/export/stretchyman-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "sneak", true);
    step(&mut state, &mut skeleton, 1.366_667);

    let rust = dump_pose(&skeleton, 1.366_667, Some("back-leg-path"));
    let cpp = read_pose(&golden_skel_path("stretchyman_sneak_t1_366667.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_hero_idle_head_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_path("hero_idle_head_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_hero_idle_head_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_path("hero_idle_head_deform_jitter_dt_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_hero_idle_plus_run_add_head_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    step(&mut state, &mut skeleton, 0.1);

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_path(
        "hero_idle_plus_run_add_head_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_hero_idle_plus_run_add_head_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_path(
        "hero_idle_plus_run_add_head_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_hero_idle_plus_run_add_to_empty_mix0_2_head_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    step(&mut state, &mut skeleton, 0.1);

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_path(
        "hero_idle_plus_run_add_to_empty_mix0_2_head_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_hero_idle_plus_run_add_to_empty_mix0_2_head_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_path(
        "hero_idle_plus_run_add_to_empty_mix0_2_head_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_hero_idle_head_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_skel_path("hero_idle_head_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_hero_idle_head_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_skel_path(
        "hero_idle_head_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_hero_idle_plus_run_add_head_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    step(&mut state, &mut skeleton, 0.1);

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_skel_path(
        "hero_idle_plus_run_add_head_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_hero_idle_plus_run_add_head_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_skel_path(
        "hero_idle_plus_run_add_head_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_hero_idle_plus_run_add_to_empty_mix0_2_head_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    step(&mut state, &mut skeleton, 0.1);

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_skel_path(
        "hero_idle_plus_run_add_to_empty_mix0_2_head_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_hero_idle_plus_run_add_to_empty_mix0_2_head_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("hero/export/hero-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let run = state.set_animation(1, "run", true);
    run.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head"));
    let cpp = read_pose(&golden_skel_path(
        "hero_idle_plus_run_add_to_empty_mix0_2_head_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_head_base_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_path("owl_up_head_base_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_head_base_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_path("owl_up_head_base_deform_jitter_dt_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_idle_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_path("owl_idle_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_idle_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_path("owl_idle_physics_jitter_dt_t1_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_idle_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_idle_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_idle_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_idle_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_head_base_deform_physics_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);

    let dt = 1.0 / 60.0;
    for _ in 0..6 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..27 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_head_base_deform_physics_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_head_base_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_head_base_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_head_base_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_head_base_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_l_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path("owl_up_l_wing_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_l_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path("owl_up_l_wing_deform_jitter_dt_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_l_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_l_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_l_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_l_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_r_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_path("owl_up_r_wing_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_r_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_path("owl_up_r_wing_deform_jitter_dt_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_r_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_r_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_r_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_r_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_blink_l_wing_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.0);

    state.set_animation(1, "blink", false);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, Some("L_wing"));
    let cpp = read_pose(&golden_path("owl_up_plus_blink_l_wing_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_blink_l_wing_jitter_dt_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    state.set_animation(1, "blink", false);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.5, Some("L_wing"));
    let cpp = read_pose(&golden_path("owl_up_plus_blink_l_wing_jitter_dt_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_blink_to_empty_mix0_2_l_wing_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.0);

    state.set_animation(1, "blink", false);
    step(&mut state, &mut skeleton, 0.5);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_blink_to_empty_mix0_2_l_wing_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_owl_up_plus_blink_to_empty_mix0_2_l_wing_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    state.set_animation(1, "blink", false);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_path(
        "owl_up_plus_blink_to_empty_mix0_2_l_wing_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_head_base_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_skel_path("owl_up_head_base_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_head_base_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_head_base_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_idle_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_skel_path("owl_idle_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_idle_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_skel_path("owl_idle_physics_jitter_dt_t1_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_idle_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_idle_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_idle_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_idle_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_head_base_deform_physics_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);

    let dt = 1.0 / 60.0;
    for _ in 0..6 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..27 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_head_base_deform_physics_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_head_base_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_head_base_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_head_base_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_head_base_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("head-base"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_to_empty_mix0_2_head_base_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_blink_l_wing_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.0);

    state.set_animation(1, "blink", false);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path("owl_up_plus_blink_l_wing_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_blink_l_wing_jitter_dt_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    state.set_animation(1, "blink", false);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.5, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_blink_l_wing_jitter_dt_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_blink_to_empty_mix0_2_l_wing_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.0);

    state.set_animation(1, "blink", false);
    step(&mut state, &mut skeleton, 0.5);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_blink_to_empty_mix0_2_l_wing_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_blink_to_empty_mix0_2_l_wing_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    state.set_animation(1, "blink", false);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_blink_to_empty_mix0_2_l_wing_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_l_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path("owl_up_l_wing_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_l_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_l_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_l_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_l_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_l_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_l_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("L_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_to_empty_mix0_2_l_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_r_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.55);

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_skel_path("owl_up_r_wing_deform_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_r_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..27 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_r_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_r_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.45);

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_r_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_r_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..24 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_r_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    step(&mut state, &mut skeleton, 0.1);

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_jitter_dt_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("owl/export/owl-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "up", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let left = state.set_animation(1, "left", true);
    left.set_additive(&mut state, true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, Some("R_wing"));
    let cpp = read_pose(&golden_skel_path(
        "owl_up_plus_left_add_to_empty_mix0_2_r_wing_deform_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_goblins_walk_skin_goblin_left_foot_deform_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblin"));
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_skel_path(
        "goblins_walk_skin_goblin_left_foot_deform_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_goblins_walk_skin_goblingirl_left_foot_deform_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblingirl"));
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_skel_path(
        "goblins_walk_skin_goblingirl_left_foot_deform_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_goblins_walk_skin_goblin_left_foot_deform_jitter_dt_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblin"));
    state.set_animation(0, "walk", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_skel_path(
        "goblins_walk_skin_goblin_left_foot_deform_jitter_dt_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_goblins_walk_skin_goblingirl_left_foot_deform_jitter_dt_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("goblins/export/goblins-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("goblingirl"));
    state.set_animation(0, "walk", true);
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..12 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.3, Some("left-foot"));
    let cpp = read_pose(&golden_skel_path(
        "goblins_walk_skin_goblingirl_left_foot_deform_jitter_dt_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_shoot_clipping_deform_vertices_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("clipping"));
    let cpp = read_pose(&golden_path("tank_shoot_clipping_deform_t0_3.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_shoot_clipping_deform_vertices_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, Some("clipping"));
    let cpp = read_pose(&golden_skel_path("tank_shoot_clipping_deform_t0_3.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.4, Some("clipping"));
    let cpp = read_pose(&golden_path("tank_drive_plus_shoot_add_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_smoke_glow_deform_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.15);

    let rust = dump_pose(&skeleton, 0.25, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_smoke_glow_deform_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_smoke_glow_deform_jitter_dt_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.25, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_smoke_glow_deform_jitter_dt_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_alpha0_5_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.4, Some("clipping"));
    let cpp = read_pose(&golden_path("tank_drive_plus_shoot_add_alpha0_5_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_alpha0_5_jitter_dt_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..13 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.4, Some("clipping"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_alpha0_5_jitter_dt_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.4, Some("clipping"));
    let cpp = read_pose(&golden_skel_path("tank_drive_plus_shoot_add_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_smoke_glow_deform_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.15);

    let rust = dump_pose(&skeleton, 0.25, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_smoke_glow_deform_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_smoke_glow_deform_jitter_dt_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.25, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_smoke_glow_deform_jitter_dt_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_alpha0_5_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.4, Some("clipping"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_alpha0_5_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_alpha0_5_jitter_dt_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..13 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.4, Some("clipping"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_alpha0_5_jitter_dt_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_1_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_1_jitter_dt_t0_55_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 2.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_0_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_0_jitter_dt_t0_55_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 2.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.15);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.35, Some("clipping"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);

    // Critical edge case: immediately mix out before the Add entry is ever applied.
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_shoot_plus_drive_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "shoot", false);

    let drive = state.set_animation(1, "drive", true);
    drive.set_additive(&mut state, true);

    // Critical edge case: immediately mix out before the Add entry is ever applied.
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "tank_shoot_plus_drive_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_plus_shoot_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);

    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_plus_shoot_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_shoot_plus_drive_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "shoot", false);

    let drive = state.set_animation(1, "drive", true);
    drive.set_additive(&mut state, true);

    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_shoot_plus_drive_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.15);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.35, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_jitter_dt_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.35, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_jitter_dt_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_1_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_1_jitter_dt_t0_55_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 2.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_0_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_to_empty_mix_draw_order_threshold_0_jitter_dt_t0_55_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..16 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_jitter_dt_t0_55.json",
    ));
    assert_pose_parity(&rust, &cpp, 2.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_alpha0_5_to_empty_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.15);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.35, Some("clipping"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_alpha0_5_to_empty_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_plus_shoot_add_alpha0_5_to_empty_jitter_dt_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);

    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..6 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.35, Some("clipping"));
    let cpp = read_pose(&golden_path(
        "tank_drive_plus_shoot_add_alpha0_5_to_empty_jitter_dt_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_shoot_to_shoot_mix_draw_order_threshold_0_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path(
        "tank_shoot_to_shoot_mixDrawOrderThreshold_0_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_shoot_to_shoot_mix_draw_order_threshold_0_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_shoot_to_shoot_mixDrawOrderThreshold_0_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_shoot_to_shoot_mix_draw_order_threshold_1_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path(
        "tank_shoot_to_shoot_mixDrawOrderThreshold_1_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_draw_order_threshold_0_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixDrawOrderThreshold_0_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_draw_order_threshold_1_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixDrawOrderThreshold_1_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_attachment_threshold_0_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixAttachmentThreshold_0_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_attachment_threshold_1_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_attachment_threshold(&mut state, 1.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixAttachmentThreshold_1_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_shoot_to_shoot_to_drive_hold_mix_smoke_glow_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);
    state_data.set_mix("shoot", "drive", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.1);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(1, "drive", true);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.2, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_shoot_to_shoot_to_drive_holdMix_smoke_glow_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_shoot_to_shoot_to_drive_hold_mix_smoke_glow_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);
    state_data.set_mix("shoot", "drive", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.1);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(1, "drive", true);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.2, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_shoot_to_shoot_to_drive_holdMix_smoke_glow_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_shoot_to_shoot_mix_draw_order_threshold_1_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "tank_shoot_to_shoot_mixDrawOrderThreshold_1_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_draw_order_threshold_0_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixDrawOrderThreshold_0_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_draw_order_threshold_1_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixDrawOrderThreshold_1_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_attachment_threshold_0_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixAttachmentThreshold_0_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mix_attachment_threshold_1_smoke_glow_t0_4_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("shoot", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);

    let shoot_add = state.set_animation(2, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.35);
    shoot.set_alpha(&mut state, 1.0);
    shoot.set_mix_attachment_threshold(&mut state, 1.0);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.4, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_t2_shoot_add_alpha0_5_t1_shoot_to_shoot_mixAttachmentThreshold_1_smoke_glow_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_tank_drive_t1_shoot_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_smoke_glow_matches_cpp() {
    let data = load_data(&example_json_path("tank/export/tank-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_add = state.set_animation(1, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_replace = state.set_animation(2, "shoot", false);
    shoot_replace.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.3, Some("smoke-glow"));
    let cpp = read_pose(&golden_path(
        "tank_drive_t1_shoot_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_smoke_glow.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_tank_drive_t1_shoot_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_smoke_glow_matches_cpp()
 {
    let data = load_data(&example_json_path("tank/export/tank-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "drive", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_add = state.set_animation(1, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_replace = state.set_animation(2, "shoot", false);
    shoot_replace.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.3, Some("smoke-glow"));
    let cpp = read_pose(&golden_skel_path(
        "tank_drive_t1_shoot_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_smoke_glow.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_aim_add_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.2, None);
    let cpp = read_pose(&golden_path("spineboy_run_plus_aim_add_t0_2.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_aim_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);

    // Critical edge case: immediately mix out before the Add entry is ever applied.
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_aim_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_portal_add_to_empty_mix0_2_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);
    step(&mut state, &mut skeleton, 0.2);

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.2);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_portal_add_to_empty_mix0_2_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_portal_add_to_empty_mix0_2_jitter_dt_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_portal_add_to_empty_mix0_2_jitter_dt_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_portal_add_reverse_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);
    portal.set_reverse(&mut state, true);

    let dt = 0.35;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_portal_add_reverse_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_portal_add_reverse_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);
    portal.set_reverse(&mut state, true);

    // Critical edge case: immediately mix out before the (reverse) Add entry is ever applied.
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_portal_add_reverse_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_portal_reverse_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_reverse(&mut state, true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path("spineboy_portal_reverse_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_portal_alpha0_5_shortest_rotation_true_t2_0_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_alpha(&mut state, 0.5);
    portal.set_shortest_rotation(&mut state, true);

    let dt = 2.0;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "spineboy_portal_alpha0_5_shortestRotation_true_t2_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_portal_reverse_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let portal = state.set_animation(0, "portal", false);
    portal.set_reverse(&mut state, true);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_to_portal_reverse_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_portal_mix0_2_shortest_rotation_true_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let portal = state.set_animation(0, "portal", false);
    portal.set_shortest_rotation(&mut state, true);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_to_portal_mix0_2_shortestRotation_true_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_portal_reverse_to_shoot_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("portal", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_reverse(&mut state, true);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_path(
        "spineboy_portal_reverse_to_shoot_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_portal_shortest_rotation_true_to_shoot_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("portal", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_shortest_rotation(&mut state, true);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_path(
        "spineboy_portal_shortestRotation_true_to_shoot_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_portal_alpha0_5_reset_rotation_directions_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_alpha(&mut state, 0.5);

    step(&mut state, &mut skeleton, 0.2);
    portal.reset_rotation_directions(&mut state);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path(
        "spineboy_portal_alpha0_5_reset_rotation_directions_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_portal_add_reverse_to_shoot_replace_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("portal", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_additive(&mut state, true);
    portal.set_reverse(&mut state, true);
    step(&mut state, &mut skeleton, 0.05);

    // Deliberately leave the new entry as default `MixBlend::Replace` to lock the
    // from(Add) -> to(Replace) interaction in `applyMixingFrom` under `reverse=true`.
    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_path(
        "spineboy_portal_add_reverse_to_shoot_replace_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_aim_to_shoot_to_portal_hold_mix_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    state.set_animation(1, "aim", true);
    step(&mut state, &mut skeleton, 0.1);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(1, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.2, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_aim_to_shoot_to_portal_holdMix_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_aim_to_shoot_to_portal_hold_mix_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    state.set_animation(1, "aim", true);
    step(&mut state, &mut skeleton, 0.1);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(1, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.2, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_aim_to_shoot_to_portal_holdMix_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_aim_to_shoot_interrupt_to_portal_mix0_2_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    state.set_animation(1, "aim", true);
    step(&mut state, &mut skeleton, 0.1);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    // Interrupt the mix before it completes: portal's interruptAlpha should capture shoot's mix ratio.
    state.set_animation(1, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.2, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_aim_to_shoot_interrupt_to_portal_mix0_2_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_aim_to_shoot_interrupt_to_portal_mix0_2_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    state.set_animation(1, "aim", true);
    step(&mut state, &mut skeleton, 0.1);

    state.set_animation(1, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(1, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.2, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_aim_to_shoot_interrupt_to_portal_mix0_2_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_t1_aim_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    aim.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_t1_aim_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    aim.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_shoot_replace_alpha0_5_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_t1_aim_add_alpha0_5_t2_shoot_add_alpha0_5_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    aim.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_shoot_add_alpha0_5_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_t1_aim_add_alpha0_5_t2_shoot_add_alpha0_5_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    aim.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_shoot_add_alpha0_5_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_0_mix_draw_order_threshold_0_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim_add = state.set_animation(1, "aim", true);
    aim_add.set_additive(&mut state, true);
    aim_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);

    step(&mut state, &mut skeleton, 0.05);

    // Interrupt before `aim -> shoot` completes to lock `interruptAlpha` and threshold gating.
    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_0_mix_draw_order_threshold_0_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim_add = state.set_animation(1, "aim", true);
    aim_add.set_additive(&mut state, true);
    aim_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);

    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_1_mix_draw_order_threshold_1_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim_add = state.set_animation(1, "aim", true);
    aim_add.set_additive(&mut state, true);
    aim_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);

    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_1_mix_draw_order_threshold_1_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let aim_add = state.set_animation(1, "aim", true);
    aim_add.set_additive(&mut state, true);
    aim_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);

    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_t1_aim_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_0_mix_draw_order_threshold_0_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_add = state.set_animation(1, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_0_mix_draw_order_threshold_0_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_add = state.set_animation(1, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_1_mix_draw_order_threshold_1_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_add = state.set_animation(1, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_path(
        "spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mix_attachment_threshold_1_mix_draw_order_threshold_1_interrupt_to_portal_t0_3_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    state_data.set_mix("shoot", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.1);

    let shoot_add = state.set_animation(1, "shoot", false);
    shoot_add.set_additive(&mut state, true);
    shoot_add.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "aim", true);
    step(&mut state, &mut skeleton, 0.05);

    let shoot = state.set_animation(2, "shoot", false);
    shoot.set_mix_attachment_threshold(&mut state, 1.0);
    shoot.set_mix_draw_order_threshold(&mut state, 1.0);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(2, "portal", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.3, Some("gun"));
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_t1_shoot_add_alpha0_5_t2_aim_to_shoot_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_interrupt_to_portal_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_alien_run_plus_death_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("alien/export/alien-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    let death = state.set_animation(1, "death", false);
    death.set_additive(&mut state, true);

    // Critical edge case: immediately mix out before the Add entry is ever applied.
    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_path(
        "alien_run_plus_death_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_aim_add_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.2, None);
    let cpp = read_pose(&golden_skel_path("spineboy_run_plus_aim_add_t0_2.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_aim_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);

    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_aim_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_portal_add_to_empty_mix0_2_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);
    step(&mut state, &mut skeleton, 0.2);

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.2);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_portal_add_to_empty_mix0_2_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_portal_add_to_empty_mix0_2_jitter_dt_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_portal_add_to_empty_mix0_2_jitter_dt_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_portal_add_reverse_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);
    portal.set_reverse(&mut state, true);

    let dt = 0.35;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_portal_add_reverse_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_portal_add_reverse_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    let portal = state.set_animation(1, "portal", false);
    portal.set_additive(&mut state, true);
    portal.set_reverse(&mut state, true);

    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_portal_add_reverse_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_portal_reverse_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_reverse(&mut state, true);

    let dt = 0.5;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path("spineboy_portal_reverse_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_portal_alpha0_5_shortest_rotation_true_t2_0_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_alpha(&mut state, 0.5);
    portal.set_shortest_rotation(&mut state, true);

    let dt = 2.0;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_portal_alpha0_5_shortestRotation_true_t2_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_portal_reverse_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let portal = state.set_animation(0, "portal", false);
    portal.set_reverse(&mut state, true);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_to_portal_reverse_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_portal_mix0_2_shortest_rotation_true_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "portal", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let portal = state.set_animation(0, "portal", false);
    portal.set_shortest_rotation(&mut state, true);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_to_portal_mix0_2_shortestRotation_true_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_portal_reverse_to_shoot_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("portal", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_reverse(&mut state, true);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_portal_reverse_to_shoot_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_portal_shortest_rotation_true_to_shoot_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("portal", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_shortest_rotation(&mut state, true);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_portal_shortestRotation_true_to_shoot_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_portal_alpha0_5_reset_rotation_directions_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_alpha(&mut state, 0.5);

    step(&mut state, &mut skeleton, 0.2);
    portal.reset_rotation_directions(&mut state);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_portal_alpha0_5_reset_rotation_directions_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_portal_add_reverse_to_shoot_replace_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("portal", "shoot", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    let portal = state.set_animation(0, "portal", false);
    portal.set_additive(&mut state, true);
    portal.set_reverse(&mut state, true);
    step(&mut state, &mut skeleton, 0.05);

    state.set_animation(0, "shoot", false);
    step(&mut state, &mut skeleton, 0.05);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_portal_add_reverse_to_shoot_replace_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_alien_run_plus_death_add_to_empty_immediate_mix0_2_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("alien/export/alien-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);

    let death = state.set_animation(1, "death", false);
    death.set_additive(&mut state, true);

    state.set_empty_animation(1, 0.2);

    let dt = 0.1;
    step(&mut state, &mut skeleton, dt);

    let rust = dump_pose(&skeleton, dt, None);
    let cpp = read_pose(&golden_skel_path(
        "alien_run_plus_death_add_to_empty_immediate_mix0_2_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_shoot_add_to_empty_mix0_2_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);
    step(&mut state, &mut skeleton, 0.2);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.2);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_shoot_add_to_empty_mix0_2_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_shoot_add_to_empty_mix0_2_jitter_dt_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_shoot_add_to_empty_mix0_2_jitter_dt_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_shoot_add_to_empty_mix0_2_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);
    step(&mut state, &mut skeleton, 0.2);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    step(&mut state, &mut skeleton, 0.2);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_shoot_add_to_empty_mix0_2_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_shoot_add_to_empty_mix0_2_jitter_dt_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_shoot_add_to_empty_mix0_2_jitter_dt_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_shoot_add_alpha0_5_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);
    step(&mut state, &mut skeleton, 0.2);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_shoot_add_alpha0_5_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_shoot_add_alpha0_5_jitter_dt_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path(
        "spineboy_run_plus_shoot_add_alpha0_5_jitter_dt_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_shoot_add_alpha0_5_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);
    step(&mut state, &mut skeleton, 0.2);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_shoot_add_alpha0_5_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_plus_shoot_add_alpha0_5_jitter_dt_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    shoot.set_alpha(&mut state, 0.5);

    for _ in 0..10 {
        step(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..7 {
        step(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_plus_shoot_add_alpha0_5_jitter_dt_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_plus_aim_add_alpha0_5_t0_2_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    aim.set_alpha(&mut state, 0.5);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.2, None);
    let cpp = read_pose(&golden_path("spineboy_run_plus_aim_add_alpha0_5_t0_2.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_aim_to_shoot_add_t0_4_matches_cpp() {
    let path = example_json_path("spineboy/export/spineboy-pro.json");
    let json = std::fs::read_to_string(&path).expect("read spineboy-pro.json");
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).expect("parse spineboy-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    let mut state = AnimationState::new(state_data);

    state.set_animation(0, "run", true);
    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    state.update(0.3);
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    state.update(0.1);
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path("spineboy_aim_to_shoot_add_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_aim_add_to_shoot_replace_t0_4_matches_cpp() {
    let path = example_json_path("spineboy/export/spineboy-pro.json");
    let json = std::fs::read_to_string(&path).expect("read spineboy-pro.json");
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).expect("parse spineboy-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    let mut state = AnimationState::new(state_data);

    state.set_animation(0, "run", true);

    let aim = state.set_animation(1, "aim", true);
    aim.set_additive(&mut state, true);
    state.update(0.3);
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    // Deliberately leave the new entry as default `MixBlend::Replace` to lock the
    // from(Add) -> to(Replace) interaction in `applyMixingFrom`.
    state.set_animation(1, "shoot", false);
    state.update(0.1);
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path("spineboy_aim_add_to_shoot_replace_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_aim_replace_to_shoot_add_t0_4_matches_cpp() {
    let path = example_json_path("spineboy/export/spineboy-pro.json");
    let json = std::fs::read_to_string(&path).expect("read spineboy-pro.json");
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).expect("parse spineboy-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("aim", "shoot", 0.2);
    let mut state = AnimationState::new(state_data);

    state.set_animation(0, "run", true);

    state.set_animation(1, "aim", true);
    state.update(0.3);
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_additive(&mut state, true);
    state.update(0.1);
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path("spineboy_aim_replace_to_shoot_add_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_shoot_alpha_attachment_threshold_0_6_alpha_0_5_t0_1_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha(&mut state, 0.5);
    shoot.set_alpha_attachment_threshold(&mut state, 0.6);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_path(
        "spineboy_shoot_alphaAttachmentThreshold_0_6_alpha_0_5_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_shoot_to_empty_mix_attachment_threshold_0_mix_draw_order_threshold_0_t0_2_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.0);

    let shoot = state.set_animation(1, "shoot", false);
    shoot.set_alpha_attachment_threshold(&mut state, 0.0);
    shoot.set_mix_attachment_threshold(&mut state, 0.0);
    shoot.set_mix_draw_order_threshold(&mut state, 0.0);
    step(&mut state, &mut skeleton, 0.1);

    state.set_empty_animation(1, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.2, None);
    let cpp = read_pose(&golden_path(
        "spineboy_shoot_to_empty_mixAttachmentThreshold_0_mixDrawOrderThreshold_0_t0_2.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_ess_run_to_empty_immediate_mix0_2_mix_attachment_threshold_1_mix_draw_order_threshold_1_t0_1_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();

    // Force attachment + drawOrder timelines to apply during mixingFrom.
    let run = state.set_animation(0, "run", true);
    run.set_mix_attachment_threshold(&mut state, 1.0);
    run.set_mix_draw_order_threshold(&mut state, 1.0);

    // Critical edge case: immediately mix out before the entry is ever applied.
    state.set_empty_animation(0, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_path(
        "spineboy_ess_run_to_empty_immediate_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_ess_run_to_empty_immediate_mix0_2_mix_attachment_threshold_1_mix_draw_order_threshold_1_t0_1_matches_cpp()
 {
    let data = load_data(&example_json_path("spineboy/export/spineboy-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();

    let run = state.set_animation(0, "run", true);
    run.set_mix_attachment_threshold(&mut state, 1.0);
    run.set_mix_draw_order_threshold(&mut state, 1.0);

    state.set_empty_animation(0, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.1, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_ess_run_to_empty_immediate_mix0_2_mixAttachmentThreshold_1_mixDrawOrderThreshold_1_t0_1.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_mix_and_match_skin_switch_boy_to_girl_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("full-skins/boy"));
    step(&mut state, &mut skeleton, 0.0);

    skeleton.set_skin(Some("full-skins/girl"));
    step(&mut state, &mut skeleton, 0.0);

    let rust = dump_pose(&skeleton, 0.0, None);
    let cpp = read_pose(&golden_path("mix_and_match_skin_switch_boy_to_girl.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_mix_and_match_skin_switch_boy_to_girl_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    skeleton.set_skin(Some("full-skins/boy"));
    step(&mut state, &mut skeleton, 0.0);

    skeleton.set_skin(Some("full-skins/girl"));
    step(&mut state, &mut skeleton, 0.0);

    let rust = dump_pose(&skeleton, 0.0, None);
    let cpp = read_pose(&golden_skel_path(
        "mix_and_match_skin_switch_boy_to_girl.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_mix_and_match_dance_t0_25_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "dance", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_path("mix_and_match_dance_t0_25.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_mix_and_match_dance_t0_25_matches_cpp() {
    let data = load_data(&example_json_path(
        "mix-and-match/export/mix-and-match-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "dance", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_skel_path("mix_and_match_dance_t0_25.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_coin_animation_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("coin/export/coin-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "animation", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, None);
    let cpp = read_pose(&golden_path("coin_animation_t0_3.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_6_arcs_arcs_t5_666667_matches_cpp() {
    let data = load_data(&example_json_path("6-arcs/export/6-arcs-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "arcs", false);
    step(&mut state, &mut skeleton, 5.666_667);

    let rust = dump_pose(&skeleton, 5.666_667, Some("arc-tracker"));
    let cpp = read_pose(&golden_path("6_arcs_arcs_t5_666667.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_8_follow_through_ball_follow_through_t2_4_matches_cpp() {
    let data = load_data(&example_json_path(
        "8-follow-through/export/8-follow-through-pro-ball.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "follow-through", false);
    step(&mut state, &mut skeleton, 2.4);

    let rust = dump_pose(&skeleton, 2.4, None);
    let cpp = read_pose(&golden_path(
        "8_follow_through_ball_follow_through_t2_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_food_app_search_add_bread_t1_4_matches_cpp() {
    let data = load_data(&example_json_path(
        "food-app/export/food-app-search-ess.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "add-bread", false);
    step(&mut state, &mut skeleton, 1.4);

    let rust = dump_pose(&skeleton, 1.4, None);
    let cpp = read_pose(&golden_path("food_app_search_add_bread_t1_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_food_app_search_add_carrot_t0_2_matches_cpp() {
    let data = load_data(&example_json_path(
        "food-app/export/food-app-search-ess.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "add-carrot", false);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.2, None);
    let cpp = read_pose(&golden_path("food_app_search_add_carrot_t0_2.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_coin_animation_t0_3_matches_cpp() {
    let data = load_data(&example_json_path("coin/export/coin-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "animation", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, None);
    let cpp = read_pose(&golden_skel_path("coin_animation_t0_3.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_6_arcs_arcs_t5_666667_matches_cpp() {
    let data = load_data(&example_json_path("6-arcs/export/6-arcs-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "arcs", false);
    step(&mut state, &mut skeleton, 5.666_667);

    let rust = dump_pose(&skeleton, 5.666_667, Some("arc-tracker"));
    let cpp = read_pose(&golden_skel_path("6_arcs_arcs_t5_666667.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_8_follow_through_ball_follow_through_t2_4_matches_cpp() {
    let data = load_data(&example_json_path(
        "8-follow-through/export/8-follow-through-pro-ball.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "follow-through", false);
    step(&mut state, &mut skeleton, 2.4);

    let rust = dump_pose(&skeleton, 2.4, None);
    let cpp = read_pose(&golden_skel_path(
        "8_follow_through_ball_follow_through_t2_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_food_app_search_add_bread_t1_4_matches_cpp() {
    let data = load_data(&example_json_path(
        "food-app/export/food-app-search-ess.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "add-bread", false);
    step(&mut state, &mut skeleton, 1.4);

    let rust = dump_pose(&skeleton, 1.4, None);
    let cpp = read_pose(&golden_skel_path("food_app_search_add_bread_t1_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_food_app_search_add_carrot_t0_2_matches_cpp() {
    let data = load_data(&example_json_path(
        "food-app/export/food-app-search-ess.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "add-carrot", false);
    step(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.2, None);
    let cpp = read_pose(&golden_skel_path("food_app_search_add_carrot_t0_2.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_windmill_animation_t3_0_matches_cpp() {
    let data = load_data(&example_json_path("windmill/export/windmill-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "animation", true);
    step(&mut state, &mut skeleton, 3.0);

    let rust = dump_pose(&skeleton, 3.0, None);
    let cpp = read_pose(&golden_path("windmill_animation_t3_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_windmill_animation_t3_0_matches_cpp() {
    let data = load_data(&example_json_path("windmill/export/windmill-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "animation", true);
    step(&mut state, &mut skeleton, 3.0);

    let rust = dump_pose(&skeleton, 3.0, None);
    let cpp = read_pose(&golden_skel_path("windmill_animation_t3_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_powerup_bounce_t0_7_matches_cpp() {
    let data = load_data(&example_json_path("powerup/export/powerup-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "bounce", true);
    step(&mut state, &mut skeleton, 0.7);

    let rust = dump_pose(&skeleton, 0.7, Some("token"));
    let cpp = read_pose(&golden_path("powerup_bounce_t0_7.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_powerup_bounce_t0_7_matches_cpp() {
    let data = load_data(&example_json_path("powerup/export/powerup-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "bounce", true);
    step(&mut state, &mut skeleton, 0.7);

    let rust = dump_pose(&skeleton, 0.7, Some("token"));
    let cpp = read_pose(&golden_skel_path("powerup_bounce_t0_7.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_speedy_run_t0_433333_matches_cpp() {
    let data = load_data(&example_json_path("speedy/export/speedy-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.433_333_34);

    let rust = dump_pose(&skeleton, 0.433_333_34, None);
    let cpp = read_pose(&golden_path("speedy_run_t0_433333.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_speedy_run_t0_433333_matches_cpp() {
    let data = load_data(&example_json_path("speedy/export/speedy-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.433_333_34);

    let rust = dump_pose(&skeleton, 0.433_333_34, None);
    let cpp = read_pose(&golden_skel_path("speedy_run_t0_433333.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_chibi_stickers_movement_trot_front_t0_3_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "movement/trot-front", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, None);
    let cpp = read_pose(&golden_path(
        "chibi_stickers_movement__trot-front_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_chibi_stickers_movement_trot_front_t0_3_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "movement/trot-front", true);
    step(&mut state, &mut skeleton, 0.3);

    let rust = dump_pose(&skeleton, 0.3, None);
    let cpp = read_pose(&golden_skel_path(
        "chibi_stickers_movement__trot-front_t0_3.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_chibi_stickers_emotes_excited_t0_35_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "emotes/excited", true);
    step(&mut state, &mut skeleton, 0.35);

    let rust = dump_pose(&skeleton, 0.35, None);
    let cpp = read_pose(&golden_path("chibi_stickers_emotes__excited_t0_35.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_chibi_stickers_emotes_excited_t0_35_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "emotes/excited", true);
    step(&mut state, &mut skeleton, 0.35);

    let rust = dump_pose(&skeleton, 0.35, None);
    let cpp = read_pose(&golden_skel_path(
        "chibi_stickers_emotes__excited_t0_35.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_chibi_stickers_interactive_password_hooray_t0_25_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "interactive/password/hooray", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_path(
        "chibi_stickers_interactive__password__hooray_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_chibi_stickers_interactive_password_hooray_t0_25_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "interactive/password/hooray", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_skel_path(
        "chibi_stickers_interactive__password__hooray_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_chibi_stickers_emotes_dramatic_stare_t0_5_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "emotes/dramatic-stare", true);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_path(
        "chibi_stickers_emotes__dramatic-stare_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_chibi_stickers_emotes_dramatic_stare_t0_5_matches_cpp() {
    let data = load_data(&example_json_path(
        "chibi-stickers/export/chibi-stickers.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "emotes/dramatic-stare", true);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_skel_path(
        "chibi_stickers_emotes__dramatic-stare_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_vine_grow_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("vine/export/vine-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "grow", false);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_path("vine_grow_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_vine_grow_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("vine/export/vine-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "grow", false);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_skel_path("vine_grow_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_raptor_walk_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("raptor/export/raptor-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_path("raptor_walk_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_raptor_walk_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("raptor/export/raptor-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.5);

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_skel_path("raptor_walk_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_dragon_flying_sequence_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_path("dragon_flying_sequence_t0_25.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_dragon_flying_sequence_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_skel_path("dragon_flying_sequence_t0_25.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_dragon_flying_sequence_t0_65_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.65);

    let rust = dump_pose(&skeleton, 0.65, None);
    let cpp = read_pose(&golden_path("dragon_flying_sequence_t0_65.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_dragon_flying_sequence_t0_65_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.65);

    let rust = dump_pose(&skeleton, 0.65, None);
    let cpp = read_pose(&golden_skel_path("dragon_flying_sequence_t0_65.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_dragon_flying_sequence_t0_76_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.76);

    let rust = dump_pose(&skeleton, 0.76, None);
    let cpp = read_pose(&golden_path("dragon_flying_sequence_t0_76.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_dragon_flying_sequence_t0_76_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.76);

    let rust = dump_pose(&skeleton, 0.76, None);
    let cpp = read_pose(&golden_skel_path("dragon_flying_sequence_t0_76.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_dragon_flying_sequence_t0_85_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.85);

    let rust = dump_pose(&skeleton, 0.85, None);
    let cpp = read_pose(&golden_path("dragon_flying_sequence_t0_85.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_dragon_flying_sequence_t0_85_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.85);

    let rust = dump_pose(&skeleton, 0.85, None);
    let cpp = read_pose(&golden_skel_path("dragon_flying_sequence_t0_85.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_dragon_flying_sequence_t0_98_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.98);

    let rust = dump_pose(&skeleton, 0.98, None);
    let cpp = read_pose(&golden_path("dragon_flying_sequence_t0_98.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_dragon_flying_sequence_t0_98_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.98);

    let rust = dump_pose(&skeleton, 0.98, None);
    let cpp = read_pose(&golden_skel_path("dragon_flying_sequence_t0_98.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_dragon_flying_to_empty_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.25);

    state.set_empty_animation(0, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.35, None);
    let cpp = read_pose(&golden_path("dragon_flying_to_empty_t0_35.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_dragon_flying_to_empty_t0_35_matches_cpp() {
    let data = load_data(&example_json_path("dragon/export/dragon-ess.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "flying", true);
    step(&mut state, &mut skeleton, 0.25);

    state.set_empty_animation(0, 0.2);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.35, None);
    let cpp = read_pose(&golden_skel_path("dragon_flying_to_empty_t0_35.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_walk_mix0_2_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "walk", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.3);

    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path("spineboy_run_to_walk_mix0_2_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_walk_mix0_2_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "walk", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.3);

    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path("spineboy_run_to_walk_mix0_2_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_walk_mix0_2_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "walk", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.3);

    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_path("spineboy_run_to_walk_mix0_2_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_walk_headless_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..5 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_path("spineboy_run_to_walk_headless_t0_25.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_walk_headless_t0_25_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..5 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.25, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_to_walk_headless_t0_25.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_walk_headless_t0_333333_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.33333334, None);
    let cpp = read_pose(&golden_path("spineboy_run_to_walk_headless_t0_333333.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_walk_headless_t0_333333_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.33333334, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_to_walk_headless_t0_333333.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_walk_headless_t0_416667_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..15 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.4166667, None);
    let cpp = read_pose(&golden_path("spineboy_run_to_walk_headless_t0_416667.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_walk_headless_t0_416667_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..15 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.4166667, None);
    let cpp = read_pose(&golden_skel_path(
        "spineboy_run_to_walk_headless_t0_416667.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_spineboy_run_to_walk_headless_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..20 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_path("spineboy_run_to_walk_headless_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_walk_headless_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    for _ in 0..10 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    state.set_animation(0, "walk", true);
    for _ in 0..20 {
        step(&mut state, &mut skeleton, 1.0 / 60.0);
    }

    let rust = dump_pose(&skeleton, 0.5, None);
    let cpp = read_pose(&golden_skel_path("spineboy_run_to_walk_headless_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_spineboy_run_to_walk_mix0_2_t0_55_matches_cpp() {
    let data = load_data(&example_json_path("spineboy/export/spineboy-pro.skel"));

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("run", "walk", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "run", true);
    step(&mut state, &mut skeleton, 0.3);

    state.set_animation(0, "walk", true);
    step(&mut state, &mut skeleton, 0.25);

    let rust = dump_pose(&skeleton, 0.55, None);
    let cpp = read_pose(&golden_skel_path("spineboy_run_to_walk_mix0_2_t0_55.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_update_to_pose_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_update_to_pose_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_jitter_dt_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    for _ in 0..10 {
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.008_333_334);
        }
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.033_333_336);
        }
        for _ in 0..35 {
            step_physics(&mut state, &mut skeleton, 0.016_666_668);
        }
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_physics_none_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::None);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("sack_walk_physics_none_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("sack_walk_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("sack_walk_physics_jitter_dt_t1_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "sack_walk_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_to_hello_mix0_2_physics_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("walk", "hello", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step_physics(&mut state, &mut skeleton, 0.3);

    state.set_animation(0, "hello", true);
    step_physics(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path("sack_walk_to_hello_mix0_2_physics_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_plus_hello_add_physics_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step_physics(&mut state, &mut skeleton, 0.0);
    step_physics(&mut state, &mut skeleton, 0.2);

    let hello = state.set_animation(1, "hello", true);
    hello.set_additive(&mut state, true);
    step_physics(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_path("sack_walk_plus_hello_add_physics_t0_4.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_sack_walk_plus_hello_add_to_empty_mix0_2_physics_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step_physics(&mut state, &mut skeleton, 0.0);
    step_physics(&mut state, &mut skeleton, 0.2);

    let hello = state.set_animation(1, "hello", true);
    hello.set_additive(&mut state, true);
    step_physics(&mut state, &mut skeleton, 0.2);

    state.set_empty_animation(1, 0.2);
    step_physics(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_path(
        "sack_walk_plus_hello_add_to_empty_mix0_2_physics_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_celestial_circus_wind_idle_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("celestial_circus_wind_idle_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_celestial_circus_wind_idle_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "celestial_circus_wind_idle_physics_jitter_dt_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_celestial_circus_wind_idle_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "celestial_circus_wind_idle_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_celestial_circus_wind_idle_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "celestial_circus_wind_idle_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_celestial_circus_wind_idle_physics_t10_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.json",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "celestial_circus_wind_idle_physics_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("snowglobe_idle_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("snowglobe_idle_physics_jitter_dt_t1_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "snowglobe_idle_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "snowglobe_idle_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_physics_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("snowglobe_idle_physics_t10_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_physics_jitter_dt_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.008_333_334);
        }
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.033_333_336);
        }
        for _ in 0..35 {
            step_physics(&mut state, &mut skeleton, 0.016_666_668);
        }
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path("snowglobe_idle_physics_jitter_dt_t10_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..18 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let shake = state.set_animation(1, "shake", false);
    shake.set_additive(&mut state, true);
    for _ in 0..6 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..12 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
fn oracle_snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.json"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    // Phase1 ~0.3s.
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..5 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shake = state.set_animation(1, "shake", false);
    shake.set_additive(&mut state, true);

    // Phase2 ~0.1s.
    for _ in 0..4 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..2 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }

    state.set_empty_animation(1, 0.2);

    // Phase3 ~0.2s.
    for _ in 0..6 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..3 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..3 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_path(
        "snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_update_to_pose_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_update_to_pose_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_jitter_dt_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("cloud-pot/export/cloud-pot.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "playing-in-the-rain", true);

    for _ in 0..10 {
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.008_333_334);
        }
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.033_333_336);
        }
        for _ in 0..35 {
            step_physics(&mut state, &mut skeleton, 0.016_666_668);
        }
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_sack_walk_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path("sack_walk_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_sack_walk_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path("sack_walk_physics_jitter_dt_t1_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_sack_walk_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "sack_walk_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_sack_walk_to_hello_mix0_2_physics_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.skel"));
    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("walk", "hello", 0.2);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(state_data);

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step_physics(&mut state, &mut skeleton, 0.3);

    state.set_animation(0, "hello", true);
    step_physics(&mut state, &mut skeleton, 0.1);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "sack_walk_to_hello_mix0_2_physics_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_sack_walk_plus_hello_add_physics_t0_4_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step_physics(&mut state, &mut skeleton, 0.0);
    step_physics(&mut state, &mut skeleton, 0.2);

    let hello = state.set_animation(1, "hello", true);
    hello.set_additive(&mut state, true);
    step_physics(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.4, None);
    let cpp = read_pose(&golden_skel_path(
        "sack_walk_plus_hello_add_physics_t0_4.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_sack_walk_plus_hello_add_to_empty_mix0_2_physics_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("7-anticipation/export/sack-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "walk", true);
    step_physics(&mut state, &mut skeleton, 0.0);
    step_physics(&mut state, &mut skeleton, 0.2);

    let hello = state.set_animation(1, "hello", true);
    hello.set_additive(&mut state, true);
    step_physics(&mut state, &mut skeleton, 0.2);

    state.set_empty_animation(1, 0.2);
    step_physics(&mut state, &mut skeleton, 0.2);

    let rust = dump_pose(&skeleton, 0.6, None);
    let cpp = read_pose(&golden_skel_path(
        "sack_walk_plus_hello_add_to_empty_mix0_2_physics_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_celestial_circus_wind_idle_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "celestial_circus_wind_idle_physics_t0_5.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_celestial_circus_wind_idle_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "celestial_circus_wind_idle_physics_jitter_dt_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_celestial_circus_wind_idle_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "celestial_circus_wind_idle_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_celestial_circus_wind_idle_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "celestial_circus_wind_idle_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_celestial_circus_wind_idle_physics_t10_0_matches_cpp() {
    let data = load_data(&example_json_path(
        "celestial-circus/export/celestial-circus-pro.skel",
    ));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "wind-idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "celestial_circus_wind_idle_physics_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_physics_t0_5_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path("snowglobe_idle_physics_t0_5.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_physics_jitter_dt_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..35 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "snowglobe_idle_physics_jitter_dt_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_physics_update_pose_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Pose);
    }
    for _ in 0..15 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "snowglobe_idle_physics_update_pose_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_physics_update_reset_update_t1_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }
    step_with_physics(&mut state, &mut skeleton, 0.0, crate::Physics::Reset);
    for _ in 0..30 {
        step_with_physics(&mut state, &mut skeleton, dt, crate::Physics::Update);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "snowglobe_idle_physics_update_reset_update_t1_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_physics_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path("snowglobe_idle_physics_t10_0.json"));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_physics_jitter_dt_t10_0_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));
    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.008_333_334);
        }
        for _ in 0..10 {
            step_physics(&mut state, &mut skeleton, 0.033_333_336);
        }
        for _ in 0..35 {
            step_physics(&mut state, &mut skeleton, 0.016_666_668);
        }
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "snowglobe_idle_physics_jitter_dt_t10_0.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    let dt = 1.0 / 60.0;
    for _ in 0..18 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let shake = state.set_animation(1, "shake", false);
    shake.set_additive(&mut state, true);
    for _ in 0..6 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    state.set_empty_animation(1, 0.2);
    for _ in 0..12 {
        step_physics(&mut state, &mut skeleton, dt);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}

#[test]
#[cfg(all(feature = "binary", feature = "upstream-smoke"))]
fn oracle_skel_snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6_matches_cpp() {
    let data = load_data(&example_json_path("snowglobe/export/snowglobe-pro.skel"));

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    skeleton.setup_pose();
    state.set_animation(0, "idle", true);

    for _ in 0..10 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..4 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..5 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let shake = state.set_animation(1, "shake", false);
    shake.set_additive(&mut state, true);

    for _ in 0..4 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..2 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }

    state.set_empty_animation(1, 0.2);

    for _ in 0..6 {
        step_physics(&mut state, &mut skeleton, 0.008_333_334);
    }
    for _ in 0..3 {
        step_physics(&mut state, &mut skeleton, 0.033_333_336);
    }
    for _ in 0..3 {
        step_physics(&mut state, &mut skeleton, 0.016_666_668);
    }

    let rust = dump_pose(&skeleton, skeleton.get_time(), None);
    let cpp = read_pose(&golden_skel_path(
        "snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6.json",
    ));
    assert_pose_parity(&rust, &cpp, 1.0e-3);
}
