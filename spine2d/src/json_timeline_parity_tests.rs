use crate::runtime::MixBlend;
use crate::{BoneTimeline, Curve, Error, MixFrom, Skeleton, SkeletonData, apply_animation};

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

#[test]
fn json_preserves_frame_order_and_uses_each_timeline_last_frame_for_duration() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "events": { "event": {} },
  "animations": {
    "bone": {
      "bones": {
        "root": {
          "rotate": [
            { "time": 2, "value": 20 },
            { "time": 1, "value": 10 }
          ]
        }
      }
    },
    "events": {
      "events": [
        { "time": 4, "name": "event" },
        { "time": 0.5, "name": "event" }
      ]
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let bone_animation = data.find_animation("bone").expect("bone animation");
    let BoneTimeline::Rotate(rotate) = &bone_animation.bone_timelines[0] else {
        panic!("expected rotate timeline");
    };
    let event_animation = data.find_animation("events").expect("event animation");
    let events = event_animation
        .event_timeline
        .as_ref()
        .expect("event timeline")
        .get_events();

    assert_eq!(
        rotate
            .frames
            .iter()
            .map(|frame| frame.time)
            .collect::<Vec<_>>(),
        vec![2.0, 1.0]
    );
    assert_eq!(
        events
            .iter()
            .map(|event| event.get_time())
            .collect::<Vec<_>>(),
        vec![4.0, 0.5]
    );
    assert_eq!(bone_animation.get_duration(), 1.0);
    assert_eq!(event_animation.get_duration(), 0.5);
}

#[test]
fn json_event_playback_respects_input_order_boundaries() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "events": { "a": {}, "b": {}, "c": {} },
  "animations": {
    "test": {
      "events": [
        { "time": 2, "name": "a" },
        { "time": 1, "name": "b" },
        { "time": 3, "name": "c" }
      ]
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let animation = data.find_animation("test").expect("test animation");
    let mut skeleton = Skeleton::new(data.clone());
    let mut events = Vec::new();

    animation.apply(
        &mut skeleton,
        -1.0,
        1.5,
        false,
        Some(&mut events),
        1.0,
        MixFrom::Setup,
        false,
        false,
        false,
    );
    assert!(events.is_empty());

    animation.apply(
        &mut skeleton,
        1.5,
        3.0,
        false,
        Some(&mut events),
        1.0,
        MixFrom::Setup,
        false,
        false,
        false,
    );
    assert_eq!(
        events
            .iter()
            .map(|event| event.get_data().get_name())
            .collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
}

#[test]
fn json_timeline_sampling_uses_official_linear_search() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "rotate": [
            { "time": 0, "value": 0 },
            { "time": 2, "value": 100 },
            { "time": 1, "value": 1000 },
            { "time": 3, "value": 0 }
          ]
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
    apply_animation(animation, &mut skeleton, 1.5, false, 1.0, MixBlend::Replace);

    assert_eq!(skeleton.bones[0].rotation, 75.0);
}

#[test]
fn json_ignores_last_single_value_curve() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "rotate": [
            { "curve": "stepped" },
            { "time": 1, "value": 10, "curve": [0] }
          ]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("last curve is not read");
    let animation = data.find_animation("test").expect("test animation");
    let BoneTimeline::Rotate(rotate) = &animation.bone_timelines[0] else {
        panic!("expected rotate timeline");
    };

    assert_eq!(rotate.frames[0].curve, Curve::Stepped);
    assert_eq!(rotate.frames[1].curve, Curve::Linear);
}

#[test]
fn json_ignores_last_multi_value_curve() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "translate": [
            { "curve": [0.25, 0, 0.75, 1, 0.25, 0, 0.75, 1] },
            { "time": 1, "x": 10, "y": 20, "curve": [0] }
          ]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("last curve is not read");
    let animation = data.find_animation("test").expect("test animation");
    let BoneTimeline::Translate(translate) = &animation.bone_timelines[0] else {
        panic!("expected translate timeline");
    };

    assert!(
        translate.frames[0]
            .curve
            .iter()
            .all(|curve| matches!(curve, Curve::Bezier { .. }))
    );
    assert_eq!(translate.frames[1].curve, [Curve::Linear; 2]);
}

#[test]
fn json_ignores_single_frame_curve() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "rotate": [{ "time": 1, "curve": [0] }]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("single frame curve is not read");
    let animation = data.find_animation("test").expect("test animation");
    let BoneTimeline::Rotate(rotate) = &animation.bone_timelines[0] else {
        panic!("expected rotate timeline");
    };

    assert_eq!(rotate.frames[0].curve, Curve::Linear);
}

#[test]
fn json_preserves_deform_order_and_ignores_last_curve() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [{ "name": "slot", "bone": "root", "attachment": "mesh" }],
  "skins": {
    "default": {
      "slot": {
        "mesh": {
          "type": "mesh",
          "uvs": [0, 0, 1, 0, 1, 1, 0, 1],
          "vertices": [-1, -1, 1, -1, 1, 1, -1, 1],
          "triangles": [0, 1, 2, 2, 3, 0]
        }
      }
    }
  },
  "animations": {
    "test": {
      "attachments": {
        "default": {
          "slot": {
            "mesh": {
              "deform": [
                { "time": 2, "vertices": [1, 0, 0, 0, 0, 0, 0, 0], "curve": "stepped" },
                { "time": 1, "vertices": [0, 1, 0, 0, 0, 0, 0, 0], "curve": [0] }
              ]
            }
          }
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("last deform curve is not read");
    let animation = data.find_animation("test").expect("test animation");
    let timeline = &animation.deform_timelines[0];

    assert_eq!(
        timeline
            .frames
            .iter()
            .map(|frame| frame.time)
            .collect::<Vec<_>>(),
        vec![2.0, 1.0]
    );
    assert_eq!(timeline.frames[0].curve, Curve::Stepped);
    assert_eq!(timeline.frames[1].curve, Curve::Linear);
    assert_eq!(animation.get_duration(), 1.0);
}

#[test]
fn json_preserves_rgba2_order_and_ignores_last_curve() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [{ "name": "slot", "bone": "root", "dark": "000000" }],
  "animations": {
    "test": {
      "slots": {
        "slot": {
          "rgba2": [
            { "time": 2, "light": "FFFFFFFF", "dark": "000000", "curve": "stepped" },
            { "time": 1, "light": "FFFFFFFF", "dark": "000000", "curve": [0] }
          ]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("last rgba2 curve is not read");
    let animation = data.find_animation("test").expect("test animation");
    let timeline = &animation.slot_rgba2_timelines[0];

    assert_eq!(
        timeline
            .frames
            .iter()
            .map(|frame| frame.time)
            .collect::<Vec<_>>(),
        vec![2.0, 1.0]
    );
    assert_eq!(timeline.frames[0].curve, [Curve::Stepped; 7]);
    assert_eq!(timeline.frames[1].curve, [Curve::Linear; 7]);
    assert_eq!(animation.get_duration(), 1.0);
}

#[test]
fn json_still_rejects_invalid_nonfinal_curve() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "animations": {
    "test": {
      "bones": {
        "root": {
          "rotate": [
            { "curve": [0] },
            { "time": 1, "value": 10 }
          ]
        }
      }
    }
  }
}
"#;

    let error = SkeletonData::from_json_str(json).expect_err("nonfinal curve must be read");

    assert!(matches!(error, Error::JsonInvalidCurve { .. }));
}
