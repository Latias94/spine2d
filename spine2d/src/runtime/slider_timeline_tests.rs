use crate::{MixBlend, Skeleton, SkeletonData, apply_animation};

const SKELETON_WITH_SLIDER_TIMELINES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slider": [
    {
      "name": "slider",
      "bone": "root",
      "time": 1.0,
      "mix": 0.5
    }
  ],
  "animations": {
    "anim": {
      "slider": {
        "slider": {
          "time": [
            { "time": 0.0, "value": 3.0 },
            { "time": 1.0, "value": 3.0 }
          ],
          "mix": [
            { "time": 0.0, "value": 0.9 },
            { "time": 1.0, "value": 0.9 }
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
fn slider_timelines_update_runtime_values() {
    let data = SkeletonData::from_json_str(SKELETON_WITH_SLIDER_TIMELINES).unwrap();
    let (_, anim) = data.animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);

    assert_approx(skeleton.slider_constraints[0].time(), 3.0);
    assert_approx(skeleton.slider_constraints[0].mix(), 0.9);
}

#[test]
fn slider_timelines_apply_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_WITH_SLIDER_TIMELINES).unwrap();
    let (_, anim) = data.animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, -0.5, MixBlend::Replace);

    assert_approx(skeleton.slider_constraints[0].time(), 0.0);
    assert_approx(skeleton.slider_constraints[0].mix(), 0.3);
}
