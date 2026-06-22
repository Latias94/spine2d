use crate::{
    BlendMode, BoneData, IkConstraint, Inherit, PathConstraint, PhysicsConstraint, ScaleYMode,
    Skeleton, SkeletonData, SliderConstraintData, SlotData, TransformConstraint,
};
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
fn slot_accessors_expose_attachment_tint_and_deform_state() {
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
        slots: vec![SlotData {
            name: "slot".to_string(),
            bone: 0,
            attachment: None,
            ..Default::default()
        }],
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
    let slot = &mut skeleton.slots_mut()[0];

    assert_eq!(slot.bone_index(), 0);
    slot.set_bone_index(0);
    assert_eq!(slot.bone_index(), 0);

    slot.set_color([0.1, 0.2, 0.3, 0.4]);
    slot.set_has_dark(true);
    slot.set_dark_color([0.5, 0.6, 0.7]);
    slot.set_blend(BlendMode::Additive);
    assert_eq!(slot.color(), [0.1, 0.2, 0.3, 0.4]);
    assert!(slot.has_dark());
    assert_eq!(slot.dark_color(), [0.5, 0.6, 0.7]);
    assert_eq!(slot.blend(), BlendMode::Additive);

    slot.set_sequence_index(4);
    slot.deform_mut().extend_from_slice(&[1.0, 2.0, 3.0]);
    slot.set_attachment_name(Some("mesh".to_string()));
    assert_eq!(slot.attachment_name(), Some("mesh"));
    assert_eq!(slot.sequence_index(), -1);
    assert!(slot.deform().is_empty());

    slot.set_sequence_index(6);
    slot.deform_mut().extend_from_slice(&[9.0]);
    slot.set_attachment_name(Some("mesh".to_string()));
    assert_eq!(slot.sequence_index(), 6);
    assert_eq!(slot.deform(), &[9.0]);
}

