use crate::{
    AttachmentData, AttachmentFrame, AttachmentTimeline, BlendMode, Bone, BoneData,
    ClippingAttachmentData, ColorFrame, ColorTimeline, ConstraintRef, Curve, DrawOrderFrame,
    DrawOrderTimeline, IkConstraint, IkConstraintData, Inherit, MeshAttachmentData, MeshVertices,
    PathConstraint, PathConstraintData, PhysicsConstraint, PhysicsConstraintData, PositionMode,
    RegionAttachmentData, RotateMode, ScaleYMode, Skeleton, SkeletonData, SkinData,
    SliderConstraintData, SlotData, SpacingMode, TransformConstraint, TransformConstraintData,
    UpdateCacheItem,
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

fn assert_approx_pair(actual: (f32, f32), expected: (f32, f32)) {
    assert_approx(actual.0, expected.0);
    assert_approx(actual.1, expected.1);
}

fn assert_approx_angle(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-5,
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

fn region_attachment(name: &str, color: [f32; 4]) -> AttachmentData {
    AttachmentData::Region(RegionAttachmentData {
        name: name.to_string(),
        path: name.to_string(),
        sequence: None,
        color,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        width: 1.0,
        height: 1.0,
    })
}

fn named_attachment_skeleton_data() -> Arc<SkeletonData> {
    let mut default_skin = SkinData::new("default", 2);
    default_skin.attachments[0].insert(
        "shared".to_string(),
        region_attachment("default-shared", [1.0, 0.0, 0.0, 1.0]),
    );
    default_skin.attachments[1].insert(
        "fallback".to_string(),
        region_attachment("default-fallback", [0.0, 1.0, 0.0, 1.0]),
    );

    let mut custom_skin = SkinData::new("custom", 2);
    custom_skin.attachments[0].insert(
        "shared".to_string(),
        region_attachment("custom-shared", [0.0, 0.0, 1.0, 1.0]),
    );

    let mut skins = HashMap::new();
    skins.insert("default".to_string(), default_skin);
    skins.insert("custom".to_string(), custom_skin);

    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![
            BoneData {
                name: "root".to_string(),
                parent: None,
                length: 0.0,
                skin_required: false,
                ..Default::default()
            },
            BoneData {
                name: "child".to_string(),
                parent: Some(0),
                length: 0.0,
                skin_required: false,
                ..Default::default()
            },
        ],
        slots: vec![
            SlotData {
                name: "slot0".to_string(),
                bone: 0,
                attachment: Some("shared".to_string()),
                ..Default::default()
            },
            SlotData {
                name: "slot1".to_string(),
                bone: 1,
                attachment: None,
                ..Default::default()
            },
        ],
        skins,
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

fn physics_constraint_data(name: &str, order: i32, bone: usize) -> PhysicsConstraintData {
    PhysicsConstraintData {
        name: name.to_string(),
        order,
        skin_required: false,
        bone,
        x: 0.0,
        y: 0.0,
        rotate: 0.0,
        scale_x: 0.0,
        scale_y_mode: ScaleYMode::None,
        shear_x: 0.0,
        limit: 0.0,
        step: 0.0,
        inertia: 0.0,
        strength: 0.0,
        damping: 0.0,
        mass_inverse: 0.0,
        wind: 0.0,
        gravity: 0.0,
        mix: 1.0,
        inertia_global: false,
        strength_global: false,
        damping_global: false,
        mass_global: false,
        wind_global: false,
        gravity_global: false,
        mix_global: false,
    }
}

fn constraint_lookup_skeleton_data() -> Arc<SkeletonData> {
    Arc::new(SkeletonData {
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
        ik_constraints: vec![IkConstraintData {
            name: "ik".to_string(),
            order: 0,
            skin_required: false,
            bones: vec![0],
            target: 0,
            scale_y_mode: ScaleYMode::None,
            mix: 1.0,
            softness: 0.0,
            compress: false,
            stretch: false,
            bend_direction: 1,
        }],
        transform_constraints: vec![TransformConstraintData {
            name: "transform".to_string(),
            order: 1,
            skin_required: false,
            bones: vec![0],
            source: 0,
            local_source: false,
            local_target: false,
            additive: false,
            clamp: false,
            offsets: [0.0; 6],
            properties: Vec::new(),
            mix_rotate: 1.0,
            mix_x: 1.0,
            mix_y: 1.0,
            mix_scale_x: 1.0,
            mix_scale_y: 1.0,
            mix_shear_y: 1.0,
        }],
        path_constraints: vec![PathConstraintData {
            name: "path".to_string(),
            order: 2,
            bones: vec![0],
            target: 0,
            position_mode: PositionMode::Fixed,
            spacing_mode: SpacingMode::Length,
            rotate_mode: RotateMode::Tangent,
            offset_rotation: 0.0,
            position: 0.0,
            spacing: 0.0,
            mix_rotate: 1.0,
            mix_x: 1.0,
            mix_y: 1.0,
            skin_required: false,
        }],
        physics_constraints: vec![physics_constraint_data("physics", 3, 0)],
        slider_constraints: vec![SliderConstraintData {
            name: "slider".to_string(),
            order: 4,
            skin_required: false,
            setup_time: 0.0,
            setup_mix: 1.0,
            additive: false,
            looped: false,
            bone: Some(0),
            property: None,
            property_from: 0.0,
            to: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
        }],
    })
}

fn slider_draw_order_skeleton_data() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "slider-draw".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: Vec::new(),
        slot_color_timelines: Vec::new(),
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: Some(DrawOrderTimeline {
            frames: vec![DrawOrderFrame {
                time: 0.0,
                draw_order_to_setup_index: Some(vec![1, 0]),
            }],
        }),
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut animation_index = HashMap::new();
    animation_index.insert("slider-draw".to_string(), 0);

    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: vec![
            SlotData {
                name: "slot0".to_string(),
                bone: 0,
                attachment: None,
                ..Default::default()
            },
            SlotData {
                name: "slot1".to_string(),
                bone: 0,
                attachment: None,
                ..Default::default()
            },
        ],
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: vec![animation],
        animation_index,
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
            animation: Some(0),
        }],
    })
}

