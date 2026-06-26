use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation};
use std::sync::Arc;

const SKELETON_WITH_SLIDER_TIMELINES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "a" },
    { "name": "slot1", "bone": "root", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "slot0": { "a": { "type": "region", "path": "a.png", "width": 10, "height": 10 } },
      "slot1": { "b": { "type": "region", "path": "b.png", "width": 10, "height": 10 } }
    }
  },
  "slider": [
    {
      "name": "slider",
      "time": 1.0,
      "mix": 0.5,
      "animation": "anim"
    }
  ],
  "animations": {
    "idle": {},
    "anim": {
      "drawOrder": [
        { "time": 0.0, "offsets": [ { "slot": "slot1", "offset": -1 } ] }
      ],
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
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);

    assert_approx(skeleton.slider_constraints[0].get_time(), 3.0);
    assert_approx(skeleton.slider_constraints[0].get_mix(), 0.9);
}

#[test]
fn slider_timelines_apply_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_WITH_SLIDER_TIMELINES).unwrap();
    let anim = data.find_animation("anim").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, -0.5, MixBlend::Replace);

    assert_approx(skeleton.slider_constraints[0].get_time(), 0.0);
    assert_approx(skeleton.slider_constraints[0].get_mix(), 0.3);
}

#[test]
fn skeleton_data_find_slider_animations_appends_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_WITH_SLIDER_TIMELINES).unwrap();
    let idle = data.find_animation("idle").unwrap();
    let mut animations = vec![idle];

    let returned_len = data.find_slider_animations(&mut animations).len();

    assert_eq!(returned_len, 2);
    assert_eq!(animations[0].name, "idle");
    assert_eq!(animations[1].name, "anim");
}

#[test]
fn slider_animation_name_only_binding_still_drives_runtime() {
    let mut data = SkeletonData::from_json_str(SKELETON_WITH_SLIDER_TIMELINES).unwrap();
    {
        let data = Arc::make_mut(&mut data);
        let slider = data
            .slider_constraints
            .get_mut(0)
            .expect("slider constraint exists");
        slider.animation = None;
        slider.animation_name = Some("anim".to_string());
    }

    let mut skeleton = Skeleton::new(Arc::clone(&data));
    assert_eq!(data.find_slider_animations(&mut Vec::new()).len(), 1);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert_eq!(skeleton.get_draw_order(), &[1, 0]);
}
