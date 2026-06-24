use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation};

const SKELETON_WITH_PATH_CONSTRAINT_AND_TIMELINES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root" }
  ],
  "slots": [
    { "name": "pathSlot", "bone": "root", "attachment": "p" }
  ],
  "skins": [
    {
      "name": "default",
      "attachments": {
        "pathSlot": {
          "p": {
            "type": "path",
            "vertexCount": 3,
            "vertices": [ 0, 0, 10, 0, 20, 0 ],
            "lengths": [ 20 ],
            "closed": false,
            "constantSpeed": true
          }
        }
      }
    }
  ],
  "path": [
    {
      "name": "pc",
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "percent",
      "spacingMode": "length",
      "rotateMode": "tangent",
      "position": 0,
      "spacing": 0,
      "mixRotate": 0,
      "mixX": 0,
      "mixY": 0
    }
  ],
  "animations": {
    "anim": {
      "path": {
        "pc": {
          "mix": [
            { "time": 0.0, "mixRotate": 0.0, "mixX": 0.0, "mixY": 0.0 },
            { "time": 1.0, "mixRotate": 1.0, "mixX": 1.0, "mixY": 1.0 }
          ],
          "position": [
            { "time": 0.0, "value": 0.0 },
            { "time": 1.0, "value": 10.0 }
          ],
          "spacing": [
            { "time": 0.0, "value": 0.0 },
            { "time": 1.0, "value": 5.0 }
          ]
        }
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

#[test]
fn path_constraint_timelines_update_runtime_values() {
    let data = SkeletonData::from_json_str(SKELETON_WITH_PATH_CONSTRAINT_AND_TIMELINES).unwrap();
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);

    assert_approx(skeleton.path_constraints[0].mix_rotate, 0.5);
    assert_approx(skeleton.path_constraints[0].mix_x, 0.5);
    assert_approx(skeleton.path_constraints[0].mix_y, 0.5);
    assert_approx(skeleton.path_constraints[0].position, 5.0);
    assert_approx(skeleton.path_constraints[0].spacing, 2.5);
}

#[test]
fn path_constraint_timelines_apply_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_WITH_PATH_CONSTRAINT_AND_TIMELINES).unwrap();
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, -0.5, MixBlend::Replace);

    assert_approx(skeleton.path_constraints[0].mix_rotate, -0.25);
    assert_approx(skeleton.path_constraints[0].mix_x, -0.25);
    assert_approx(skeleton.path_constraints[0].mix_y, -0.25);
    assert_approx(skeleton.path_constraints[0].position, -2.5);
    assert_approx(skeleton.path_constraints[0].spacing, -1.25);
}
