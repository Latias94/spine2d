use crate::runtime::{AnimationState, AnimationStateData};
use crate::{Skeleton, SkeletonData};
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
        "Upstream Spine examples not found. Run `python3 ./scripts/prepare_spine_runtimes_web_assets.py --scope tests` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn assert_skeleton_world_finite(example: &str, skeleton: &Skeleton) {
    for (i, bone) in skeleton.bones.iter().enumerate() {
        let ok = bone.x.is_finite()
            && bone.y.is_finite()
            && bone.rotation.is_finite()
            && bone.scale_x.is_finite()
            && bone.scale_y.is_finite()
            && bone.shear_x.is_finite()
            && bone.shear_y.is_finite()
            && bone.a.is_finite()
            && bone.b.is_finite()
            && bone.c.is_finite()
            && bone.d.is_finite()
            && bone.world_x.is_finite()
            && bone.world_y.is_finite();
        assert!(
            ok,
            "non-finite bone transform: example={example:?} bone_index={i} bone_data_index={} world=({}, {}) matrix=[{}, {}, {}, {}] local=({}, {})",
            bone.data_index(),
            bone.world_x,
            bone.world_y,
            bone.a,
            bone.b,
            bone.c,
            bone.d,
            bone.x,
            bone.y
        );
    }
}

fn pick_preferred_skel_in_export_dir(export_dir: &Path) -> Option<PathBuf> {
    let mut skels: Vec<PathBuf> = std::fs::read_dir(export_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("skel"))
        .collect();
    skels.sort();

    skels
        .iter()
        .find(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with("-pro.skel"))
        })
        .cloned()
        .or_else(|| skels.first().cloned())
}

fn read_skel_spine_version_prefix(bytes: &[u8]) -> Option<String> {
    // Binary header is:
    // - hash: 8 bytes (2x int32 big-endian)
    // - spineVersion: string (varint length + utf8 bytes)
    let mut cursor = 8usize;
    if bytes.len() < cursor + 1 {
        return None;
    }

    let mut value: u32 = 0;
    let mut shift = 0u32;
    loop {
        let b = *bytes.get(cursor)? as u32;
        cursor += 1;
        value |= (b & 0x7f) << shift;
        if (b & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            return None;
        }
    }

    let length = value as usize;
    if length == 0 {
        return None;
    }
    if length == 1 {
        return Some(String::new());
    }
    let byte_len = length - 1;
    let s = std::str::from_utf8(bytes.get(cursor..cursor + byte_len)?).ok()?;
    Some(s.to_string())
}

fn run_each_animation_sample_smoke(path: &Path, example_label: &str) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    // This suite is intended to exercise the current pinned 4.3 baseline exports.
    if read_skel_spine_version_prefix(&bytes).is_some_and(|v| !v.starts_with("4.3")) {
        return;
    }
    let data: Arc<SkeletonData> =
        SkeletonData::from_skel_bytes(&bytes).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));

    let animations = data
        .animations
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();
    if animations.is_empty() {
        return;
    }

    for anim in animations {
        let mut skeleton = Skeleton::new(data.clone());
        skeleton.setup_pose();
        skeleton.update_world_transform_with_physics(crate::Physics::Update);

        let mut state_data = AnimationStateData::new(data.clone());
        state_data.set_default_mix(0.2);
        let mut state = AnimationState::new(state_data);
        state
            .set_animation(0, &anim, true)
            .unwrap_or_else(|e| panic!("set animation {anim} ({example_label}): {e}"));

        for _ in 0..120 {
            let dt = 1.0 / 60.0;
            state.update(dt);
            state.apply(&mut skeleton);
            skeleton.update(dt);
            skeleton.update_world_transform_with_physics(crate::Physics::Update);
            assert_skeleton_world_finite(example_label, &skeleton);
        }
    }
}