fn slider_slot_pose_skeleton_data() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "slider-slot-pose".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: vec![AttachmentFrame {
                time: 0.0,
                name: Some("alt".to_string()),
            }],
        }],
        slot_color_timelines: vec![ColorTimeline {
            slot_index: 0,
            frames: vec![ColorFrame {
                time: 0.0,
                color: [0.2, 0.3, 0.4, 0.5],
                curve: [Curve::Linear; 4],
            }],
        }],
        slot_rgb_timelines: Vec::new(),
        slot_alpha_timelines: Vec::new(),
        slot_rgba2_timelines: Vec::new(),
        slot_rgb2_timelines: Vec::new(),
        ik_constraint_timelines: Vec::new(),
        transform_constraint_timelines: Vec::new(),
        path_constraint_timelines: Vec::new(),
        physics_constraint_timelines: Vec::new(),
        physics_reset_timelines: Vec::new(),
        slider_time_timelines: Vec::new(),
        slider_mix_timelines: Vec::new(),
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut animation_index = HashMap::new();
    animation_index.insert("slider-slot-pose".to_string(), 0);

    let mut default_skin = SkinData::new("default", 1);
    default_skin.attachments[0].insert(
        "base".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "base".to_string(),
            path: "base".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 2.0,
            height: 2.0,
        }),
    );
    default_skin.attachments[0].insert(
        "alt".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "alt".to_string(),
            path: "alt".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 2.0,
            height: 2.0,
        }),
    );

    let mut skins = HashMap::new();
    skins.insert("default".to_string(), default_skin);

    Arc::new(SkeletonData {
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
            name: "slot0".to_string(),
            bone: 0,
            attachment: Some("base".to_string()),
            color: [0.1, 0.2, 0.3, 0.4],
            ..Default::default()
        }],
        skins,
        events: HashMap::new(),
        animations: vec![animation],
        animation_index,
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
            animation: Some(0),
        }],
    })
}

fn setup_pose_split_skeleton_data() -> Arc<SkeletonData> {
    let mut physics = physics_constraint_data("physics", 3, 0);
    physics.inertia = 0.1;
    physics.strength = 0.2;
    physics.damping = 0.3;
    physics.mass_inverse = 0.4;
    physics.wind = 0.5;
    physics.gravity = 0.6;
    physics.mix = 0.7;

    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            x: 1.0,
            y: 2.0,
            rotation: 3.0,
            scale_x: 4.0,
            scale_y: 5.0,
            shear_x: 6.0,
            shear_y: 7.0,
            ..Default::default()
        }],
        slots: vec![
            SlotData {
                name: "slot0".to_string(),
                bone: 0,
                attachment: None,
                color: [0.1, 0.2, 0.3, 0.4],
                has_dark: true,
                dark_color: [0.5, 0.6, 0.7],
                blend: BlendMode::Additive,
                visible: true,
            },
            SlotData {
                name: "slot1".to_string(),
                bone: 0,
                attachment: None,
                ..Default::default()
            },
        ],
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: vec![IkConstraintData {
            name: "ik".to_string(),
            order: 0,
            skin_required: false,
            bones: vec![0],
            target: 0,
            scale_y_mode: ScaleYMode::Volume,
            mix: 0.8,
            softness: 0.9,
            compress: true,
            stretch: true,
            bend_direction: -1,
        }],
        transform_constraints: vec![TransformConstraintData {
            name: "transform".to_string(),
            order: 1,
            skin_required: false,
            bones: vec![0],
            source: 0,
            local_source: false,
            local_target: false,
            additive: false,
            clamp: false,
            offsets: [0.0; 6],
            properties: Vec::new(),
            mix_rotate: 0.11,
            mix_x: 0.12,
            mix_y: 0.13,
            mix_scale_x: 0.14,
            mix_scale_y: 0.15,
            mix_shear_y: 0.16,
        }],
        path_constraints: vec![PathConstraintData {
            name: "path".to_string(),
            order: 2,
            bones: vec![0],
            target: 0,
            position_mode: PositionMode::Fixed,
            spacing_mode: SpacingMode::Length,
            rotate_mode: RotateMode::Tangent,
            offset_rotation: 0.0,
            position: 1.1,
            spacing: 1.2,
            mix_rotate: 1.3,
            mix_x: 1.4,
            mix_y: 1.5,
            skin_required: false,
        }],
        physics_constraints: vec![physics],
        slider_constraints: vec![SliderConstraintData {
            name: "slider".to_string(),
            order: 4,
            skin_required: false,
            setup_time: 2.1,
            setup_mix: 2.2,
            additive: false,
            looped: false,
            bone: Some(0),
            property: None,
            property_from: 0.0,
            to: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
        }],
    })
}

