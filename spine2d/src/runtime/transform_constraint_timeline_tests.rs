use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation};

const SKELETON_TRANSFORM_WITH_TIMELINE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0 },
    { "name": "t", "parent": "root", "rotation": 90 }
  ],
  "slots": [],
  "skins": {},
  "constraints": [
    {
      "type": "transform",
      "name": "tc",
      "source": "t",
      "bones": ["b"],
      "properties": { "rotate": { "to": { "rotate": {} } } },
      "mixRotate": 0
    }
  ],
  "animations": {
    "anim": {
      "transform": {
        "tc": [
          { "time": 0.0, "mixRotate": 0.0, "mixX": 0.0, "mixY": 0.0, "mixScaleX": 0.0, "mixScaleY": 0.0, "mixShearY": 0.0 },
          { "time": 1.0, "mixRotate": 1.0, "mixX": 0.0, "mixY": 0.0, "mixScaleX": 0.0, "mixScaleY": 0.0, "mixShearY": 0.0 }
        ]
      }
    }
  }
}
"#;

fn shortest_rotation(mut degrees: f32) -> f32 {
    degrees = degrees.rem_euclid(360.0);
    if degrees > 180.0 {
        degrees -= 360.0;
    }
    degrees
}

fn bone_world_rotation_degrees(skeleton: &Skeleton, bone_index: usize) -> f32 {
    let bone = &skeleton.bones[bone_index];
    bone.c.atan2(bone.a).to_degrees()
}

fn assert_angle_approx(actual: f32, expected: f32) {
    let diff = shortest_rotation(actual - expected).abs();
    assert!(
        diff <= 1.0e-2,
        "expected angle {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn transform_timeline_mix_zero_disables_rotation() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_WITH_TIMELINE).unwrap();
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 0.0);
}

#[test]
fn transform_timeline_interpolates_mix_rotate() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_WITH_TIMELINE).unwrap();
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 45.0);
}

#[test]
fn transform_timeline_applies_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_WITH_TIMELINE).unwrap();
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, -0.5, MixBlend::Replace);

    assert_angle_approx(skeleton.transform_constraints[0].mix_rotate(), -0.25);
}