#[test]
fn constraint_accessors_expose_pose_state() {
    let mut ik = IkConstraint {
        data_index: 1,
        bones: vec![0],
        target: 2,
        scale_y_mode: ScaleYMode::None,
        mix: 0.25,
        softness: 1.0,
        compress: false,
        stretch: true,
        bend_direction: 1,
        active: true,
    };
    assert_eq!(ik.data_index(), 1);
    ik.bones_mut().push(1);
    ik.set_target(3);
    ik.set_scale_y_mode(ScaleYMode::Volume);
    ik.set_mix(0.5);
    ik.set_softness(2.0);
    ik.set_compress(true);
    ik.set_stretch(false);
    ik.set_bend_direction(-1);
    ik.set_active(false);
    assert_eq!(ik.bones(), &[0, 1]);
    assert_eq!(ik.target(), 3);
    assert_eq!(ik.scale_y_mode(), ScaleYMode::Volume);
    assert_eq!(ik.mix(), 0.5);
    assert_eq!(ik.softness(), 2.0);
    assert!(ik.compress());
    assert!(!ik.stretch());
    assert_eq!(ik.bend_direction(), -1);
    assert!(!ik.is_active());

    let mut transform = TransformConstraint {
        data_index: 2,
        bones: vec![1],
        source: 0,
        mix_rotate: 0.1,
        mix_x: 0.2,
        mix_y: 0.3,
        mix_scale_x: 0.4,
        mix_scale_y: 0.5,
        mix_shear_y: 0.6,
        active: true,
    };
    transform.bones_mut().push(2);
    transform.set_source(3);
    transform.set_mix_rotate(1.1);
    transform.set_mix_x(1.2);
    transform.set_mix_y(1.3);
    transform.set_mix_scale_x(1.4);
    transform.set_mix_scale_y(1.5);
    transform.set_mix_shear_y(1.6);
    transform.set_active(false);
    assert_eq!(transform.data_index(), 2);
    assert_eq!(transform.bones(), &[1, 2]);
    assert_eq!(transform.source(), 3);
    assert_eq!(transform.mix_rotate(), 1.1);
    assert_eq!(transform.mix_x(), 1.2);
    assert_eq!(transform.mix_y(), 1.3);
    assert_eq!(transform.mix_scale_x(), 1.4);
    assert_eq!(transform.mix_scale_y(), 1.5);
    assert_eq!(transform.mix_shear_y(), 1.6);
    assert!(!transform.is_active());

    let mut path = PathConstraint {
        data_index: 3,
        bones: vec![0],
        target: 1,
        position: 2.0,
        spacing: 3.0,
        mix_rotate: 4.0,
        mix_x: 5.0,
        mix_y: 6.0,
        active: true,
    };
    path.bones_mut().push(1);
    path.set_target_slot(2);
    path.set_position(7.0);
    path.set_spacing(8.0);
    path.set_mix_rotate(9.0);
    path.set_mix_x(10.0);
    path.set_mix_y(11.0);
    path.set_active(false);
    assert_eq!(path.data_index(), 3);
    assert_eq!(path.bones(), &[0, 1]);
    assert_eq!(path.target_slot(), 2);
    assert_eq!(path.position(), 7.0);
    assert_eq!(path.spacing(), 8.0);
    assert_eq!(path.mix_rotate(), 9.0);
    assert_eq!(path.mix_x(), 10.0);
    assert_eq!(path.mix_y(), 11.0);
    assert!(!path.is_active());

    let mut physics = PhysicsConstraint {
        data_index: 4,
        bone: 0,
        inertia: 0.1,
        strength: 0.2,
        damping: 0.3,
        mass_inverse: 0.4,
        wind: 0.5,
        gravity: 0.6,
        mix: 0.7,
        scale_y_mode: ScaleYMode::None,
        reset: false,
        ux: 0.0,
        uy: 0.0,
        cx: 0.0,
        cy: 0.0,
        tx: 0.0,
        ty: 0.0,
        x_offset: 0.0,
        x_lag: 0.0,
        x_velocity: 0.0,
        y_offset: 0.0,
        y_lag: 0.0,
        y_velocity: 0.0,
        rotate_offset: 0.0,
        rotate_lag: 0.0,
        rotate_velocity: 0.0,
        scale_offset: 0.0,
        scale_lag: 0.0,
        scale_velocity: 0.0,
        active: true,
        remaining: 0.0,
        last_time: 0.0,
    };
    physics.set_bone_index(2);
    physics.set_inertia(1.1);
    physics.set_strength(1.2);
    physics.set_damping(1.3);
    physics.set_mass_inverse(1.4);
    physics.set_wind(1.5);
    physics.set_gravity(1.6);
    physics.set_mix(1.7);
    physics.set_scale_y_mode(ScaleYMode::Uniform);
    physics.set_active(false);
    assert_eq!(physics.data_index(), 4);
    assert_eq!(physics.bone_index(), 2);
    assert_eq!(physics.inertia(), 1.1);
    assert_eq!(physics.strength(), 1.2);
    assert_eq!(physics.damping(), 1.3);
    assert_eq!(physics.mass_inverse(), 1.4);
    assert_eq!(physics.wind(), 1.5);
    assert_eq!(physics.gravity(), 1.6);
    assert_eq!(physics.mix(), 1.7);
    assert_eq!(physics.scale_y_mode(), ScaleYMode::Uniform);
    assert!(!physics.is_active());

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
        slider_constraints: vec![SliderConstraintData {
            name: "slider".to_string(),
            order: 0,
            skin_required: false,
            setup_time: 0.0,
            setup_mix: 1.0,
            additive: false,
            looped: false,
            bone: None,
            property: None,
            property_from: 0.0,
            to: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
        }],
    });
    let mut skeleton = Skeleton::new(data);
    let slider = &mut skeleton.slider_constraints_mut()[0];
    slider.set_time(2.5);
    slider.set_mix(0.75);
    slider.set_active(false);
    assert_eq!(slider.data_index(), 0);
    assert_eq!(slider.time(), 2.5);
    assert_eq!(slider.mix(), 0.75);
    assert!(!slider.is_active());
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