fn clipping_bounds_skeleton_data() -> Arc<SkeletonData> {
    let mut default_skin = SkinData::new("default", 3);
    default_skin.attachments[0].insert(
        "clip".to_string(),
        AttachmentData::Clipping(ClippingAttachmentData {
            vertex_id: 1,
            name: "clip".to_string(),
            vertices: MeshVertices::Unweighted(vec![
                [-1.0, -1.0],
                [1.0, -1.0],
                [1.0, 1.0],
                [-1.0, 1.0],
            ]),
            end_slot: Some(1),
            convex: false,
            inverse: false,
        }),
    );
    default_skin.attachments[1].insert(
        "region0".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "region0".to_string(),
            path: "region0".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 4.0,
            height: 4.0,
        }),
    );
    default_skin.attachments[2].insert(
        "region1".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "region1".to_string(),
            path: "region1".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 4.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 2.0,
            height: 2.0,
        }),
    );

    let mut skins = HashMap::new();
    skins.insert("default".to_string(), default_skin);

    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: vec![
            SlotData {
                name: "clip".to_string(),
                bone: 0,
                attachment: Some("clip".to_string()),
                ..Default::default()
            },
            SlotData {
                name: "region0".to_string(),
                bone: 0,
                attachment: Some("region0".to_string()),
                ..Default::default()
            },
            SlotData {
                name: "region1".to_string(),
                bone: 0,
                attachment: Some("region1".to_string()),
                ..Default::default()
            },
        ],
        skins,
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
fn skeleton_world_controls_follow_cpp_direct_assignment() {
    let mut skeleton = Skeleton::new(empty_skeleton_data());

    assert_eq!(skeleton.wind(), (1.0, 0.0));
    assert_eq!(skeleton.wind_x(), 1.0);
    assert_eq!(skeleton.wind_y(), 0.0);
    assert_eq!(skeleton.gravity(), (0.0, 1.0));
    assert_eq!(skeleton.gravity_x(), 0.0);
    assert_eq!(skeleton.gravity_y(), 1.0);
    assert_eq!(skeleton.time(), 0.0);

    skeleton.set_wind_x(2.0);
    skeleton.set_wind_y(3.0);
    skeleton.set_gravity_x(4.0);
    skeleton.set_gravity_y(5.0);
    skeleton.set_time(1.5);

    assert_eq!(skeleton.wind(), (2.0, 3.0));
    assert_eq!(skeleton.wind_x(), 2.0);
    assert_eq!(skeleton.wind_y(), 3.0);
    assert_eq!(skeleton.gravity(), (4.0, 5.0));
    assert_eq!(skeleton.gravity_x(), 4.0);
    assert_eq!(skeleton.gravity_y(), 5.0);
    assert_eq!(skeleton.time(), 1.5);

    skeleton.set_wind(-2.0, 6.0);
    skeleton.set_gravity(7.0, -5.0);
    skeleton.set_time(0.5);
    skeleton.update(-1.0);

    assert_eq!(skeleton.wind(), (-2.0, 6.0));
    assert_eq!(skeleton.gravity(), (7.0, -5.0));
    assert_eq!(skeleton.time(), -0.5);

    skeleton.update(0.25);
    assert_eq!(skeleton.time(), -0.25);
}

