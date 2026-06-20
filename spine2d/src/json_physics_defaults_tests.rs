use crate::SkeletonData;

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
        bone.color,
        [
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            0x44 as f32 / 255.0
        ]
    );
    assert_eq!(bone.icon, "root-icon");
    assert_eq!(bone.icon_size, 2.5);
    assert_eq!(bone.icon_rotation, 45.0);
    assert!(!bone.visible);
    assert!(!data.slots[0].visible);
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
        .skin("default")
        .and_then(|skin| skin.attachment(0, "clip"))
        .and_then(|attachment| match attachment {
            crate::AttachmentData::Clipping(clip) => Some(clip),
            _ => None,
        })
        .expect("clipping attachment");

    assert_eq!(clip.end_slot, Some(1));
    assert!(clip.convex);
    assert!(clip.inverse);
}