fn run_queued_animations_smoke(data: Arc<SkeletonData>, example_label: &str) {
    const MAX_ANIMS: usize = 6;
    const MAX_FRAMES: usize = 1800; // 30s at 60fps
    let dt = 1.0 / 60.0;

    let animations = data
        .animations
        .iter()
        .map(|a| a.name.clone())
        .take(MAX_ANIMS)
        .collect::<Vec<_>>();
    if animations.is_empty() {
        return;
    }

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::Update);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_default_mix(0.2);
    let mut state = AnimationState::new(state_data);

    state
        .set_animation(0, &animations[0], false)
        .unwrap_or_else(|e| panic!("set animation {} ({example_label}): {e}", animations[0]));
    for name in animations.iter().skip(1) {
        state
            .add_animation(0, name, false, 0.0)
            .unwrap_or_else(|e| panic!("add animation {name} ({example_label}): {e}"));
    }

    for _ in 0..MAX_FRAMES {
        if state.current(0).is_none() {
            break;
        }
        state.update(dt);
        state.apply(&mut skeleton);
        skeleton.update(dt);
        skeleton.update_world_transform_with_physics(crate::Physics::Update);
        assert_skeleton_world_finite(example_label, &skeleton);
    }
}

fn run_multitrack_overlay_smoke(data: Arc<SkeletonData>, example_label: &str) {
    const MAX_FRAMES: usize = 240; // 4s at 60fps
    let dt = 1.0 / 60.0;

    let animations = data
        .animations
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();
    if animations.len() < 2 {
        return;
    }

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::Update);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_default_mix(0.2);
    let mut state = AnimationState::new(state_data);

    state
        .set_animation(0, &animations[0], true)
        .unwrap_or_else(|e| panic!("set track0 {} ({example_label}): {e}", animations[0]));
    state
        .set_animation(1, &animations[1], true)
        .unwrap_or_else(|e| panic!("set track1 {} ({example_label}): {e}", animations[1]));

    if let Some(name) = animations.get(2) {
        let e = state
            .set_animation(2, name, true)
            .unwrap_or_else(|e| panic!("set track2 {name} ({example_label}): {e}"));
        e.set_mix_blend(&mut state, crate::MixBlend::Add);
        e.set_alpha(&mut state, 0.5);
    }

    for _ in 0..MAX_FRAMES {
        state.update(dt);
        state.apply(&mut skeleton);
        skeleton.update(dt);
        skeleton.update_world_transform_with_physics(crate::Physics::Update);
        assert_skeleton_world_finite(example_label, &skeleton);
    }
}

#[test]
fn upstream_examples_tests_scope_skel_sample_smoke_all_examples() {
    let examples_root = upstream_examples_root();
    let mut example_dirs: Vec<PathBuf> = std::fs::read_dir(&examples_root)
        .expect("read examples dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    example_dirs.sort();

    for example_dir in example_dirs {
        let export_dir = example_dir.join("export");
        if !export_dir.is_dir() {
            continue;
        }
        let Some(skel_path) = pick_preferred_skel_in_export_dir(&export_dir) else {
            continue;
        };

        let example_name = example_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>");

        run_each_animation_sample_smoke(&skel_path, example_name);
    }
}

#[test]
fn upstream_examples_tests_scope_skel_queue_smoke_all_examples() {
    let examples_root = upstream_examples_root();
    let mut example_dirs: Vec<PathBuf> = std::fs::read_dir(&examples_root)
        .expect("read examples dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    example_dirs.sort();

    for example_dir in example_dirs {
        let export_dir = example_dir.join("export");
        if !export_dir.is_dir() {
            continue;
        }
        let Some(skel_path) = pick_preferred_skel_in_export_dir(&export_dir) else {
            continue;
        };

        let example_name = example_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>");

        let bytes = std::fs::read(&skel_path).unwrap_or_else(|e| panic!("read {skel_path:?}: {e}"));
        if read_skel_spine_version_prefix(&bytes).is_some_and(|v| !v.starts_with("4.3")) {
            continue;
        }
        let data: Arc<SkeletonData> = SkeletonData::from_skel_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {skel_path:?}: {e}"));

        run_queued_animations_smoke(data.clone(), example_name);
        run_multitrack_overlay_smoke(data, example_name);
    }
}