#[test]
fn skeleton_setup_pose_methods_match_cpp_split() {
    let mut skeleton = Skeleton::new(setup_pose_split_skeleton_data());

    skeleton.bones_mut()[0].set_position(10.0, 20.0);
    skeleton.bones_mut()[0].set_rotation(30.0);
    skeleton.ik_constraints_mut()[0].set_mix(0.25);
    skeleton.ik_constraints_mut()[0].set_bend_direction(1);
    skeleton.transform_constraints_mut()[0].set_mix_x(0.35);
    skeleton.path_constraints_mut()[0].set_position(0.45);
    skeleton.physics_constraints_mut()[0].set_gravity(0.55);
    skeleton.slider_constraints_mut()[0].set_time(0.65);
    skeleton.slots_mut()[0].set_color([0.9, 0.8, 0.7, 0.6]);
    {
        let slot = &mut skeleton.slots_mut()[0];
        slot.attachment = Some("manual".to_string());
        slot.attachment_skin = None;
        slot.sequence_index = 7;
        slot.deform.extend_from_slice(&[1.0, 2.0]);
    }
    skeleton.draw_order_mut().swap(0, 1);

    skeleton.setup_pose_bones();

    assert_eq!(skeleton.bones()[0].position(), (1.0, 2.0));
    assert_eq!(skeleton.bones()[0].rotation(), 3.0);
    assert_eq!(skeleton.bones()[0].applied_position(), (1.0, 2.0));
    assert_eq!(skeleton.ik_constraints()[0].mix(), 0.8);
    assert_eq!(skeleton.ik_constraints()[0].bend_direction(), -1);
    assert_eq!(skeleton.transform_constraints()[0].mix_x(), 0.12);
    assert_eq!(skeleton.path_constraints()[0].position(), 1.1);
    assert_eq!(skeleton.physics_constraints()[0].gravity(), 0.6);
    assert_eq!(skeleton.slider_constraints()[0].time(), 2.1);
    assert_eq!(skeleton.slots()[0].color(), [0.9, 0.8, 0.7, 0.6]);
    assert_eq!(skeleton.slots()[0].attachment_name(), Some("manual"));
    assert_eq!(skeleton.slots()[0].sequence_index(), 7);
    assert_eq!(skeleton.draw_order(), &[1, 0]);

    skeleton.bones_mut()[0].set_position(40.0, 50.0);
    skeleton.ik_constraints_mut()[0].set_mix(0.33);

    skeleton.setup_pose_slots();

    assert_eq!(skeleton.bones()[0].position(), (40.0, 50.0));
    assert_eq!(skeleton.ik_constraints()[0].mix(), 0.33);
    assert_eq!(skeleton.slots()[0].color(), [0.1, 0.2, 0.3, 0.4]);
    assert!(skeleton.slots()[0].has_dark());
    assert_eq!(skeleton.slots()[0].dark_color(), [0.5, 0.6, 0.7]);
    assert_eq!(skeleton.slots()[0].blend(), BlendMode::Additive);
    assert_eq!(skeleton.slots()[0].attachment_name(), None);
    assert_eq!(skeleton.slots()[0].sequence_index(), -1);
    assert!(skeleton.slots()[0].deform().is_empty());
    assert_eq!(skeleton.draw_order(), &[0, 1]);

    skeleton.bones_mut()[0].set_position(70.0, 80.0);
    skeleton.ik_constraints_mut()[0].set_mix(0.44);
    skeleton.slots_mut()[0].set_color([0.2, 0.3, 0.4, 0.5]);
    skeleton.draw_order_mut().swap(0, 1);

    skeleton.setup_pose();

    assert_eq!(skeleton.bones()[0].position(), (1.0, 2.0));
    assert_eq!(skeleton.ik_constraints()[0].mix(), 0.8);
    assert_eq!(skeleton.slots()[0].color(), [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(skeleton.slots()[0].sequence_index(), 0);
    assert_eq!(skeleton.draw_order(), &[0, 1]);
}

#[test]
fn slider_draw_order_uses_applied_buffer_without_mutating_pose() {
    let mut skeleton = Skeleton::new(slider_draw_order_skeleton_data());

    assert_eq!(skeleton.draw_order_pose(), &[0, 1]);
    assert_eq!(skeleton.draw_order(), &[0, 1]);

    skeleton.update_world_transform();

    assert_eq!(skeleton.draw_order_pose(), &[0, 1]);
    assert_eq!(skeleton.draw_order(), &[1, 0]);

    skeleton.slider_constraints_mut()[0].set_mix(0.0);
    skeleton.update_world_transform();

    assert_eq!(skeleton.draw_order_pose(), &[0, 1]);
    assert_eq!(skeleton.draw_order(), &[0, 1]);
}

#[test]
fn slider_slot_pose_uses_applied_buffer_without_mutating_pose() {
    let mut skeleton = Skeleton::new(slider_slot_pose_skeleton_data());

    assert_eq!(skeleton.slots()[0].attachment_name(), Some("base"));
    assert_eq!(skeleton.slots()[0].applied_attachment_name(), Some("base"));
    assert_eq!(skeleton.slots()[0].color(), [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(skeleton.slots()[0].applied_color(), [0.1, 0.2, 0.3, 0.4]);

    skeleton.update_world_transform();

    assert_eq!(skeleton.slots()[0].attachment_name(), Some("base"));
    assert_eq!(skeleton.slots()[0].applied_attachment_name(), Some("alt"));
    assert_eq!(skeleton.slots()[0].color(), [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(skeleton.slots()[0].applied_color(), [0.2, 0.3, 0.4, 0.5]);
    assert_eq!(
        skeleton.slot_attachment_data(0).map(AttachmentData::name),
        Some("alt")
    );

    skeleton.slider_constraints_mut()[0].set_mix(0.0);
    skeleton.update_world_transform();

    assert_eq!(skeleton.slots()[0].attachment_name(), Some("base"));
    assert_eq!(skeleton.slots()[0].applied_attachment_name(), Some("base"));
    assert_eq!(skeleton.slots()[0].applied_color(), [0.1, 0.2, 0.3, 0.4]);
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
    assert_eq!(skeleton.skin_name(), None);
    assert!(skeleton.skin().is_none());
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
    assert!(skeleton.update_cache_items().is_empty());

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
fn skeleton_update_cache_items_expose_read_only_solver_order() {
    let skeleton = Skeleton::new(constraint_lookup_skeleton_data());

    assert_eq!(
        skeleton.update_cache_items(),
        &[
            UpdateCacheItem::Bone(0),
            UpdateCacheItem::Ik(0),
            UpdateCacheItem::Bone(0),
            UpdateCacheItem::Transform(0),
            UpdateCacheItem::Path(0),
            UpdateCacheItem::Physics(0),
            UpdateCacheItem::Slider(0),
        ]
    );
}

#[test]
fn skeleton_finders_match_setup_order() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());

    assert_eq!(skeleton.root_bone().unwrap().data_index(), 0);
    assert_eq!(skeleton.root_bone_mut().unwrap().data_index(), 0);
    assert_eq!(skeleton.find_bone_index("root"), Some(0));
    assert_eq!(skeleton.find_bone_index("child"), Some(1));
    assert!(skeleton.find_bone("").is_none());
    assert!(skeleton.find_slot("").is_none());
    assert_eq!(skeleton.find_bone("child").unwrap().parent_index(), Some(0));

    skeleton.root_bone_mut().unwrap().set_y(3.0);
    assert_eq!(skeleton.bones()[0].y(), 3.0);

    skeleton.find_bone_mut("child").unwrap().set_x(7.0);
    assert_eq!(skeleton.bones()[1].x(), 7.0);

    assert_eq!(skeleton.find_slot_index("slot1"), Some(1));
    assert_eq!(skeleton.find_slot("slot0").unwrap().bone_index(), 0);
    skeleton
        .find_slot_mut("slot1")
        .unwrap()
        .set_color([0.2, 0.3, 0.4, 0.5]);
    assert_eq!(skeleton.slots()[1].color(), [0.2, 0.3, 0.4, 0.5]);
}

#[test]
fn skeleton_constraint_finders_match_data_names() {
    let mut skeleton = Skeleton::new(constraint_lookup_skeleton_data());

    assert_eq!(skeleton.find_ik_constraint_index("ik"), Some(0));
    assert_eq!(skeleton.find_ik_constraint("ik").unwrap().data_index(), 0);
    skeleton.find_ik_constraint_mut("ik").unwrap().set_mix(0.25);
    assert_eq!(skeleton.ik_constraints()[0].mix(), 0.25);

    assert_eq!(
        skeleton.find_transform_constraint_index("transform"),
        Some(0)
    );
    assert_eq!(
        skeleton
            .find_transform_constraint("transform")
            .unwrap()
            .data_index(),
        0
    );
    skeleton
        .find_transform_constraint_mut("transform")
        .unwrap()
        .set_mix_x(0.5);
    assert_eq!(skeleton.transform_constraints()[0].mix_x(), 0.5);

    assert_eq!(skeleton.find_path_constraint_index("path"), Some(0));
    assert_eq!(
        skeleton.find_path_constraint("path").unwrap().data_index(),
        0
    );
    skeleton
        .find_path_constraint_mut("path")
        .unwrap()
        .set_position(2.0);
    assert_eq!(skeleton.path_constraints()[0].position(), 2.0);

    assert_eq!(skeleton.find_physics_constraint_index("physics"), Some(0));
    assert_eq!(
        skeleton
            .find_physics_constraint("physics")
            .unwrap()
            .data_index(),
        0
    );
    skeleton
        .find_physics_constraint_mut("physics")
        .unwrap()
        .set_mix(0.75);
    assert_eq!(skeleton.physics_constraints()[0].mix(), 0.75);

    assert_eq!(skeleton.find_slider_constraint_index("slider"), Some(0));
    assert_eq!(
        skeleton
            .find_slider_constraint("slider")
            .unwrap()
            .data_index(),
        0
    );
    skeleton
        .find_slider_constraint_mut("slider")
        .unwrap()
        .set_time(3.0);
    assert_eq!(skeleton.slider_constraints()[0].time(), 3.0);

    assert!(skeleton.find_ik_constraint("").is_none());
    assert!(skeleton.find_transform_constraint("missing").is_none());
    assert!(skeleton.find_path_constraint("").is_none());
    assert!(skeleton.find_physics_constraint("missing").is_none());
    assert!(skeleton.find_slider_constraint("").is_none());
}

#[test]
fn skeleton_constraints_expose_ordered_typed_constraint_refs() {
    let skeleton = Skeleton::new(constraint_lookup_skeleton_data());
    let constraints = skeleton.constraints();

    assert_eq!(constraints.len(), 5);
    assert!(constraints.iter().all(ConstraintRef::is_active));
    match constraints.as_slice() {
        [
            ConstraintRef::Ik(ik),
            ConstraintRef::Transform(transform),
            ConstraintRef::Path(path),
            ConstraintRef::Physics(physics),
            ConstraintRef::Slider(slider),
        ] => {
            assert_eq!(ik.data_index(), 0);
            assert_eq!(transform.data_index(), 0);
            assert_eq!(path.data_index(), 0);
            assert_eq!(physics.data_index(), 0);
            assert_eq!(slider.data_index(), 0);
        }
        other => panic!("unexpected constraint order: {other:?}"),
    }
}

#[test]
fn skeleton_attachment_lookup_prefers_current_skin_then_default_skin() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());

    assert_eq!(
        skeleton.attachment(0, "shared").unwrap().name(),
        "default-shared"
    );
    assert_eq!(
        skeleton
            .attachment_by_slot_name("slot0", "shared")
            .unwrap()
            .name(),
        "default-shared"
    );
    assert_eq!(
        skeleton
            .attachment_by_slot_name("slot1", "fallback")
            .unwrap()
            .name(),
        "default-fallback"
    );
    assert!(skeleton.attachment(0, "").is_none());
    assert!(skeleton.attachment_by_slot_name("", "shared").is_none());

    skeleton.set_skin(Some("missing"));
    assert_eq!(skeleton.skin_name(), None);
    assert!(skeleton.skin().is_none());
    assert_eq!(
        skeleton.attachment(0, "shared").unwrap().name(),
        "default-shared"
    );

    skeleton.set_skin(Some("custom"));
    assert_eq!(skeleton.skin_name(), Some("custom"));
    assert_eq!(
        skeleton.skin().map(|skin| skin.name.as_str()),
        Some("custom")
    );
    assert_eq!(
        skeleton.attachment(0, "shared").unwrap().name(),
        "custom-shared"
    );
    assert_eq!(
        skeleton
            .attachment_by_slot_name("slot0", "shared")
            .unwrap()
            .name(),
        "custom-shared"
    );
    assert_eq!(
        skeleton
            .attachment_by_slot_name("slot1", "fallback")
            .unwrap()
            .name(),
        "default-fallback"
    );
}

#[test]
fn skeleton_set_attachment_updates_source_skin_and_pose_state() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());

    skeleton.set_skin(Some("custom"));
    assert_eq!(
        skeleton.slots()[0].attachment_skin.as_deref(),
        Some("custom")
    );

    {
        let slot = &mut skeleton.slots_mut()[0];
        slot.deform_mut().extend_from_slice(&[1.0, 2.0]);
        slot.set_sequence_index(7);
    }

    skeleton.set_skin(None);
    assert!(skeleton.set_attachment("slot0", "shared"));
    assert_eq!(skeleton.slots()[0].attachment_name(), Some("shared"));
    assert_eq!(
        skeleton.slots()[0].attachment_skin.as_deref(),
        Some("default")
    );
    assert_eq!(skeleton.slots()[0].sequence_index(), -1);
    assert!(skeleton.slots()[0].deform().is_empty());

    {
        let slot = &mut skeleton.slots_mut()[0];
        slot.deform_mut().extend_from_slice(&[3.0]);
        slot.set_sequence_index(9);
    }

    assert!(!skeleton.set_attachment("slot0", "shared"));
    assert_eq!(
        skeleton.slots()[0].attachment_skin.as_deref(),
        Some("default")
    );
    assert_eq!(skeleton.slots()[0].sequence_index(), 9);
    assert_eq!(skeleton.slots()[0].deform(), &[3.0]);

    assert!(skeleton.set_attachment("slot0", ""));
    assert_eq!(skeleton.slots()[0].attachment_name(), None);
    assert_eq!(skeleton.slots()[0].attachment_skin, None);
    assert!(skeleton.slots()[0].deform().is_empty());
    assert_eq!(skeleton.slots()[0].sequence_index(), -1);
    assert!(!skeleton.set_attachment("missing", "shared"));
    assert!(!skeleton.set_attachment("", "shared"));
}

#[test]
fn skeleton_bounds_cover_region_and_mesh_attachments() {
    let mut default_skin = SkinData::new("default", 2);
    default_skin.attachments[0].insert(
        "region".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "region".to_string(),
            path: "region".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 2.0,
            height: 2.0,
        }),
    );
    default_skin.attachments[1].insert(
        "mesh".to_string(),
        AttachmentData::Mesh(MeshAttachmentData {
            vertex_id: 1,
            name: "mesh".to_string(),
            path: "mesh".to_string(),
            timeline_skin: "default".to_string(),
            timeline_attachment: "mesh".to_string(),
            timeline_slots: vec![1],
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            vertices: MeshVertices::Unweighted(vec![
                [3.0, 4.0],
                [5.0, 4.0],
                [5.0, 6.0],
                [3.0, 6.0],
            ]),
            uvs: vec![[0.0, 0.0]; 4],
            triangles: vec![0, 1, 2, 2, 3, 0],
        }),
    );

    let mut skins = HashMap::new();
    skins.insert("default".to_string(), default_skin);

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
        slots: vec![
            SlotData {
                name: "region-slot".to_string(),
                bone: 0,
                attachment: Some("region".to_string()),
                ..Default::default()
            },
            SlotData {
                name: "mesh-slot".to_string(),
                bone: 0,
                attachment: Some("mesh".to_string()),
                ..Default::default()
            },
        ],
        skins,
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
    assert_eq!(skeleton.bounds(), Some((-1.0, -1.0, 6.0, 7.0)));
}

