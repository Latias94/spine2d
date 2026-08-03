use crate::{
    AnimationState, AnimationStateData, PhysicsConstraintTimeline, Skeleton, SkeletonData,
};

fn assert_approx_eq(actual: f32, expected: f32) {
    let difference = (actual - expected).abs();
    assert!(
        difference <= 1.0e-6,
        "expected {expected}, got {actual} (difference {difference})"
    );
}

#[test]
fn json_physics_constraint_defaults_match_spine_cpp() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [],
  "constraints": [
    { "name": "p0", "type": "physics", "bone": "root" }
  ]
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let c = data
        .physics_constraints
        .first()
        .expect("physics constraint");

    assert_eq!(c.inertia, 0.5);
    assert_eq!(c.strength, 100.0);
    assert_eq!(c.damping, 0.85);
    assert_eq!(c.mass_inverse, 1.0);
    assert_eq!(c.wind, 0.0);
    assert_eq!(c.gravity, 0.0);
    assert_eq!(c.mix, 1.0);
    assert_eq!(c.limit, 5000.0);
    assert!((c.step - (1.0 / 60.0)).abs() <= 1.0e-6);
}

#[test]
fn json_named_physics_mix_timeline_uses_official_default_value() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [],
  "constraints": [
    { "name": "physics", "type": "physics", "bone": "root", "mix": 1 }
  ],
  "animations": {
    "test": {
      "physics": {
        "physics": {
          "mix": [
            { "value": 0 },
            { "time": 1 }
          ],
          "mass": [
            { "value": 2 },
            { "time": 1 }
          ]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let animation = data.find_animation("test").expect("test animation");

    let mix_timeline = animation
        .physics_constraint_timelines
        .iter()
        .find_map(|timeline| match timeline {
            PhysicsConstraintTimeline::Mix(timeline) => Some(timeline),
            _ => None,
        })
        .expect("mix timeline");
    assert_eq!(
        mix_timeline
            .frames
            .iter()
            .map(|frame| frame.value)
            .collect::<Vec<_>>(),
        vec![0.0, 1.0]
    );

    let mass_timeline = animation
        .physics_constraint_timelines
        .iter()
        .find_map(|timeline| match timeline {
            PhysicsConstraintTimeline::Mass(timeline) => Some(timeline),
            _ => None,
        })
        .expect("mass timeline");
    assert_eq!(
        mass_timeline
            .frames
            .iter()
            .map(|frame| frame.value)
            .collect::<Vec<_>>(),
        vec![2.0, 0.0]
    );

    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    state.set_animation(0, "test", false);
    state.update(0.5);
    assert!(state.apply(&mut skeleton));
    assert_approx_eq(skeleton.physics_constraints[0].get_mix(), 0.5);
}

#[test]
fn json_global_physics_mix_timeline_uses_official_default_value() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [],
  "constraints": [
    {
      "name": "physics",
      "type": "physics",
      "bone": "root",
      "mix": 1,
      "mixGlobal": true
    }
  ],
  "animations": {
    "test": {
      "physics": {
        "": {
          "mix": [
            { "value": 0 },
            { "time": 1 }
          ]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let animation = data.find_animation("test").expect("test animation");
    let mix_timeline = animation
        .physics_constraint_timelines
        .iter()
        .find_map(|timeline| match timeline {
            PhysicsConstraintTimeline::Mix(timeline) => Some(timeline),
            _ => None,
        })
        .expect("global mix timeline");

    assert_eq!(mix_timeline.constraint_index, -1);
    assert_eq!(
        mix_timeline
            .frames
            .iter()
            .map(|frame| frame.value)
            .collect::<Vec<_>>(),
        vec![0.0, 1.0]
    );

    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    state.set_animation(0, "test", false);
    state.update(0.5);
    assert!(state.apply(&mut skeleton));
    assert_approx_eq(skeleton.physics_constraints[0].get_mix(), 0.5);
}

#[test]
fn json_unknown_physics_timeline_is_ignored_before_decoding_keys() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [],
  "animations": {
    "test": {
      "physics": {
        "": {
          "future": { "opaque": true },
          "mix": [
            { "time": 1, "value": 1 }
          ]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("unknown physics timeline is ignored");
    let animation = data.find_animation("test").expect("test animation");

    assert_approx_eq(animation.get_duration(), 1.0);
    assert_eq!(animation.physics_constraint_timelines.len(), 1);
}

#[test]
fn json_skeleton_header_metadata_matches_spine_cpp() {
    let json = r#"
{
  "skeleton": {
    "spine": "4.3.00",
    "hash": "abc123",
    "x": 1,
    "y": 2,
    "width": 3,
    "height": 4,
    "referenceScale": 50,
    "fps": 24,
    "images": "images/",
    "audio": "audio/"
  },
  "bones": [{ "name": "root" }]
}
"#;

    let data = SkeletonData::from_json_str_with_scale(json, 2.0).expect("parse skeleton json");

    assert_eq!(data.name, "");
    assert_eq!(data.spine_version.as_deref(), Some("4.3.00"));
    assert_eq!(data.hash, "abc123");
    assert_eq!(data.x, 1.0);
    assert_eq!(data.y, 2.0);
    assert_eq!(data.width, 3.0);
    assert_eq!(data.height, 4.0);
    assert_eq!(data.reference_scale, 100.0);
    assert_eq!(data.fps, 24.0);
    assert_eq!(data.images_path, "images/");
    assert_eq!(data.audio_path, "audio/");
}

#[test]
fn json_ik_scale_y_mode_matches_spine_cpp() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "bone", "parent": "root", "length": 10 },
    { "name": "target", "parent": "root", "x": 20 }
  ],
  "slots": [],
  "constraints": [
    { "name": "ik_default", "type": "ik", "bones": ["bone"], "target": "target" },
    { "name": "ik_uniform", "type": "ik", "bones": ["bone"], "target": "target", "scaleY": "uniform" },
    { "name": "ik_volume", "type": "ik", "bones": ["bone"], "target": "target", "scaleY": "volume" },
    { "name": "physics_scale_x", "type": "physics", "bone": "bone", "scaleX": 0.5, "scaleY": "uniform" }
  ]
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");

    assert_eq!(data.ik_constraints[0].scale_y_mode, crate::ScaleYMode::None);
    assert_eq!(
        data.ik_constraints[1].scale_y_mode,
        crate::ScaleYMode::Uniform
    );
    assert_eq!(
        data.ik_constraints[2].scale_y_mode,
        crate::ScaleYMode::Volume
    );
    assert_eq!(data.physics_constraints[0].scale_x, 0.5);
    assert_eq!(
        data.physics_constraints[0].scale_y_mode,
        crate::ScaleYMode::Uniform
    );
}

#[test]
fn json_bone_and_slot_nonessential_fields_are_preserved() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    {
      "name": "root",
      "color": "11223344",
      "icon": "root-icon",
      "iconSize": 2.5,
      "iconRotation": 45,
      "visible": false
    }
  ],
  "slots": [
    { "name": "slot0", "bone": "root", "visible": false }
  ],
  "animations": {}
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let bone = &data.bones[0];
    assert_eq!(
        bone.get_color(),
        [
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            0x44 as f32 / 255.0
        ]
    );
    assert_eq!(bone.get_icon(), "root-icon");
    assert_eq!(bone.get_icon_size(), 2.5);
    assert_eq!(bone.get_icon_rotation(), 45.0);
    assert!(!bone.get_visible());
    assert!(!data.slots[0].get_visible());
}

#[test]
fn json_clipping_flags_parse_convex_and_inverse() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [{ "name": "root" }],
  "slots": [
    { "name": "clip", "bone": "root", "attachment": "clip" },
    { "name": "end", "bone": "root" }
  ],
  "skins": {
    "default": {
      "clip": {
        "clip": {
          "type": "clipping",
          "end": "end",
          "convex": true,
          "inverse": true,
          "vertexCount": 3,
          "vertices": [0,0, 10,0, 0,10]
        }
      }
    }
  }
}
"#;

    let data = SkeletonData::from_json_str(json).expect("parse skeleton json");
    let clip = data
        .find_skin("default")
        .and_then(|skin| skin.get_attachment(0, "clip"))
        .and_then(|attachment| match attachment {
            crate::AttachmentData::Clipping(clip) => Some(clip),
            _ => None,
        })
        .expect("clipping attachment");

    assert_eq!(clip.end_slot, Some(1));
    assert!(clip.convex);
    assert!(clip.inverse);
}
