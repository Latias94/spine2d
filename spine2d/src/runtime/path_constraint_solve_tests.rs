use crate::{Skeleton, SkeletonData};

const SKELETON_PATH_CONSTRAINT_SOLVE_CONSTANT_SPEED_TRUE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 3.3333333, 0, 6.6666665, 0, 10, 0, 10, 0 ],
            "lengths": [ 10 ],
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
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "fixed",
      "rotateMode": "tangent",
      "position": 5,
      "spacing": 0,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_CONSTANT_SPEED_FALSE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 3.3333333, 0, 6.6666665, 0, 10, 0, 10, 0 ],
            "lengths": [ 10 ],
            "closed": false,
            "constantSpeed": false
          }
        }
      }
    }
  ],
  "path": [
    {
      "name": "pc",
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "fixed",
      "rotateMode": "tangent",
      "position": 5,
      "spacing": 0,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_POSITION_PERCENT_SPACING_PERCENT: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 6.6666665, 0, 13.333333, 0, 20, 0, 20, 0 ],
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
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "percent",
      "spacingMode": "percent",
      "rotateMode": "tangent",
      "position": 0.25,
      "spacing": 0.10,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_ROTATE_MODE_CHAIN_TWO_BONES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b1", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 5 },
    { "name": "b2", "parent": "b1", "x": 5, "y": 0, "rotation": 90, "length": 5 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 6.6666665, 0, 13.333333, 0, 20, 0, 20, 0 ],
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
      "order": 0,
      "bones": ["b1", "b2"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "length",
      "rotateMode": "chain",
      "position": 0,
      "spacing": 0,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_ROTATE_MODE_CHAIN_SCALE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 0, "length": 2 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 6.6666665, 0, 13.333333, 0, 20, 0, 20, 0 ],
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
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "fixed",
      "rotateMode": "chainScale",
      "position": 0,
      "spacing": 4,
      "mixRotate": 1,
      "mixX": 0,
      "mixY": 0
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_SPACING_PROPORTIONAL_TWO_BONES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b1", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 },
    { "name": "b2", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 3.3333333, 0, 6.6666665, 0, 10, 0, 10, 0 ],
            "lengths": [ 10 ],
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
      "order": 0,
      "bones": ["b1", "b2"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "proportional",
      "rotateMode": "tangent",
      "position": 0,
      "spacing": 1,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_CLOSED_WRAP: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 0, 0, 10, 0, 10, 0, 10, 0 ],
            "lengths": [ 10, 20 ],
            "closed": true,
            "constantSpeed": true
          }
        }
      }
    }
  ],
  "path": [
    {
      "name": "pc",
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "fixed",
      "rotateMode": "tangent",
      "position": 25,
      "spacing": 0,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_CHAIN_SPACING_ZERO_USES_TANGENT: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 0, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0 ],
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
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "fixed",
      "rotateMode": "chain",
      "position": 0,
      "spacing": 0,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_SPACING_MODE_LENGTH_TWO_BONES: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b1", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 5 },
    { "name": "b2", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 5 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 6.6666665, 0, 13.333333, 0, 20, 0, 20, 0 ],
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
      "order": 0,
      "bones": ["b1", "b2"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "length",
      "rotateMode": "chain",
      "position": 0,
      "spacing": 0,
      "mixRotate": 1,
      "mixX": 1,
      "mixY": 1
    }
  ],
  "animations": {}
}
"#;

const SKELETON_PATH_CONSTRAINT_SOLVE_MIX_ROTATE_PARTIAL: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root", "x": 0, "y": 0, "rotation": 90, "length": 1 }
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
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 3.3333333, 0, 6.6666665, 0, 10, 0, 10, 0 ],
            "lengths": [ 10 ],
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
      "order": 0,
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "fixed",
      "rotateMode": "tangent",
      "position": 0,
      "spacing": 0,
      "mixRotate": 0.5,
      "mixX": 0,
      "mixY": 0
    }
  ],
  "animations": {}
}
"#;