#[test]
fn skeleton_bounds_with_clipping_respects_clip_polygons_and_end_slots() {
    let mut skeleton = Skeleton::new(clipping_bounds_skeleton_data());
    skeleton.setup_pose();
    skeleton.update_world_transform();

    assert_eq!(skeleton.bounds(), Some((-2.0, -2.0, 7.0, 4.0)));
    assert_eq!(
        skeleton.bounds_with_clipping(),
        Some((-1.0, -1.0, 6.0, 2.0))
    );
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
fn bone_y_down_switch_controls_skeleton_scale_y() {
    struct ResetYDown;

    impl Drop for ResetYDown {
        fn drop(&mut self) {
            Bone::set_y_down(false);
        }
    }

    let _reset = ResetYDown;
    Bone::set_y_down(false);

    let data = Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            x: 1.0,
            y: 2.0,
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

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_scale(2.0, 3.0);
    skeleton.update_world_transform();
    assert!(!Bone::is_y_down());
    assert_eq!(skeleton.scale(), (2.0, 3.0));
    assert_eq!(skeleton.scale_y(), 3.0);
    assert_approx_pair(skeleton.bones()[0].world_position(), (2.0, 6.0));
    assert_approx(skeleton.bones()[0].d(), 3.0);

    Bone::set_y_down(true);

    let mut skeleton = Skeleton::new(data);
    skeleton.set_scale(2.0, 3.0);
    skeleton.update_world_transform();
    assert!(Bone::is_y_down());
    assert_eq!(skeleton.scale(), (2.0, -3.0));
    assert_eq!(skeleton.scale_y(), -3.0);
    assert_approx_pair(skeleton.bones()[0].world_position(), (2.0, -6.0));
    assert_approx(skeleton.bones()[0].d(), -3.0);
}

#[test]
fn bone_child_indices_expose_skeleton_hierarchy() {
    let skeleton = Skeleton::new(named_attachment_skeleton_data());

    assert_eq!(skeleton.bones()[0].child_indices(&skeleton), &[1]);
    assert!(skeleton.bones()[1].child_indices(&skeleton).is_empty());
}

#[test]
fn bone_parent_space_helpers_follow_parent_world_transform() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    {
        let root = &mut skeleton.bones_mut()[0];
        root.set_a(2.0);
        root.set_b(0.25);
        root.set_c(0.5);
        root.set_d(3.0);
        root.set_world_position(10.0, 20.0);
    }

    let root = &skeleton.bones()[0];
    let child = &skeleton.bones()[1];

    assert_approx_pair(root.world_to_parent(&skeleton, 17.0, 29.0), (17.0, 29.0));
    assert_approx_pair(root.parent_to_world(&skeleton, 7.0, 9.0), (7.0, 9.0));

    let world = (18.25, 29.5);
    let parent = child.world_to_parent(&skeleton, world.0, world.1);
    assert_approx_pair(parent, root.world_to_local(world.0, world.1));
    assert_approx_pair(child.parent_to_world(&skeleton, parent.0, parent.1), world);
}

