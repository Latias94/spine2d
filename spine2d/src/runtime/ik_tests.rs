use crate::{IkConstraint, Skeleton, SkeletonData};

const SKELETON_IK_TWO_BONES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "p", "parent": "root", "length": 1, "x": 0, "y": 0 },
    { "name": "c", "parent": "p", "length": 1, "x": 1, "y": 0 },
    { "name": "t", "parent": "root", "x": 1, "y": 1 }
  ],
  "slots": [],
  "skins": {},
  "ik": [
    { "name": "ik", "bones": ["p", "c"], "target": "t", "mix": 1, "bendPositive": true }
  ],
  "animations": {}
}
"#;

const SKELETON_IK_ONE_BONE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "p", "parent": "root", "length": 1, "x": 0, "y": 0 },
    { "name": "t", "parent": "root", "x": 0, "y": 1 }
  ],
  "slots": [],
  "skins": {},
  "ik": [
    { "name": "ik", "bones": ["p"], "target": "t", "mix": 1, "bendPositive": true }
  ],
  "animations": {}
}
"#;

const SKELETON_IK_DEFAULT_BEND_POSITIVE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "p", "parent": "root", "length": 1, "x": 0, "y": 0 },
    { "name": "t", "parent": "root", "x": 0, "y": 1 }
  ],
  "slots": [],
  "skins": {},
  "ik": [
    { "name": "ik", "bones": ["p"], "target": "t", "mix": 1 }
  ],
  "animations": {}
}
"#;

fn one_bone_stretch_json(scale_y_mode: Option<&str>) -> String {
    let scale_y = scale_y_mode
        .map(|mode| format!(r#", "scaleY": "{mode}""#))
        .unwrap_or_default();
    format!(
        r#"
{{
  "skeleton": {{ "spine": "4.3.00" }},
  "bones": [
    {{ "name": "root" }},
    {{ "name": "p", "parent": "root", "length": 1, "x": 0, "y": 0, "scaleY": 2 }},
    {{ "name": "t", "parent": "root", "x": 2, "y": 0 }}
  ],
  "slots": [],
  "skins": {{}},
  "ik": [
    {{ "name": "ik", "bones": ["p"], "target": "t", "mix": 1, "stretch": true{scale_y} }}
  ],
  "animations": {{}}
}}
"#
    )
}

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-3,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn ik_two_bones_moves_end_effector_close_to_target() {
    let data = SkeletonData::from_json_str(SKELETON_IK_TWO_BONES).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let target = &skeleton.bones[3];
    let child = &skeleton.bones[2];
    let child_len = skeleton.data.bones[2].length;
    let tip_x = child.a * child_len + child.world_x;
    let tip_y = child.c * child_len + child.world_y;

    assert_approx(tip_x, target.world_x);
    assert_approx(tip_y, target.world_y);
}

#[test]
fn ik_one_bone_rotates_toward_target() {
    let data = SkeletonData::from_json_str(SKELETON_IK_ONE_BONE).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let target = &skeleton.bones[2];
    let bone = &skeleton.bones[1];
    let bone_len = skeleton.data.bones[1].length;
    let tip_x = bone.a * bone_len + bone.world_x;
    let tip_y = bone.c * bone_len + bone.world_y;

    assert_approx(tip_x, target.world_x);
    assert_approx(tip_y, target.world_y);
}

#[test]
fn ik_one_bone_negative_mix_rotates_away_from_target() {
    let data = SkeletonData::from_json_str(SKELETON_IK_ONE_BONE).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_mix(-1.0);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert!(skeleton.bones[1].arotation < 0.0);
}

#[test]
fn ik_one_bone_nan_mix_propagates_nan_rotation() {
    let data = SkeletonData::from_json_str(SKELETON_IK_ONE_BONE).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_mix(f32::NAN);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert!(skeleton.bones[1].arotation.is_nan());
}

#[test]
fn ik_constraint_bend_positive_defaults_to_true() {
    let data = SkeletonData::from_json_str(SKELETON_IK_DEFAULT_BEND_POSITIVE).unwrap();
    assert_eq!(data.ik_constraints.len(), 1);
    assert_eq!(data.ik_constraints[0].bend_direction, 1);
}

#[test]
fn ik_scale_y_mode_controls_stretch_scale_y() {
    let data = SkeletonData::from_json_str(&one_bone_stretch_json(None)).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert_approx(skeleton.bones[1].ascale_x, 2.0);
    assert_approx(skeleton.bones[1].ascale_y, 2.0);

    let data = SkeletonData::from_json_str(&one_bone_stretch_json(Some("uniform"))).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert_approx(skeleton.bones[1].ascale_x, 2.0);
    assert_approx(skeleton.bones[1].ascale_y, 4.0);
}