fn assert_approx_eps(actual: f32, expected: f32, eps: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= eps,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

fn assert_approx(actual: f32, expected: f32) {
    assert_approx_eps(actual, expected, 1.0e-2);
}

fn wrap_pi(mut radians: f32) -> f32 {
    const PI: f32 = std::f32::consts::PI;
    const PI2: f32 = std::f32::consts::PI * 2.0;
    if radians > PI {
        radians -= PI2;
    } else if radians < -PI {
        radians += PI2;
    }
    radians
}

fn assert_path_constraint_solve(json: &str) {
    let data = SkeletonData::from_json_str(json).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let bone = &skeleton.bones[1];
    assert_approx(bone.world_x, 5.0);
    assert_approx(bone.world_y, 0.0);

    let angle = wrap_pi(bone.c.atan2(bone.a));
    assert_approx(angle, 0.0);
}

#[test]
fn path_constraint_solve_constant_speed_true() {
    assert_path_constraint_solve(SKELETON_PATH_CONSTRAINT_SOLVE_CONSTANT_SPEED_TRUE);
}

#[test]
fn path_constraint_solve_constant_speed_false() {
    assert_path_constraint_solve(SKELETON_PATH_CONSTRAINT_SOLVE_CONSTANT_SPEED_FALSE);
}

#[test]
fn path_constraint_solve_position_percent_spacing_percent() {
    let data = SkeletonData::from_json_str(
        SKELETON_PATH_CONSTRAINT_SOLVE_POSITION_PERCENT_SPACING_PERCENT,
    )
    .unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let bone = &skeleton.bones[1];
    assert_approx(bone.world_x, 5.0);
    assert_approx(bone.world_y, 0.0);

    let angle = wrap_pi(bone.c.atan2(bone.a));
    assert_approx(angle, 0.0);
}

#[test]
fn path_constraint_solve_rotate_mode_chain_two_bones() {
    let data =
        SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_ROTATE_MODE_CHAIN_TWO_BONES)
            .unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let b1 = &skeleton.bones[1];
    let b2 = &skeleton.bones[2];
    let l1 = skeleton.data.bones[1].length;
    let l2 = skeleton.data.bones[2].length;

    assert_approx(b1.world_x, 0.0);
    assert_approx(b1.world_y, 0.0);
    let a1 = wrap_pi(b1.c.atan2(b1.a));
    assert_approx(a1, 0.0);
    let tip1_x = b1.a * l1 + b1.world_x;
    let tip1_y = b1.c * l1 + b1.world_y;
    assert_approx(tip1_x, 5.0);
    assert_approx(tip1_y, 0.0);

    assert_approx(b2.world_x, 5.0);
    assert_approx(b2.world_y, 0.0);
    let a2 = wrap_pi(b2.c.atan2(b2.a));
    assert_approx(a2, 0.0);
    let tip2_x = b2.a * l2 + b2.world_x;
    let tip2_y = b2.c * l2 + b2.world_y;
    assert_approx(tip2_x, 10.0);
    assert_approx(tip2_y, 0.0);
}

#[test]
fn path_constraint_solve_rotate_mode_chain_scale_scales_along_path() {
    let data = SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_ROTATE_MODE_CHAIN_SCALE)
        .unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let bone = &skeleton.bones[1];
    assert_approx(bone.a, 2.0);
    assert_approx(bone.c, 0.0);
}

#[test]
fn path_constraint_solve_spacing_proportional_two_bones() {
    let data =
        SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_SPACING_PROPORTIONAL_TWO_BONES)
            .unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let b1 = &skeleton.bones[1];
    let b2 = &skeleton.bones[2];

    assert_approx(b1.world_x, 0.0);
    assert_approx(b1.world_y, 0.0);
    assert_approx(b2.world_x, 10.0);
    assert_approx(b2.world_y, 0.0);

    let a1 = wrap_pi(b1.c.atan2(b1.a));
    let a2 = wrap_pi(b2.c.atan2(b2.a));
    assert_approx(a1, 0.0);
    assert_approx(a2, 0.0);
}

#[test]
fn path_constraint_solve_closed_wraps_position() {
    let data = SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_CLOSED_WRAP).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let bone = &skeleton.bones[1];
    assert_approx_eps(bone.world_x, 5.0, 2.0e-1);
    assert_approx_eps(bone.world_y, 0.0, 2.0e-1);

    let angle = wrap_pi(bone.c.atan2(bone.a));
    assert_approx_eps(angle, 0.0, 2.0e-1);
}

#[test]
fn path_constraint_mix_zero_disables_effect() {
    let data =
        SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_CONSTANT_SPEED_TRUE).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();

    skeleton.path_constraints[0].mix_rotate = 0.0;
    skeleton.path_constraints[0].mix_x = 0.0;
    skeleton.path_constraints[0].mix_y = 0.0;

    skeleton.update_world_transform_with_physics(crate::Physics::None);
    let after = &skeleton.bones[1];

    assert_approx(after.world_x, 0.0);
    assert_approx(after.world_y, 0.0);

    let angle = wrap_pi(after.c.atan2(after.a));
    assert_approx(angle, std::f32::consts::FRAC_PI_2);
}

#[test]
fn path_constraint_chain_spacing_zero_uses_tangent_angle() {
    let data =
        SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_CHAIN_SPACING_ZERO_USES_TANGENT)
            .unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let bone = &skeleton.bones[1];
    assert_approx(bone.world_x, 0.0);
    assert_approx(bone.world_y, 0.0);

    let angle = wrap_pi(bone.c.atan2(bone.a));
    assert_approx_eps(angle, std::f32::consts::FRAC_PI_2, 2.0e-1);
}

#[test]
fn path_constraint_spacing_mode_length_two_bones() {
    let data =
        SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_SPACING_MODE_LENGTH_TWO_BONES)
            .unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let b1 = &skeleton.bones[1];
    let b2 = &skeleton.bones[2];
    assert_approx(b1.world_x, 0.0);
    assert_approx(b1.world_y, 0.0);
    assert_approx(b2.world_x, 5.0);
    assert_approx(b2.world_y, 0.0);
}

#[test]
fn path_constraint_mix_rotate_partial_rotates_halfway() {
    let data =
        SkeletonData::from_json_str(SKELETON_PATH_CONSTRAINT_SOLVE_MIX_ROTATE_PARTIAL).unwrap();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let bone = &skeleton.bones[1];
    let angle = wrap_pi(bone.c.atan2(bone.a));
    assert_approx_eps(angle, std::f32::consts::FRAC_PI_4, 2.0e-1);
}
