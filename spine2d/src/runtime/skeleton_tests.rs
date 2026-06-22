use crate::{BoneData, Inherit, Skeleton, SkeletonData};
use std::collections::HashMap;
use std::sync::Arc;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-6,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

fn empty_skeleton_data() -> Arc<SkeletonData> {
    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: Vec::new(),
        slots: Vec::new(),
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    })
}

#[test]
fn skeleton_world_controls_are_public_and_ignore_non_finite_inputs() {
    let mut skeleton = Skeleton::new(empty_skeleton_data());

    assert_eq!(skeleton.wind(), (1.0, 0.0));
    assert_eq!(skeleton.gravity(), (0.0, 1.0));
    assert_eq!(skeleton.time(), 0.0);

    skeleton.set_wind(2.0, 3.0);
    skeleton.set_gravity(4.0, 5.0);
    skeleton.set_time(1.5);

    assert_eq!(skeleton.wind(), (2.0, 3.0));
    assert_eq!(skeleton.gravity(), (4.0, 5.0));
    assert_eq!(skeleton.time(), 1.5);

    skeleton.set_wind(f32::NAN, 6.0);
    skeleton.set_gravity(7.0, f32::INFINITY);
    skeleton.set_time(f32::NAN);
    skeleton.update(-1.0);

    assert_eq!(skeleton.wind(), (2.0, 3.0));
    assert_eq!(skeleton.gravity(), (4.0, 5.0));
    assert_eq!(skeleton.time(), 1.5);

    skeleton.update(0.25);
    assert_eq!(skeleton.time(), 1.75);
}

#[test]
fn skeleton_accessors_expose_runtime_controls_without_public_vec_fields() {
    let mut skeleton = Skeleton::new(empty_skeleton_data());

    assert_eq!(skeleton.data().bones.len(), 0);
    assert_eq!(skeleton.bones().len(), 0);
    assert_eq!(skeleton.bones_mut().len(), 0);
    assert_eq!(skeleton.slots().len(), 0);
    assert_eq!(skeleton.slots_mut().len(), 0);
    assert!(skeleton.draw_order().is_empty());
    assert!(skeleton.draw_order_mut().is_empty());
    assert_eq!(skeleton.skin(), None);
    assert_eq!(skeleton.ik_constraints().len(), 0);
    assert_eq!(skeleton.ik_constraints_mut().len(), 0);
    assert_eq!(skeleton.transform_constraints().len(), 0);
    assert_eq!(skeleton.transform_constraints_mut().len(), 0);
    assert_eq!(skeleton.path_constraints().len(), 0);
    assert_eq!(skeleton.path_constraints_mut().len(), 0);
    assert_eq!(skeleton.physics_constraints().len(), 0);
    assert_eq!(skeleton.physics_constraints_mut().len(), 0);
    assert_eq!(skeleton.slider_constraints().len(), 0);
    assert_eq!(skeleton.slider_constraints_mut().len(), 0);

    assert_eq!(skeleton.color(), [1.0, 1.0, 1.0, 1.0]);
    skeleton.set_color([0.25, 0.5, 0.75, 0.875]);
    assert_eq!(skeleton.color(), [0.25, 0.5, 0.75, 0.875]);

    assert_eq!(skeleton.position(), (0.0, 0.0));
    skeleton.set_position(10.0, -2.0);
    assert_eq!(skeleton.position(), (10.0, -2.0));
    assert_eq!(skeleton.x(), 10.0);
    assert_eq!(skeleton.y(), -2.0);

    skeleton.set_x(3.0);
    skeleton.set_y(4.0);
    assert_eq!(skeleton.position(), (3.0, 4.0));

    assert_eq!(skeleton.scale(), (1.0, 1.0));
    skeleton.set_scale(2.0, -3.0);
    assert_eq!(skeleton.scale(), (2.0, -3.0));
    assert_eq!(skeleton.scale_x(), 2.0);
    assert_eq!(skeleton.scale_y(), -3.0);

    skeleton.set_scale_x(5.0);
    skeleton.set_scale_y(6.0);
    assert_eq!(skeleton.scale(), (5.0, 6.0));
}

