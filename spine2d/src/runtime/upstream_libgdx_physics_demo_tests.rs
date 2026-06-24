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

fn load_skel_with_scale(relative: &str, scale: f32) -> Arc<SkeletonData> {
    let path = example_skel_path(relative);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    SkeletonData::from_skel_bytes_with_scale(&bytes, scale)
        .unwrap_or_else(|e| panic!("parse {path:?} scale={scale}: {e}"))
}

fn run_for_frames(
    example: &str,
    skeleton: &mut Skeleton,
    state: &mut AnimationState,
    frames: usize,
) {
    let dt = 1.0 / 60.0;
    for _ in 0..frames {
        state.update(dt);
        state.apply(skeleton);
        skeleton.update(dt);
        skeleton.update_world_transform_with_physics(Physics::Update);
        assert_skeleton_world_finite(example, skeleton);
    }
}

#[test]
fn physics_test2_celestial_circus_skel_scale_0_1_smoke() {
    // Port of `spine-libgdx` `PhysicsTest2.java` (headless):
    // - loads `.skel` with `SkeletonBinary.setScale(0.1f)`
    // - applies world transform with physics update.
    let data = load_skel_with_scale("celestial-circus/export/celestial-circus-pro.skel", 0.1);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.x = 320.0;
    skeleton.y = 100.0;
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(Physics::Update);

    let mut state = AnimationState::new(AnimationStateData::new(data));
    run_for_frames("celestial-circus", &mut skeleton, &mut state, 120);
}

#[test]
fn physics_test3_snowglobe_shake_skel_scale_0_15_smoke() {
    // Port of `spine-libgdx` `PhysicsTest3.java` (headless).
    let data = load_skel_with_scale("snowglobe/export/snowglobe-pro.skel", 0.15);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.x = 320.0;
    skeleton.y = 100.0;
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(Physics::Update);

    let mut state = AnimationState::new(AnimationStateData::new(data));
    state.set_animation(0, "shake", true);
    run_for_frames("snowglobe", &mut skeleton, &mut state, 240);
}

#[test]
fn physics_test4_cloud_pot_skel_scale_0_15_smoke() {
    // Port of `spine-libgdx` `PhysicsTest4.java` (headless).
    let data = load_skel_with_scale("cloud-pot/export/cloud-pot.skel", 0.15);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.x = 320.0;
    skeleton.y = 100.0;
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(Physics::Update);

    let mut state = AnimationState::new(AnimationStateData::new(data));
    state.set_animation(0, "playing-in-the-rain", true);
    run_for_frames("cloud-pot", &mut skeleton, &mut state, 240);
}
