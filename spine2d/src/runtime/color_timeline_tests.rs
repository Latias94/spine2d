use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation, build_draw_list};

const SKELETON_SLOT_RGBA_TIMELINE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "a", "color": "FF0000FF" } ],
  "skins": {
    "default": {
      "slot0": {
        "a": { "type": "region", "path": "a.png", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {
    "anim": {
      "slots": {
        "slot0": {
          "rgba": [
            { "time": 0.0, "color": "00FF00FF" },
            { "time": 1.0, "color": "0000FFFF" }
          ]
        }
      }
    }
  }
}
"#;

const SKELETON_SLOT_RGBA2_TIMELINE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    {
      "name": "slot0",
      "bone": "root",
      "attachment": "a",
      "color": "FF0000FF",
      "dark": "0000FFFF"
    }
  ],
  "skins": {
    "default": {
      "slot0": {
        "a": { "type": "region", "path": "a.png", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {
    "anim": {
      "slots": {
        "slot0": {
          "rgba2": [
            { "time": 0.0, "light": "00FF00FF", "dark": "FF0000" }
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
        diff <= 1.0e-6,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn slot_color_timeline_interpolates_and_affects_draw_list_vertex_colors() {
    let data = SkeletonData::from_json_str(SKELETON_SLOT_RGBA_TIMELINE).unwrap();
    let (_, animation) = data.animation("anim").unwrap();

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform();

    apply_animation(animation, &mut skeleton, 0.5, false, 1.0, MixBlend::Replace);
    assert_approx(skeleton.slots[0].color[0], 0.0);
    assert_approx(skeleton.slots[0].color[1], 0.5);
    assert_approx(skeleton.slots[0].color[2], 0.5);
    assert_approx(skeleton.slots[0].color[3], 1.0);

    let draw_list = build_draw_list(&skeleton);
    assert!(!draw_list.vertices.is_empty());
    assert_approx(draw_list.vertices[0].color[0], 0.0);
    assert_approx(draw_list.vertices[0].color[1], 0.5);
    assert_approx(draw_list.vertices[0].color[2], 0.5);
    assert_approx(draw_list.vertices[0].color[3], 1.0);
}

#[test]
fn slot_color_timeline_applies_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_SLOT_RGBA_TIMELINE).unwrap();
    let (_, animation) = data.animation("anim").unwrap();

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform();

    apply_animation(
        animation,
        &mut skeleton,
        0.5,
        false,
        -0.5,
        MixBlend::Replace,
    );
    assert_approx(skeleton.slots[0].color[0], 1.5);
    assert_approx(skeleton.slots[0].color[1], -0.25);
    assert_approx(skeleton.slots[0].color[2], -0.25);
    assert_approx(skeleton.slots[0].color[3], 1.0);
}

#[test]
fn slot_rgba2_timeline_applies_negative_alpha_to_dark_color_like_cpp() {
    let data = SkeletonData::from_json_str(SKELETON_SLOT_RGBA2_TIMELINE).unwrap();
    let (_, animation) = data.animation("anim").unwrap();

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform();

    apply_animation(
        animation,
        &mut skeleton,
        0.0,
        false,
        -0.5,
        MixBlend::Replace,
    );
    assert_approx(skeleton.slots[0].color[0], 1.5);
    assert_approx(skeleton.slots[0].color[1], -0.5);
    assert_approx(skeleton.slots[0].color[2], 0.0);
    assert_approx(skeleton.slots[0].color[3], 1.0);
    assert!(skeleton.slots[0].has_dark());
    assert_approx(skeleton.slots[0].dark_color()[0], -0.5);
    assert_approx(skeleton.slots[0].dark_color()[1], 0.0);
    assert_approx(skeleton.slots[0].dark_color()[2], 1.5);
}