#[test]
fn bone_rotation_helpers_match_bone_pose_formulas() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    let bone = &mut skeleton.bones_mut()[0];

    bone.set_a(0.0);
    bone.set_b(-1.0);
    bone.set_c(1.0);
    bone.set_d(0.0);
    bone.set_applied_rotation(0.0);
    bone.set_applied_shear_x(0.0);

    assert_approx_angle(bone.local_to_world_rotation(0.0), 90.0);
    assert_approx_angle(bone.world_to_local_rotation(90.0), 0.0);

    bone.set_a(1.0);
    bone.set_b(0.0);
    bone.set_c(0.0);
    bone.set_d(1.0);
    bone.rotate_world(90.0);

    assert_approx(bone.a(), 0.0);
    assert_approx(bone.b(), -1.0);
    assert_approx(bone.c(), 1.0);
    assert_approx(bone.d(), 0.0);
}

#[test]
fn skeleton_bone_world_transform_helper_recomputes_modified_local_pose() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    skeleton.update_world_transform();

    skeleton.bones_mut()[0].set_applied_position(12.0, 34.0);
    skeleton.bones_mut()[0].set_applied_rotation(90.0);
    skeleton.modify_bone_local(0);
    skeleton.update_bone_world_transform(0);

    let bone = &skeleton.bones()[0];
    assert_approx_pair(bone.world_position(), (12.0, 34.0));
    assert_approx(bone.world_rotation_x(), 90.0);
}

