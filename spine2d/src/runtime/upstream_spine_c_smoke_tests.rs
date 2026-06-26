use crate::runtime::{AnimationState, AnimationStateData};
use crate::{Skeleton, SkeletonData};
use std::path::PathBuf;
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
    upstream_examples_root().join(relative)
}

fn assert_skeleton_world_finite(relative: &str, skeleton: &Skeleton) {
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
            "non-finite bone transform: example={relative:?} bone_index={i} bone_data_index={} world=({}, {}) matrix=[{}, {}, {}, {}] local=({}, {})",
            bone.data_index,
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

fn run_all_animations_queue_smoke(relative: &str) {
    let path = example_json_path(relative);
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::Update);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_default_mix(0.2); // matches upstream spine-c-unit-tests
    let mut state = AnimationState::new(state_data);

    let animations = data
        .animations
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();
    if animations.is_empty() {
        return;
    }

    state.set_animation(0, animations[0].as_str(), false);
    for name in animations.iter().skip(1) {
        state.add_animation(0, name, false, 0.0);
    }

    const MAX_RUN_TIME: usize = 6000; // matches upstream (about 100s at 60fps)
    for _ in 0..MAX_RUN_TIME {
        if state.get_current(0).is_none() {
            break;
        }
        let dt = 1.0 / 60.0;
        state.update(dt);
        state.apply(&mut skeleton);
        skeleton.update(dt);
        skeleton.update_world_transform_with_physics(crate::Physics::Update);
        assert_skeleton_world_finite(relative, &skeleton);
    }
}

#[test]
fn upstream_spine_c_interface_smoke_spineboy() {
    run_all_animations_queue_smoke("spineboy/export/spineboy-ess.json");
}

#[test]
fn upstream_spine_c_interface_smoke_raptor() {
    run_all_animations_queue_smoke("raptor/export/raptor-pro.json");
}

#[test]
fn upstream_spine_c_interface_smoke_goblins() {
    run_all_animations_queue_smoke("goblins/export/goblins-pro.json");
}

fn pick_preferred_json_in_export_dir(export_dir: &std::path::Path) -> Option<PathBuf> {
    let mut jsons: Vec<PathBuf> = std::fs::read_dir(export_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    jsons.sort();

    jsons
        .iter()
        .find(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with("-pro.json"))
        })
        .cloned()
        .or_else(|| jsons.first().cloned())
}

fn read_json_spine_version(json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()?
        .get("skeleton")?
        .get("spine")?
        .as_str()
        .map(str::to_string)
}

fn run_each_animation_sample_smoke(path: &std::path::Path, example_label: &str) {
    let json = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    // This suite is intended to exercise the current pinned 4.3 baseline exports.
    if read_json_spine_version(&json).is_some_and(|v| !v.starts_with("4.3")) {
        return;
    }
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));

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
        state.set_animation(0, anim.as_str(), true);

        // Sample a short time window for each animation to catch parsing/runtime issues without
        // turning this into a long-running suite.
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
    // Keep bounded: this is a "tests scope" smoke, not a full stress test.
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

    state.set_animation(0, animations[0].as_str(), false);
    for name in animations.iter().skip(1) {
        state.add_animation(0, name, false, 0.0);
    }

    for _ in 0..MAX_FRAMES {
        if state.get_current(0).is_none() {
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
    // Cross-track overlay smoke: exercises property gating and blending across tracks without
    // assuming any particular animation names.
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

    state.set_animation(0, animations[0].as_str(), true);
    state.set_animation(1, animations[1].as_str(), true);

    // If we have a third animation, add it as an additive overlay at half alpha.
    // cross-track paths.
    if let Some(name) = animations.get(2) {
        let e = state.set_animation(2, name, true);
        e.set_additive(&mut state, true);
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
fn upstream_examples_tests_scope_json_sample_smoke_all_examples() {
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
        let Some(json_path) = pick_preferred_json_in_export_dir(&export_dir) else {
            continue;
        };

        let example_name = example_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>");

        run_each_animation_sample_smoke(&json_path, example_name);
    }
}

#[test]
fn upstream_examples_tests_scope_json_queue_smoke_all_examples() {
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
        let Some(json_path) = pick_preferred_json_in_export_dir(&export_dir) else {
            continue;
        };

        let example_name = example_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>");

        let json = std::fs::read_to_string(&json_path)
            .unwrap_or_else(|e| panic!("read {json_path:?}: {e}"));
        if read_json_spine_version(&json).is_some_and(|v| !v.starts_with("4.3")) {
            continue;
        }
        let data: Arc<SkeletonData> = SkeletonData::from_json_str(&json)
            .unwrap_or_else(|e| panic!("parse {json_path:?}: {e}"));

        run_queued_animations_smoke(data.clone(), example_name);
        run_multitrack_overlay_smoke(data, example_name);
    }
}
