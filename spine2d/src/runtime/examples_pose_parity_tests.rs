#![allow(clippy::excessive_precision)]

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

fn assert_approx(actual: f32, expected: f32) {
    let eps = 1.0e-3;
    let diff = (actual - expected).abs();
    assert!(
        diff <= eps,
        "expected {expected}, got {actual} (diff {diff}, eps {eps})"
    );
}

fn bone_index(data: &SkeletonData, name: &str) -> usize {
    data.bones
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| panic!("missing bone: {name}"))
}

struct BoneExpected {
    name: &'static str,
    world: [f32; 6],   // a,b,c,d,x,y
    applied: [f32; 7], // x,y,rotation,scaleX,scaleY,shearX,shearY
}

struct SpineboyHarness {
    data: Arc<SkeletonData>,
    skeleton: Skeleton,
    state: AnimationState,
}

impl SpineboyHarness {
    fn new(configure_state_data: impl FnOnce(&mut AnimationStateData)) -> Self {
        let path = example_json_path("spineboy/export/spineboy-pro.json");
        let json = std::fs::read_to_string(&path).expect("read spineboy-pro.json");
        let data: Arc<SkeletonData> =
            SkeletonData::from_json_str(&json).expect("parse spineboy-pro.json");

        let mut state_data = AnimationStateData::new(data.clone());
        configure_state_data(&mut state_data);

        let skeleton = Skeleton::new(data.clone());
        let state = AnimationState::new(state_data);
        Self {
            data,
            skeleton,
            state,
        }
    }

    fn step(&mut self, dt: f32) {
        self.state.update(dt);
        self.state.apply(&mut self.skeleton);
        self.skeleton.update_world_transform();
    }
}

fn assert_spineboy_pose(h: &SpineboyHarness, expected: &[BoneExpected]) {
    for b in expected {
        let i = bone_index(&h.data, b.name);
        let bone = &h.skeleton.bones[i];
        assert_approx(bone.a, b.world[0]);
        assert_approx(bone.b, b.world[1]);
        assert_approx(bone.c, b.world[2]);
        assert_approx(bone.d, b.world[3]);
        assert_approx(bone.world_x, b.world[4]);
        assert_approx(bone.world_y, b.world[5]);

        assert_approx(bone.ax, b.applied[0]);
        assert_approx(bone.ay, b.applied[1]);
        assert_approx(bone.arotation, b.applied[2]);
        assert_approx(bone.ascale_x, b.applied[3]);
        assert_approx(bone.ascale_y, b.applied[4]);
        assert_approx(bone.ashear_x, b.applied[5]);
        assert_approx(bone.ashear_y, b.applied[6]);
    }
}

fn slot_index(data: &SkeletonData, name: &str) -> usize {
    data.slots
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("missing slot: {name}"))
}

fn assert_slot_attachment(h: &SpineboyHarness, slot_name: &str, expected: Option<&str>) {
    let i = slot_index(&h.data, slot_name);
    let slot = &h.skeleton.slots[i];
    assert_eq!(slot.applied_attachment_name(), expected, "slot {slot_name}");
}

fn assert_slot_color_approx(h: &SpineboyHarness, slot_name: &str, expected: [f32; 4]) {
    let i = slot_index(&h.data, slot_name);
    let slot = &h.skeleton.slots[i];
    let eps = 1.0e-3;
    for (j, label) in ["r", "g", "b", "a"].into_iter().enumerate() {
        let actual = slot.applied_color()[j];
        let exp = expected[j];
        let diff = (actual - exp).abs();
        assert!(
            diff <= eps,
            "slot {slot_name} color.{label}: expected {exp}, got {actual} (diff {diff}, eps {eps})"
        );
    }
}

