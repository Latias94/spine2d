use crate::runtime::MixBlend;
use crate::{Error, Skeleton, SkeletonData, apply_animation};

#[test]
fn json_rotate_timeline_only_reads_official_value_field() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "rotate": [{ "angle": 30, "value": 10 }]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let animation = data.find_animation("test").expect("test animation");
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Replace);

    assert_eq!(skeleton.bones[0].rotation, 10.0);
}

#[test]
fn json_rejects_nonofficial_camel_case_bone_timeline_names() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "translateX": [{ "value": 3 }]
        }
      }
    }
  }
}
"#;

    let error =
        SkeletonData::from_json_str(json).expect_err("reject nonofficial bone timeline name");

    assert!(matches!(
        error,
        Error::JsonUnknownBoneTimeline {
            animation,
            bone,
            timeline,
        } if animation == "test" && bone == "root" && timeline == "translateX"
    ));
}

#[test]
fn json_rejects_unknown_slot_timeline_names() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [{ "name": "slot", "bone": "root" }],
  "animations": {
    "test": {
      "slots": {
        "slot": {
          "future": [{}]
        }
      }
    }
  }
}
"#;

    let error = SkeletonData::from_json_str(json).expect_err("reject unknown slot timeline name");

    assert!(matches!(
        error,
        Error::JsonUnknownSlotTimeline {
            animation,
            slot,
            timeline,
        } if animation == "test" && slot == "slot" && timeline == "future"
    ));
}

#[test]
fn json_ignores_empty_unknown_bone_and_slot_timelines() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [{ "name": "slot", "bone": "root" }],
  "animations": {
    "test": {
      "bones": { "root": { "future": [] } },
      "slots": { "slot": { "future": [] } }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("empty unknown timelines are ignored");
    let animation = data.find_animation("test").expect("test animation");

    assert_eq!(animation.get_timelines().count(), 0);
}

#[test]
fn json_omits_empty_bone_timelines() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "translate": []
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let animation = data.find_animation("test").expect("test animation");

    assert_eq!(animation.get_timelines().count(), 0);
}

#[test]
fn json_omits_empty_slot_timelines() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [{ "name": "slot", "bone": "root" }],
  "animations": {
    "test": {
      "slots": {
        "slot": {
          "attachment": [],
          "rgba": [],
          "rgb": [],
          "alpha": [],
          "rgba2": [],
          "rgb2": []
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let animation = data.find_animation("test").expect("test animation");

    assert_eq!(animation.get_timelines().count(), 0);
}

#[test]
fn json_ignores_empty_ik_and_transform_timelines_before_constraint_lookup() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "ik": { "missing": [] },
      "transform": { "missing": [] }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("empty timelines are ignored");
    let animation = data.find_animation("test").expect("test animation");

    assert_eq!(animation.get_timelines().count(), 0);
}
