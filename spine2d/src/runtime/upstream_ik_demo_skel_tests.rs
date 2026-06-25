use crate::runtime::{AnimationState, AnimationStateData, Physics};
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
        "Upstream Spine examples not found. Run `python3 ./scripts/prepare_spine_runtimes_web_assets.py --scope tests` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn example_skel_path(relative: &str) -> PathBuf {
    upstream_examples_root().join(relative)
}

fn bone_index(data: &SkeletonData, name: &str) -> usize {
    data.bones
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| panic!("missing bone: {name}"))
}

fn assert_approx(label: &str, actual: f32, expected: f32) {
    let eps = 1e-3;
    let diff = (actual - expected).abs();
    assert!(
        diff <= eps,
        "{label}: expected {expected}, got {actual} (diff {diff}, eps {eps})"
    );
}

fn load_skel_with_scale(relative: &str, scale: f32) -> Arc<SkeletonData> {
    let path = example_skel_path(relative);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    SkeletonData::from_skel_bytes_with_scale(&bytes, scale)
        .unwrap_or_else(|e| panic!("parse {path:?} scale={scale}: {e}"))
}

#[test]
fn ik_test_crosshair_parent_world_to_local_matches_upstream_demo_flow_skel() {
    let data = load_skel_with_scale("spineboy/export/spineboy-pro.skel", 0.6);

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.x = 250.0;
    skeleton.y = 20.0;

    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    state.set_animation(0, "walk", true);
    state.set_animation(1, "aim", true);

    let dt = 1.0 / 60.0;
    state.update(dt);
    state.apply(&mut skeleton);
    skeleton.update(dt);
    skeleton.update_world_transform_with_physics(Physics::Pose);

    let crosshair = bone_index(&data, "crosshair");
    let parent = skeleton.bones[crosshair].parent.expect("crosshair parent");

    let target_world_x = 320.0;
    let target_world_y = 240.0;
    let (local_x, local_y) = skeleton.bones[parent].world_to_local(target_world_x, target_world_y);
    skeleton.bones[crosshair].x = local_x;
    skeleton.bones[crosshair].y = local_y;

    skeleton.update_world_transform_with_physics(Physics::Update);

    assert_approx(
        "crosshair.world_x",
        skeleton.bones[crosshair].world_x,
        target_world_x,
    );
    assert_approx(
        "crosshair.world_y",
        skeleton.bones[crosshair].world_y,
        target_world_y,
    );
}