#[test]
fn skeleton_bone_local_transform_helper_rebuilds_applied_pose_from_world() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    skeleton.set_position(2.0, 3.0);
    skeleton.update_world_transform();
    skeleton.modify_bone_local(0);

    let bone = &mut skeleton.bones_mut()[0];
    bone.set_world_position(12.0, 18.0);
    bone.set_a(0.0);
    bone.set_b(-1.0);
    bone.set_c(1.0);
    bone.set_d(0.0);

    skeleton.update_bone_local_transform(0);

    let bone = &skeleton.bones()[0];
    assert_approx_pair(bone.applied_position(), (10.0, 15.0));
    assert_approx(bone.applied_rotation(), 90.0);
    assert_approx_pair(bone.applied_scale(), (1.0, 1.0));
    assert_approx_pair(bone.applied_shear(), (0.0, 0.0));

    skeleton.bones_mut()[0].set_applied_position(99.0, 99.0);
    skeleton.update_bone_world_transform(0);
    assert_approx_pair(skeleton.bones()[0].world_position(), (12.0, 18.0));
}

#[test]
fn skeleton_validate_bone_local_transform_uses_world_modified_marker() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    skeleton.update_world_transform();

    skeleton.bones_mut()[0].set_world_position(21.0, 22.0);
    skeleton.validate_bone_local_transform(0);
    assert_approx_pair(skeleton.bones()[0].applied_position(), (0.0, 0.0));

    skeleton.modify_bone_world(0);
    skeleton.validate_bone_local_transform(0);
    assert_approx_pair(skeleton.bones()[0].applied_position(), (21.0, 22.0));
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
    assert_eq!(slot.sequence_index(), 4);
    assert_eq!(slot.deform(), &[1.0, 2.0, 3.0]);
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

    physics.translate(3.0, -2.0);
    assert_eq!(
        (physics.ux, physics.uy, physics.cx, physics.cy),
        (-3.0, 2.0, -3.0, 2.0)
    );

    physics.ux = 0.0;
    physics.uy = 0.0;
    physics.cx = 1.0;
    physics.cy = 0.0;
    physics.rotate(0.0, 0.0, 90.0);
    assert_approx(physics.ux, 1.0);
    assert_approx(physics.uy, -1.0);
    assert_approx(physics.cx, 2.0);
    assert_approx(physics.cy, -1.0);

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
fn skeleton_physics_controls_broadcast_to_all_constraints() {
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
        physics_constraints: vec![
            physics_constraint_data("physics-a", 0, 0),
            physics_constraint_data("physics-b", 1, 0),
        ],
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    {
        let constraints = skeleton.physics_constraints_mut();
        constraints[0].ux = 10.0;
        constraints[0].uy = 20.0;
        constraints[0].cx = 30.0;
        constraints[0].cy = 40.0;
        constraints[1].ux = -5.0;
        constraints[1].uy = -6.0;
        constraints[1].cx = -7.0;
        constraints[1].cy = -8.0;
    }

    skeleton.physics_translate(3.0, -2.0);
    assert_eq!(
        (
            skeleton.physics_constraints()[0].ux,
            skeleton.physics_constraints()[0].uy,
            skeleton.physics_constraints()[0].cx,
            skeleton.physics_constraints()[0].cy,
        ),
        (7.0, 22.0, 27.0, 42.0)
    );
    assert_eq!(
        (
            skeleton.physics_constraints()[1].ux,
            skeleton.physics_constraints()[1].uy,
            skeleton.physics_constraints()[1].cx,
            skeleton.physics_constraints()[1].cy,
        ),
        (-8.0, -4.0, -10.0, -6.0)
    );

    {
        let constraints = skeleton.physics_constraints_mut();
        constraints[0].ux = 0.0;
        constraints[0].uy = 0.0;
        constraints[0].cx = 1.0;
        constraints[0].cy = 0.0;
        constraints[1].ux = 2.0;
        constraints[1].uy = 3.0;
        constraints[1].cx = 1.0;
        constraints[1].cy = 0.0;
    }

    skeleton.physics_rotate(0.0, 0.0, 90.0);
    assert_approx(skeleton.physics_constraints()[0].ux, 1.0);
    assert_approx(skeleton.physics_constraints()[0].uy, -1.0);
    assert_approx(skeleton.physics_constraints()[0].cx, 2.0);
    assert_approx(skeleton.physics_constraints()[0].cy, -1.0);
    assert_approx(skeleton.physics_constraints()[1].ux, 3.0);
    assert_approx(skeleton.physics_constraints()[1].uy, 2.0);
    assert_approx(skeleton.physics_constraints()[1].cx, 2.0);
    assert_approx(skeleton.physics_constraints()[1].cy, -1.0);
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