#[test]
fn bone_accessors_expose_local_applied_and_world_pose() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: Vec::new(),
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    let bone = &mut skeleton.bones_mut()[0];

    assert!(bone.is_active());
    bone.set_active(false);
    assert!(!bone.is_active());

    bone.set_inherit(Inherit::OnlyTranslation);
    bone.set_position(1.0, 2.0);
    bone.set_rotation(3.0);
    bone.set_scale(4.0, 5.0);
    bone.set_shear_x(6.0);
    bone.set_shear_y(7.0);
    assert_eq!(bone.inherit(), Inherit::OnlyTranslation);
    assert_eq!(bone.position(), (1.0, 2.0));
    assert_eq!(bone.rotation(), 3.0);
    assert_eq!(bone.scale(), (4.0, 5.0));
    assert_eq!(bone.shear_x(), 6.0);
    assert_eq!(bone.shear_y(), 7.0);

    bone.set_applied_position(8.0, 9.0);
    bone.set_applied_rotation(10.0);
    bone.set_applied_scale(11.0, 12.0);
    bone.set_applied_shear(13.0, 14.0);
    assert_eq!(bone.applied_position(), (8.0, 9.0));
    assert_eq!(bone.applied_rotation(), 10.0);
    assert_eq!(bone.applied_scale(), (11.0, 12.0));
    assert_eq!(bone.applied_shear(), (13.0, 14.0));

    bone.set_a(3.0);
    bone.set_b(0.0);
    bone.set_c(4.0);
    bone.set_d(2.0);
    bone.set_world_position(15.0, 16.0);
    assert_eq!(bone.a(), 3.0);
    assert_eq!(bone.b(), 0.0);
    assert_eq!(bone.c(), 4.0);
    assert_eq!(bone.d(), 2.0);
    assert_eq!(bone.world_position(), (15.0, 16.0));
    assert_approx(bone.world_scale_x(), 5.0);
    assert_approx(bone.world_scale_y(), 2.0);
    assert_approx(bone.world_rotation_x(), 4.0f32.atan2(3.0).to_degrees());
    assert_approx(bone.world_rotation_y(), 90.0);
}

#[test]
fn update_world_transform_root_and_child() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![
            BoneData {
                name: "root".to_string(),
                parent: None,
                length: 0.0,
                x: 10.0,
                y: 20.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shear_x: 0.0,
                shear_y: 0.0,
                inherit: Inherit::Normal,
                skin_required: false,
                ..Default::default()
            },
            BoneData {
                name: "child".to_string(),
                parent: Some(0),
                length: 0.0,
                x: 5.0,
                y: 0.0,
                rotation: 90.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shear_x: 0.0,
                shear_y: 0.0,
                inherit: Inherit::Normal,
                skin_required: false,
                ..Default::default()
            },
        ],
        slots: Vec::new(),
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.update_world_transform();

    let root = &skeleton.bones[0];
    assert_approx(root.world_x, 10.0);
    assert_approx(root.world_y, 20.0);
    assert_approx(root.a, 1.0);
    assert_approx(root.b, 0.0);
    assert_approx(root.c, 0.0);
    assert_approx(root.d, 1.0);

    let child = &skeleton.bones[1];
    assert_approx(child.world_x, 15.0);
    assert_approx(child.world_y, 20.0);
    assert_approx(child.a, 0.0);
    assert_approx(child.b, -1.0);
    assert_approx(child.c, 1.0);
    assert_approx(child.d, 0.0);
}

#[test]
fn update_world_transform_parent_rotation_affects_child_translation() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![
            BoneData {
                name: "root".to_string(),
                parent: None,
                length: 0.0,
                x: 0.0,
                y: 0.0,
                rotation: 90.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shear_x: 0.0,
                shear_y: 0.0,
                inherit: Inherit::Normal,
                skin_required: false,
                ..Default::default()
            },
            BoneData {
                name: "child".to_string(),
                parent: Some(0),
                length: 0.0,
                x: 1.0,
                y: 0.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                shear_x: 0.0,
                shear_y: 0.0,
                inherit: Inherit::Normal,
                skin_required: false,
                ..Default::default()
            },
        ],
        slots: Vec::new(),
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.update_world_transform();

    let child = &skeleton.bones[1];
    assert_approx(child.world_x, 0.0);
    assert_approx(child.world_y, 1.0);
}