fn assert_spineboy_run_pose(time: f32, expected: &[BoneExpected]) {
    let path = example_json_path("spineboy/export/spineboy-pro.json");
    let json = std::fs::read_to_string(&path).expect("read spineboy-pro.json");
    let data: Arc<SkeletonData> =
        SkeletonData::from_json_str(&json).expect("parse spineboy-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));

    state.set_animation(0, "run", true).expect("set run");
    state.update(time);

    skeleton.setup_pose();
    state.apply(&mut skeleton);
    skeleton.update_world_transform();

    for b in expected {
        let i = bone_index(&data, b.name);
        let bone = &skeleton.bones[i];
        assert_approx(bone.a, b.world[0]);
        assert_approx(bone.b, b.world[1]);
        assert_approx(bone.c, b.world[2]);
        assert_approx(bone.d, b.world[3]);
        assert_approx(bone.world_x, b.world[4]);
        assert_approx(bone.world_y, b.world[5]);

        assert_approx(bone.ax, b.applied[0]);
        assert_approx(bone.ay, b.applied[1]);
        assert_approx(bone.arotation, b.applied[2]);
        assert_approx(bone.ascale_x, b.applied[3]);
        assert_approx(bone.ascale_y, b.applied[4]);
        assert_approx(bone.ashear_x, b.applied[5]);
        assert_approx(bone.ashear_y, b.applied[6]);
    }
}

fn assert_tank_drive_treads_pose(time: f32, expected: &[BoneExpected], expected_position: f32) {
    let path = example_json_path("tank/export/tank-pro.json");
    let json = std::fs::read_to_string(&path).expect("read tank-pro.json");
    let data: Arc<SkeletonData> = SkeletonData::from_json_str(&json).expect("parse tank-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));

    state.set_animation(0, "drive", true).expect("set drive");
    state.update(time);

    skeleton.setup_pose();
    state.apply(&mut skeleton);
    skeleton.update_world_transform();

    assert!(!skeleton.path_constraints.is_empty());
    assert_approx(skeleton.path_constraints[0].position, expected_position);

    for b in expected {
        let i = bone_index(&data, b.name);
        let bone = &skeleton.bones[i];
        assert_approx(bone.a, b.world[0]);
        assert_approx(bone.b, b.world[1]);
        assert_approx(bone.c, b.world[2]);
        assert_approx(bone.d, b.world[3]);
        assert_approx(bone.world_x, b.world[4]);
        assert_approx(bone.world_y, b.world[5]);

        assert_approx(bone.ax, b.applied[0]);
        assert_approx(bone.ay, b.applied[1]);
        assert_approx(bone.arotation, b.applied[2]);
        assert_approx(bone.ascale_x, b.applied[3]);
        assert_approx(bone.ascale_y, b.applied[4]);
        assert_approx(bone.ashear_x, b.applied[5]);
        assert_approx(bone.ashear_y, b.applied[6]);
    }
}

// Expected values are generated from the official Spine 4.3 C++ runtime (oracle).
// See `scripts/spine_cpp_lite_oracle.cpp`.
#[test]
fn example_spineboy_run_pose_matches_spine_cpp_lite_0p1() {
    assert_spineboy_run_pose(
        0.1,
        &[
            BoneExpected {
                name: "root",
                world: [
                    0.999999642,
                    -0.000833316531,
                    0.000833302969,
                    0.999999642,
                    0.0,
                    0.0,
                ],
                applied: [0.0, 0.0, 0.047744751, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "hip",
                world: [
                    0.989800155,
                    0.142462432,
                    -0.142462417,
                    0.989800155,
                    3.49312663,
                    250.160645,
                ],
                applied: [3.70158839, 250.157654, -8.23810577, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "torso",
                world: [
                    0.511808395,
                    -0.859099567,
                    0.859099567,
                    0.511808395,
                    2.59076142,
                    255.245178,
                ],
                applied: [-1.61751556, 4.90411377, 67.4059982, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "head",
                world: [
                    0.265700042,
                    -0.97528851,
                    0.954349458,
                    0.271529734,
                    83.1404419,
                    385.17218,
                ],
                applied: [
                    27.663269,
                    -0.259902954,
                    35.5978546,
                    0.990645945,
                    1.01238132,
                    0.0,
                    0.0,
                ],
            },
        ],
    );
}

#[test]
fn example_spineboy_run_pose_matches_spine_cpp_lite_0p5() {
    assert_spineboy_run_pose(
        0.5,
        &[
            BoneExpected {
                name: "root",
                world: [
                    0.999999642,
                    -0.000833316531,
                    0.000833302969,
                    0.999999642,
                    0.0,
                    0.0,
                ],
                applied: [0.0, 0.0, 0.047744751, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "hip",
                world: [
                    0.989800155,
                    0.142462432,
                    -0.142462417,
                    0.989800155,
                    6.3931241,
                    264.288422,
                ],
                applied: [6.61335802, 264.28299, -8.23810577, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "torso",
                world: [
                    0.528350711,
                    -0.849026024,
                    0.849026203,
                    0.528350949,
                    5.4907589,
                    269.372955,
                ],
                applied: [-1.61751556, 4.90411377, 66.2962723, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "head",
                world: [
                    0.168877527,
                    -0.986394227,
                    0.989156604,
                    0.168406367,
                    86.2527542,
                    399.959534,
                ],
                applied: [
                    27.663269,
                    -0.259902954,
                    38.2701797,
                    1.00346911,
                    1.0006671,
                    0.0,
                    0.0,
                ],
            },
        ],
    );
}

#[test]
fn example_spineboy_run_pose_matches_spine_cpp_lite_0p65() {
    assert_spineboy_run_pose(
        0.65,
        &[
            BoneExpected {
                name: "root",
                world: [
                    0.999999642,
                    -0.000833316531,
                    0.000833302969,
                    0.999999642,
                    0.0,
                    0.0,
                ],
                applied: [0.0, 0.0, 0.047744751, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "hip",
                world: [
                    0.989800155,
                    0.142462432,
                    -0.142462417,
                    0.989800155,
                    -3.73198295,
                    213.659317,
                ],
                applied: [-3.55393577, 213.662354, -8.23810577, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "torso",
                world: [
                    0.534008086,
                    -0.845479369,
                    0.845479369,
                    0.534008026,
                    -4.63434792,
                    218.743851,
                ],
                applied: [-1.61751556, 4.90411377, 65.9136963, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "head",
                world: [
                    0.0586183332,
                    -0.995550334,
                    1.00280821,
                    0.0581938475,
                    75.3991699,
                    350.472595,
                ],
                applied: [
                    27.663269,
                    -0.259902954,
                    40.2303429,
                    1.00452006,
                    0.997249663,
                    0.0,
                    0.0,
                ],
            },
        ],
    );
}

#[test]
fn example_tank_drive_treads_pose_matches_spine_cpp_lite_0p3() {
    assert_tank_drive_treads_pose(
        0.3,
        &[
            BoneExpected {
                name: "tank-root",
                world: [1.0, -8.74227766e-08, 0.0, 1.0, -18.2047043, 146.787979],
                applied: [-18.2046986, 146.787979, 0.0, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "tread",
                world: [
                    -0.904994905,
                    0.412097305,
                    -0.409318,
                    -0.911139846,
                    -32.5711021,
                    343.286926,
                ],
                applied: [-22.8999443, 213.855621, 180.0, 0.993255734, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "tread14",
                world: [
                    0.995813906,
                    -0.00853618048,
                    0.00198456878,
                    0.997413993,
                    -414.542084,
                    11.7339478,
                ],
                applied: [509.189514, 153.304123, 142.013748, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "tread36",
                world: [
                    -0.93632549,
                    -0.333191663,
                    0.338643789,
                    -0.940295279,
                    37.8531876,
                    320.035217,
                ],
                applied: [-55.9900055, 60.9228821, -36.8190918, 1.0, 1.0, 0.0, 0.0],
            },
        ],
        0.00612481311,
    );
}

#[test]
fn example_tank_shoot_rgba2_slot_color_matches_spine_cpp_lite_0p3() {
    let path = example_json_path("tank/export/tank-pro.json");
    let json = std::fs::read_to_string(&path).expect("read tank-pro.json");
    let data: Arc<SkeletonData> = SkeletonData::from_json_str(&json).expect("parse tank-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));

    state.set_animation(0, "shoot", false).expect("set shoot");
    state.update(0.3);

    skeleton.setup_pose();
    state.apply(&mut skeleton);

    let i = slot_index(&data, "smoke-puff1-bg");
    let slot = &skeleton.slots[i];
    assert!(slot.applied_has_dark());

    for (j, exp) in [1.0, 0.835294, 0.0470588, 1.0].into_iter().enumerate() {
        assert_approx(slot.applied_color()[j], exp);
    }
    for (j, exp) in [0.376471, 0.294118, 0.247059].into_iter().enumerate() {
        assert_approx(slot.applied_dark_color()[j], exp);
    }
}

// Expected values are generated from the official Spine 4.3 C++ runtime (oracle).
// Scenario:
//   mix(run->walk)=0.2
//   set 0 run (loop)
//   step 0.3
//   set 0 walk (loop)
//   step 0.1
#[test]
fn example_spineboy_run_to_walk_mixing_mid_pose_matches_spine_cpp_lite() {
    let mut h = SpineboyHarness::new(|state_data| {
        state_data.set_mix("run", "walk", 0.2).expect("set mix");
    });
    h.skeleton.setup_pose();

    h.state.set_animation(0, "run", true).expect("set run");
    h.step(0.3);

    h.state.set_animation(0, "walk", true).expect("set walk");
    h.step(0.1);

    assert_spineboy_pose(
        &h,
        &[
            BoneExpected {
                name: "root",
                world: [
                    0.999999642,
                    -0.000833316531,
                    0.000833302969,
                    0.999999642,
                    0.0,
                    0.0,
                ],
                applied: [0.0, 0.0, 0.047744751, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "hip",
                world: [
                    0.994058967,
                    0.108842477,
                    -0.108842544,
                    0.994058967,
                    -2.07197142,
                    228.463211,
                ],
                applied: [-1.88158858, 228.464859, -6.2963419, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "torso",
                world: [
                    0.353503853,
                    -0.93543303,
                    0.935432971,
                    0.353503883,
                    -3.14610147,
                    233.514236,
                ],
                applied: [-1.61751556, 4.90411377, 75.546814, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "head",
                world: [
                    0.179048091,
                    -0.996590197,
                    0.974885881,
                    0.183034569,
                    53.8549385,
                    376.601807,
                ],
                applied: [
                    27.663269,
                    -0.259902954,
                    25.3036976,
                    0.991191566,
                    1.01325905,
                    0.0,
                    0.0,
                ],
            },
            BoneExpected {
                name: "front-thigh",
                world: [
                    0.926036239,
                    0.377434552,
                    -0.377434611,
                    0.926036239,
                    -10.0390034,
                    219.570786,
                ],
                applied: [-6.95182657, -9.70674801, -15.9262695, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "front-shin",
                world: [
                    -0.326730013,
                    0.945117652,
                    -0.945117712,
                    -0.326730132,
                    63.4354477,
                    191.353653,
                ],
                applied: [78.6901245, 1.60170746, -86.8955612, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "front-foot",
                world: [
                    0.384077519,
                    0.923300743,
                    -0.923300922,
                    0.384077489,
                    21.0463867,
                    69.7752228,
                ],
                applied: [128.755707, -0.339328766, 41.6569061, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-thigh",
                world: [
                    -0.452343047,
                    0.891843915,
                    -0.891843915,
                    -0.452343196,
                    -11.4685526,
                    221.98468,
                ],
                applied: [-8.63562012, -7.46277905, -110.645523, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-shin",
                world: [
                    -0.46644792,
                    0.884548545,
                    -0.884548604,
                    -0.466448128,
                    -51.5972786,
                    145.796417,
                ],
                applied: [86.0999985, -1.32533264, -0.909856796, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-foot",
                world: [
                    -0.75257051,
                    0.658511579,
                    -0.658511519,
                    -0.752570689,
                    -108.919601,
                    38.7122498,
                ],
                applied: [121.459038, -0.755193472, -21.0095711, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "gun",
                world: [
                    0.144585937,
                    0.989492059,
                    -0.989492178,
                    0.144585967,
                    57.4748459,
                    237.130341,
                ],
                applied: [34.4218864, -0.453979492, -21.1265411, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "gun-tip",
                world: [
                    0.265777081,
                    0.964034379,
                    -0.964034438,
                    0.265777051,
                    138.451263,
                    46.0549011,
                ],
                applied: [200.775742, 52.4987335, 7.09983158, 1.0, 1.0, 0.0, 0.0],
            },
        ],
    );
}

// Expected values are generated from the official Spine 4.3 C++ runtime (oracle).
// Scenario:
//   mix(run->walk)=0.2
//   set 0 run (loop)
//   step 0.3
//   set 0 walk (loop)
//   step 0.25
#[test]
fn example_spineboy_run_to_walk_mixing_done_pose_matches_spine_cpp_lite() {
    let mut h = SpineboyHarness::new(|state_data| {
        state_data.set_mix("run", "walk", 0.2).expect("set mix");
    });
    h.skeleton.setup_pose();

    h.state.set_animation(0, "run", true).expect("set run");
    h.step(0.3);

    h.state.set_animation(0, "walk", true).expect("set walk");
    h.step(0.25);

    assert_spineboy_pose(
        &h,
        &[
            BoneExpected {
                name: "root",
                world: [
                    0.999999642,
                    -0.000833316531,
                    0.000833302969,
                    0.999999642,
                    0.0,
                    0.0,
                ],
                applied: [0.0, 0.0, 0.047744751, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "hip",
                world: [
                    0.99717623,
                    0.0750976354,
                    -0.0750976503,
                    0.99717623,
                    -1.56068516,
                    237.746002,
                ],
                applied: [-1.36256695, 237.747223, -4.35457802, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "torso",
                world: [
                    0.193457231,
                    -0.981108725,
                    0.981108725,
                    0.193457201,
                    -2.80534601,
                    242.757736,
                ],
                applied: [-1.61751556, 4.90411377, 83.1522217, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "head",
                world: [
                    0.128377303,
                    -0.999764323,
                    0.995138526,
                    0.128974065,
                    34.7018127,
                    392.49469,
                ],
                applied: [
                    27.663269,
                    -0.259902954,
                    18.5686893,
                    1.00338495,
                    1.00804913,
                    0.0,
                    0.0,
                ],
            },
            BoneExpected {
                name: "front-thigh",
                world: [
                    0.452951878,
                    0.891534984,
                    -0.891535044,
                    0.452951849,
                    -14.1030169,
                    226.982559,
                ],
                applied: [-11.6986046, -11.6749477, -58.7599373, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "front-shin",
                world: [
                    -0.311222106,
                    0.950337291,
                    -0.950337231,
                    -0.311222166,
                    22.967804,
                    157.553055,
                ],
                applied: [78.6901245, 1.60170746, -45.066124, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "front-foot",
                world: [
                    0.632413328,
                    0.774631262,
                    -0.774631202,
                    0.632413268,
                    -17.4262962,
                    35.2973175,
                ],
                applied: [128.755707, -0.339328766, 57.3612976, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-thigh",
                world: [
                    0.353179395,
                    0.935555637,
                    -0.935555696,
                    0.353179425,
                    -3.3274765,
                    226.860565,
                ],
                applied: [-0.944332123, -10.9873753, -65.011261, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-shin",
                world: [
                    -0.960646033,
                    0.277775645,
                    -0.277775645,
                    -0.960646093,
                    25.8413486,
                    145.841125,
                ],
                applied: [86.1000061, -1.32533264, -94.5544128, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-foot",
                world: [
                    -0.0792633668,
                    0.996853769,
                    -0.996853828,
                    -0.0792632028,
                    -91.0475693,
                    112.828232,
                ],
                applied: [121.459038, -0.755193472, 69.3262863, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "gun",
                world: [
                    0.80332166,
                    0.595545352,
                    -0.595545411,
                    0.803321719,
                    73.0534286,
                    262.804932,
                ],
                applied: [34.4218864, -0.453979492, 8.54287338, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "gun-tip",
                world: [
                    0.871701181,
                    0.490037739,
                    -0.490037799,
                    0.871701181,
                    265.606293,
                    185.407227,
                ],
                applied: [200.775742, 52.4987335, 7.20845509, 1.0, 1.0, 0.0, 0.0],
            },
        ],
    );
}

// Expected values are generated from the official Spine 4.3 C++ runtime (oracle).
// Scenario:
//   set 0 run (loop)
//   set 1 aim (loop)
//   step 0.2
#[test]
fn example_spineboy_run_plus_aim_pose_matches_spine_cpp_lite_0p2() {
    let mut h = SpineboyHarness::new(|_| {});
    h.skeleton.setup_pose();

    h.state.set_animation(0, "run", true).expect("set run");
    h.state.set_animation(1, "aim", true).expect("set aim");
    h.step(0.2);

    assert_spineboy_pose(
        &h,
        &[
            BoneExpected {
                name: "root",
                world: [
                    0.999999642,
                    -0.000833316531,
                    0.000833302969,
                    0.999999642,
                    0.0,
                    0.0,
                ],
                applied: [0.0, 0.0, 0.047744751, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "hip",
                world: [
                    0.989800155,
                    0.142462432,
                    -0.142462417,
                    0.989800155,
                    5.85228348,
                    257.823578,
                ],
                applied: [6.06713009, 257.818604, -8.23810577, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "torso",
                world: [
                    0.136981785,
                    -0.990573525,
                    0.990573525,
                    0.136981711,
                    4.94991827,
                    262.908112,
                ],
                applied: [-1.61751556, 4.90411377, 65.7626953, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "head",
                world: [
                    -0.552195489,
                    -0.831739008,
                    0.849292696,
                    -0.540782332,
                    23.132473,
                    415.725739,
                ],
                applied: [
                    27.663269,
                    -0.259902954,
                    39.6006966,
                    1.01302421,
                    0.992086411,
                    0.0,
                    0.0,
                ],
            },
            BoneExpected {
                name: "front-thigh",
                world: [
                    -0.466155708,
                    0.884702682,
                    -0.884702682,
                    -0.466155678,
                    -20.4346046,
                    256.807434,
                ],
                applied: [-25.8740063, -4.75067997, -109.594681, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "front-shin",
                world: [
                    -0.999999702,
                    0.000745342404,
                    -0.000745304045,
                    -0.999999702,
                    -55.6994209,
                    186.44342,
                ],
                applied: [78.6901245, 1.60170746, -62.1722527, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "front-foot",
                world: [
                    -0.790175915,
                    0.612880111,
                    -0.612879992,
                    -0.790175915,
                    -184.455338,
                    186.686783,
                ],
                applied: [128.755707, -0.339328766, 37.7553368, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-thigh",
                world: [
                    0.74301368,
                    0.669276118,
                    -0.669276118,
                    0.74301368,
                    5.71222687,
                    258.083405,
                ],
                applied: [-0.175642967, 0.237220764, -33.8208618, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-shin",
                world: [
                    0.222466573,
                    0.97494024,
                    -0.97494024,
                    0.222466588,
                    68.7986908,
                    199.473984,
                ],
                applied: [86.1000061, -1.32533264, -35.1348267, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "rear-foot",
                world: [
                    0.838583946,
                    0.544772208,
                    -0.544772208,
                    0.838584006,
                    95.0830002,
                    80.8906784,
                ],
                applied: [121.459038, -0.755193472, 44.1369476, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "gun",
                world: [
                    0.887428761,
                    -0.460944563,
                    0.460944712,
                    0.887428939,
                    90.9617004,
                    407.564697,
                ],
                applied: [34.4218864, -0.453979492, -5.73283768, 1.0, 1.0, 0.0, 0.0],
            },
            BoneExpected {
                name: "gun-tip",
                world: [
                    0.823652148,
                    -0.567095041,
                    0.56709528,
                    0.823652327,
                    244.936859,
                    546.700073,
                ],
                applied: [200.775742, 52.4987335, 7.09983158, 1.0, 1.0, 0.0, 0.0],
            },
        ],
    );
}

// Scenario:
//   mix(aim->shoot)=0.2
//   set 0 run (loop)
//   set 1 aim (loop), additive=true
//   step 0.3
//   set 1 shoot (non-loop), additive=true
//   step 0.1
//
// Notes:
// - Latest spine-cpp mixes out the outgoing aim attachment timeline to setup, so crosshair clears.
// - The `shoot` animation uses the upstream JSON key `rgba` for slot color timelines.
#[test]
fn example_spineboy_aim_to_shoot_additive_mixing_clears_crosshair_and_keeps_rgba_colors() {
    let mut h = SpineboyHarness::new(|state_data| {
        state_data.set_mix("aim", "shoot", 0.2).expect("set mix");
    });
    h.skeleton.setup_pose();

    h.state.set_animation(0, "run", true).expect("set run");
    let aim = h.state.set_animation(1, "aim", true).expect("set aim");
    aim.set_additive(&mut h.state, true);
    h.step(0.3);

    let shoot = h.state.set_animation(1, "shoot", false).expect("set shoot");
    shoot.set_additive(&mut h.state, true);
    h.step(0.1);

    assert_slot_attachment(&h, "crosshair", None);
    assert_slot_attachment(&h, "muzzle-glow", Some("muzzle-glow"));
    assert_slot_color_approx(&h, "muzzle-glow", [1.0, 0.883626, 0.826886, 0.5]);
}
