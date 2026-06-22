use crate::{Skeleton, SkeletonData};

const SKELETON_TRANSFORM_ABSOLUTE_WORLD: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0 },
    { "name": "t", "parent": "root", "x": 1, "y": 2, "rotation": 90 }
  ],
  "slots": [],
  "skins": {},
  "constraints": [
    {
      "type": "transform",
      "name": "tc",
      "source": "t",
      "bones": ["b"],
      "properties": {
        "rotate": { "to": { "rotate": {} } },
        "x": { "to": { "x": {} } },
        "y": { "to": { "y": {} } }
      }
    }
  ],
  "animations": {}
}
"#;

const SKELETON_CONSTRAINT_ORDER_TRANSFORM_THEN_IK: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "p", "parent": "root", "rotation": 45 },
    { "name": "ti", "parent": "root", "x": 0, "y": 1 },
    { "name": "tt", "parent": "root", "rotation": 0 }
  ],
  "slots": [],
  "skins": {},
  "constraints": [
    {
      "type": "transform",
      "name": "tc",
      "source": "tt",
      "bones": ["p"],
      "properties": { "rotate": { "to": { "rotate": {} } } }
    },
    { "type": "ik", "name": "ik", "bones": ["p"], "target": "ti", "mix": 1, "bendPositive": true }
  ],
  "animations": {}
}
"#;

const SKELETON_TRANSFORM_RELATIVE_WORLD_ROTATION: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "rotation": 45 },
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
      "additive": true,
      "properties": { "rotate": { "to": { "rotate": {} } } }
    }
  ],
  "animations": {}
}
"#;

const SKELETON_TRANSFORM_ABSOLUTE_LOCAL_ROTATION: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "rotation": 0 },
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
      "localSource": true,
      "localTarget": true,
      "properties": { "rotate": { "to": { "rotate": {} } } }
    }
  ],
  "animations": {}
}
"#;

const SKELETON_TRANSFORM_RELATIVE_LOCAL_ROTATION: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "rotation": 45 },
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
      "localSource": true,
      "localTarget": true,
      "additive": true,
      "properties": { "rotate": { "to": { "rotate": {} } } }
    }
  ],
  "animations": {}
}
"#;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-3,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

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
fn transform_constraint_absolute_world_rotates_and_translates_bone_toward_target() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_ABSOLUTE_WORLD).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform();

    let bone = &skeleton.bones[1];
    let target = &skeleton.bones[2];

    assert_approx(bone.world_x, target.world_x);
    assert_approx(bone.world_y, target.world_y);
    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 90.0);
}

#[test]
fn constraints_apply_in_order_across_types() {
    let data = SkeletonData::from_json_str(SKELETON_CONSTRAINT_ORDER_TRANSFORM_THEN_IK).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform();

    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 90.0);
}

#[test]
fn transform_constraint_relative_world_rotates_by_target_rotation() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_RELATIVE_WORLD_ROTATION).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform();

    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 135.0);
}

#[test]
fn transform_constraint_absolute_local_rotates_toward_target_local_rotation() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_ABSOLUTE_LOCAL_ROTATION).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform();

    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 90.0);
}

#[test]
fn transform_constraint_relative_local_rotates_by_target_local_rotation() {
    let data = SkeletonData::from_json_str(SKELETON_TRANSFORM_RELATIVE_LOCAL_ROTATION).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform();

    assert_angle_approx(bone_world_rotation_degrees(&skeleton, 1), 135.0);
}
