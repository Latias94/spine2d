use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation};

const SKELETON_IK_WITH_TIMELINE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "p", "parent": "root", "length": 1, "x": 0, "y": 0 },
    { "name": "c", "parent": "p", "length": 1, "x": 1, "y": 0 },
    { "name": "t", "parent": "root", "x": 1, "y": 1 }
  ],
  "slots": [],
  "skins": {},
  "ik": [
    { "name": "ik", "bones": ["p", "c"], "target": "t", "mix": 1, "bendPositive": true }
  ],
  "animations": {
    "anim": {
      "ik": {
        "ik": [
          { "time": 0.0, "mix": 0.0, "softness": 0.0 },
          { "time": 1.0, "mix": 1.0, "softness": 10.0 }
        ]
      }
    }
  }
}
"#;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-3,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

fn child_tip(skeleton: &Skeleton, child_index: usize) -> (f32, f32) {
    let child = &skeleton.bones[child_index];
    let len = skeleton.data.bones[child_index].length;
    (child.a * len + child.world_x, child.c * len + child.world_y)
}

#[test]
fn ik_timeline_mix_zero_disables_constraint_effect() {
    let data = SkeletonData::from_json_str(SKELETON_IK_WITH_TIMELINE).unwrap();
    let (_, anim) = data.animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);
    skeleton.update_world_transform();

    let (tip_x, tip_y) = child_tip(&skeleton, 2);
    assert_approx(tip_x, 2.0);
    assert_approx(tip_y, 0.0);
}

#[test]
fn ik_timeline_interpolates_mix_and_moves_tip_toward_target() {
    let data = SkeletonData::from_json_str(SKELETON_IK_WITH_TIMELINE).unwrap();
    let (_, anim) = data.animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);
    skeleton.update_world_transform();

    let (tip_x, tip_y) = child_tip(&skeleton, 2);
    assert!(tip_x < 2.0);
    assert!(tip_y > 0.0);
}

#[test]
fn ik_timeline_interpolates_softness() {
    let data = SkeletonData::from_json_str(SKELETON_IK_WITH_TIMELINE).unwrap();
    let (_, anim) = data.animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);

    assert_approx(skeleton.ik_constraints[0].softness, 5.0);
}

#[test]
fn ik_timeline_applies_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_IK_WITH_TIMELINE).unwrap();
    let (_, anim) = data.animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, -0.5, MixBlend::Replace);

    assert_approx(skeleton.ik_constraints[0].mix(), 1.25);
    assert_approx(skeleton.ik_constraints[0].softness(), -2.5);
}
