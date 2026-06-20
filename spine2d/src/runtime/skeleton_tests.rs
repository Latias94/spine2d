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
