use crate::{
    AttachmentData, AttachmentFrame, AttachmentTimeline, BlendMode, Bone, BoneData,
    ClippingAttachmentData, ColorFrame, ColorTimeline, ConstraintRef, ConstraintRefMut, Curve,
    DrawOrderFrame, DrawOrderTimeline, IkConstraint, IkConstraintData, Inherit, MeshAttachmentData,
    MeshVertices, PathConstraint, PathConstraintData, PhysicsConstraint, PhysicsConstraintData,
    PositionMode, RegionAttachmentData, RotateMode, ScaleYMode, Skeleton, SkeletonData, SkinData,
    SliderConstraint, SliderConstraintData, SlotData, SpacingMode, TransformConstraint,
    TransformConstraintData, UpdateCacheItem,
};
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
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
        reference_scale: 100.0,
        bones: Vec::new(),
        slots: Vec::new(),
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
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
        timeline_attachment: name.to_string(),
        timeline_slots: Vec::new(),
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        width: 1.0,
        height: 1.0,
    })
}

fn mesh_attachment(
    name: &str,
    timeline_skin: &str,
    timeline_attachment: &str,
    vertex_id: u32,
) -> AttachmentData {
    AttachmentData::Mesh(MeshAttachmentData {
        vertex_id,
        name: name.to_string(),
        path: name.to_string(),
        timeline_skin: timeline_skin.to_string(),
        timeline_attachment: timeline_attachment.to_string(),
        timeline_slots: vec![0],
        sequence: None,
        color: [1.0, 1.0, 1.0, 1.0],
        vertices: MeshVertices::Unweighted(vec![
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, 1.0],
            [-1.0, 1.0],
        ]),
        uvs: vec![[0.0, 0.0]; 4],
        triangles: vec![0, 1, 2, 2, 3, 0],
        hull_length: 0,
        edges: Vec::new(),
        width: 0.0,
        height: 0.0,
    })
}

fn named_attachment_skeleton_data() -> Arc<SkeletonData> {
    let mut default_skin = SkinData::new("default");
    default_skin.attachments.resize_with(2, Default::default);
    default_skin.attachments[0].insert(
        "shared".to_string(),
        region_attachment("default-shared", [1.0, 0.0, 0.0, 1.0]),
    );
    default_skin.attachments[1].insert(
        "fallback".to_string(),
        region_attachment("default-fallback", [0.0, 1.0, 0.0, 1.0]),
    );

    let mut custom_skin = SkinData::new("custom");
    custom_skin.attachments.resize_with(2, Default::default);
    custom_skin.attachments[0].insert(
        "shared".to_string(),
        region_attachment("custom-shared", [0.0, 0.0, 1.0, 1.0]),
    );

    let skins = vec![default_skin, custom_skin];

    Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        default_skin: Some(0),
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    })
}

fn linked_mesh_attachment_skeleton_data() -> Arc<SkeletonData> {
    let mut default_skin = SkinData::new("default");
    default_skin.attachments.resize_with(1, Default::default);
    default_skin.attachments[0].insert(
        "parent".to_string(),
        mesh_attachment("parent", "default", "parent", 1),
    );
    default_skin.attachments[0].insert(
        "child".to_string(),
        mesh_attachment("child", "default", "parent", 2),
    );
    default_skin.attachments[0].insert(
        "other".to_string(),
        mesh_attachment("other", "default", "other", 3),
    );

    let skins = vec![default_skin];

    Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
            attachment: Some("parent".to_string()),
            ..Default::default()
        }],
        skins,
        default_skin: Some(0),
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    })
}

fn skin_switch_linked_mesh_skeleton_data() -> Arc<SkeletonData> {
    let mut default_skin = SkinData::new("default");
    default_skin.attachments.resize_with(1, Default::default);
    default_skin.attachments[0].insert(
        "mesh".to_string(),
        mesh_attachment("default-mesh", "source", "parent", 0),
    );

    let mut old_skin = SkinData::new("old");
    old_skin.attachments.resize_with(1, Default::default);
    old_skin.attachments[0].insert(
        "mesh".to_string(),
        mesh_attachment("old-mesh", "source", "parent", 1),
    );

    let mut new_skin = SkinData::new("new");
    new_skin.attachments.resize_with(1, Default::default);
    new_skin.attachments[0].insert(
        "mesh".to_string(),
        mesh_attachment("new-mesh", "source", "parent", 2),
    );

    let skins = vec![default_skin, old_skin, new_skin];

    Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
            attachment: Some("mesh".to_string()),
            ..Default::default()
        }],
        skins,
        default_skin: Some(0),
        events: Vec::new(),
        animations: Vec::new(),
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
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
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
            property_offset: 0.0,
            offset: 0.0,
            max: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
            animation_name: None,
        }],
    })
}

fn slider_draw_order_skeleton_data() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "slider-draw".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
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
    Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: vec![animation],
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
            property_offset: 0.0,
            offset: 0.0,
            max: 0.0,
            scale: 1.0,
            local: false,
            animation: Some(0),
            animation_name: Some("slider-animation".to_string()),
        }],
    })
}

fn slider_slot_pose_skeleton_data() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "slider-slot-pose".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
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
    let mut default_skin = SkinData::new("default");
    default_skin.attachments.resize_with(1, Default::default);
    default_skin.attachments[0].insert(
        "base".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "base".to_string(),
            path: "base".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            timeline_attachment: "base".to_string(),
            timeline_slots: Vec::new(),
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
            timeline_attachment: "alt".to_string(),
            timeline_slots: Vec::new(),
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 2.0,
            height: 2.0,
        }),
    );

    let skins = vec![default_skin];

    Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
            setup_pose: crate::SlotSetupPose {
                color: [0.1, 0.2, 0.3, 0.4],
                has_dark: false,
                dark_color: [0.0, 0.0, 0.0],
                sequence_index: 0,
            },
            ..Default::default()
        }],
        skins,
        default_skin: Some(0),
        events: Vec::new(),
        animations: vec![animation],
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
            property_offset: 0.0,
            offset: 0.0,
            max: 0.0,
            scale: 1.0,
            local: false,
            animation: Some(0),
            animation_name: Some("slider-animation".to_string()),
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
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
                index: 0,
                name: "slot0".to_string(),
                bone: 0,
                attachment: None,
                setup_pose: crate::SlotSetupPose {
                    color: [0.1, 0.2, 0.3, 0.4],
                    has_dark: true,
                    dark_color: [0.5, 0.6, 0.7],
                    sequence_index: 0,
                },
                blend: BlendMode::Additive,
                visible: true,
            },
            SlotData {
                index: 1,
                name: "slot1".to_string(),
                bone: 0,
                attachment: None,
                ..Default::default()
            },
        ],
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
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
            property_offset: 0.0,
            offset: 0.0,
            max: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
            animation_name: None,
        }],
    })
}

fn clipping_bounds_skeleton_data() -> Arc<SkeletonData> {
    let mut default_skin = SkinData::new("default");
    default_skin.attachments.resize_with(3, Default::default);
    default_skin.attachments[0].insert(
        "clip".to_string(),
        AttachmentData::Clipping(ClippingAttachmentData {
            vertex_id: 1,
            name: "clip".to_string(),
            color: ClippingAttachmentData::DEFAULT_COLOR,
            timeline_attachment: "clip".to_string(),
            timeline_slots: Vec::new(),
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
            timeline_attachment: "region0".to_string(),
            timeline_slots: Vec::new(),
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
            timeline_attachment: "region1".to_string(),
            timeline_slots: Vec::new(),
            x: 4.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 2.0,
            height: 2.0,
        }),
    );

    let skins = vec![default_skin];

    Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        default_skin: Some(0),
        events: Vec::new(),
        animations: Vec::new(),
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

    assert_eq!((skeleton.get_wind_x(), skeleton.get_wind_y()), (1.0, 0.0));
    assert_eq!(skeleton.get_wind_x(), 1.0);
    assert_eq!(skeleton.get_wind_y(), 0.0);
    assert_eq!(
        (skeleton.get_gravity_x(), skeleton.get_gravity_y()),
        (0.0, 1.0)
    );
    assert_eq!(skeleton.get_gravity_x(), 0.0);
    assert_eq!(skeleton.get_gravity_y(), 1.0);
    assert_eq!(skeleton.get_time(), 0.0);

    skeleton.set_wind_x(2.0);
    skeleton.set_wind_y(3.0);
    skeleton.set_gravity_x(4.0);
    skeleton.set_gravity_y(5.0);
    skeleton.set_time(1.5);

    assert_eq!((skeleton.get_wind_x(), skeleton.get_wind_y()), (2.0, 3.0));
    assert_eq!(skeleton.get_wind_x(), 2.0);
    assert_eq!(skeleton.get_wind_y(), 3.0);
    assert_eq!(
        (skeleton.get_gravity_x(), skeleton.get_gravity_y()),
        (4.0, 5.0)
    );
    assert_eq!(skeleton.get_gravity_x(), 4.0);
    assert_eq!(skeleton.get_gravity_y(), 5.0);
    assert_eq!(skeleton.get_time(), 1.5);

    skeleton.set_wind_x(-2.0);
    skeleton.set_wind_y(6.0);
    skeleton.set_gravity_x(7.0);
    skeleton.set_gravity_y(-5.0);
    skeleton.set_time(0.5);
    skeleton.update(-1.0);

    assert_eq!((skeleton.get_wind_x(), skeleton.get_wind_y()), (-2.0, 6.0));
    assert_eq!(
        (skeleton.get_gravity_x(), skeleton.get_gravity_y()),
        (7.0, -5.0)
    );
    assert_eq!(skeleton.get_time(), -0.5);

    skeleton.update(0.25);
    assert_eq!(skeleton.get_time(), -0.25);
}

#[test]
fn skeleton_setup_pose_methods_match_cpp_split() {
    let mut skeleton = Skeleton::new(setup_pose_split_skeleton_data());

    skeleton.get_bones_mut()[0].set_position(10.0, 20.0);
    skeleton.get_bones_mut()[0].set_rotation(30.0);
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_mix(0.25);
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_bend_direction(1);
    skeleton
        .find_constraint::<TransformConstraint>("transform")
        .unwrap()
        .set_mix_x(0.35);
    skeleton
        .find_constraint::<PathConstraint>("path")
        .unwrap()
        .set_position(0.45);
    skeleton.get_physics_constraints_mut()[0].set_gravity(0.55);
    skeleton
        .find_constraint::<SliderConstraint>("slider")
        .unwrap()
        .set_time(0.65);
    *skeleton.get_slots_mut()[0].get_color_mut() = [0.9, 0.8, 0.7, 0.6];
    {
        let slot = &mut skeleton.get_slots_mut()[0];
        slot.attachment = Some("manual".to_string());
        slot.attachment_skin = None;
        slot.sequence_index = 7;
        slot.deform.extend_from_slice(&[1.0, 2.0]);
    }
    skeleton.get_draw_order_mut().swap(0, 1);

    skeleton.setup_pose_bones();

    assert_eq!(
        (
            skeleton.get_bones()[0].get_x(),
            skeleton.get_bones()[0].get_y()
        ),
        (1.0, 2.0)
    );
    assert_eq!(skeleton.get_bones()[0].get_rotation(), 3.0);
    assert_eq!(
        (
            skeleton.get_bones()[0].get_applied_x(),
            skeleton.get_bones()[0].get_applied_y()
        ),
        (1.0, 2.0)
    );
    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .get_mix(),
        0.8
    );
    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .get_bend_direction(),
        -1
    );
    assert_eq!(
        skeleton
            .find_constraint::<TransformConstraint>("transform")
            .unwrap()
            .get_mix_x(),
        0.12
    );
    assert_eq!(
        skeleton
            .find_constraint::<PathConstraint>("path")
            .unwrap()
            .get_position(),
        1.1
    );
    assert_eq!(skeleton.get_physics_constraints()[0].get_gravity(), 0.6);
    assert_eq!(
        skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap()
            .get_time(),
        2.1
    );
    assert_eq!(skeleton.get_slots()[0].get_color(), [0.9, 0.8, 0.7, 0.6]);
    assert_eq!(
        skeleton.get_slots()[0].get_attachment_name(),
        Some("manual")
    );
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), 7);
    assert_eq!(skeleton.get_draw_order(), &[1, 0]);

    skeleton.get_bones_mut()[0].set_position(40.0, 50.0);
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_mix(0.33);

    skeleton.setup_pose_slots();

    assert_eq!(
        (
            skeleton.get_bones()[0].get_x(),
            skeleton.get_bones()[0].get_y()
        ),
        (40.0, 50.0)
    );
    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .get_mix(),
        0.33
    );
    assert_eq!(skeleton.get_slots()[0].get_color(), [0.1, 0.2, 0.3, 0.4]);
    assert!(skeleton.get_slots()[0].has_dark_color());
    assert_eq!(skeleton.get_slots()[0].get_dark_color(), [0.5, 0.6, 0.7]);
    assert_eq!(skeleton.get_data().slots[0].blend, BlendMode::Additive);
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), None);
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    assert!(skeleton.get_slots()[0].get_deform().is_empty());
    assert_eq!(skeleton.get_draw_order(), &[0, 1]);

    skeleton.get_bones_mut()[0].set_position(70.0, 80.0);
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_mix(0.44);
    *skeleton.get_slots_mut()[0].get_color_mut() = [0.2, 0.3, 0.4, 0.5];
    skeleton.get_draw_order_mut().swap(0, 1);

    skeleton.setup_pose();

    assert_eq!(
        (
            skeleton.get_bones()[0].get_x(),
            skeleton.get_bones()[0].get_y()
        ),
        (1.0, 2.0)
    );
    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .get_mix(),
        0.8
    );
    assert_eq!(skeleton.get_slots()[0].get_color(), [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), 0);
    assert_eq!(skeleton.get_draw_order(), &[0, 1]);
    assert_eq!(
        skeleton.get_bones()[0].get_data(&skeleton).get_name(),
        "root"
    );
    assert_eq!(
        skeleton.get_slots()[0].get_data(&skeleton).get_name(),
        "slot0"
    );
}

#[test]
fn slider_draw_order_uses_applied_buffer_without_mutating_pose() {
    let mut skeleton = Skeleton::new(slider_draw_order_skeleton_data());

    assert_eq!(skeleton.get_draw_order_pose(), &[0, 1]);
    assert_eq!(skeleton.get_draw_order(), &[0, 1]);

    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_eq!(skeleton.get_draw_order_pose(), &[0, 1]);
    assert_eq!(skeleton.get_draw_order(), &[1, 0]);

    skeleton
        .find_constraint::<SliderConstraint>("slider")
        .unwrap()
        .set_mix(0.0);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_eq!(skeleton.get_draw_order_pose(), &[0, 1]);
    assert_eq!(skeleton.get_draw_order(), &[0, 1]);
}

#[test]
fn slider_slot_pose_uses_applied_buffer_without_mutating_pose() {
    let mut skeleton = Skeleton::new(slider_slot_pose_skeleton_data());

    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("base"));
    assert_eq!(
        skeleton.get_slots()[0].get_applied_attachment_name(),
        Some("base")
    );
    assert_eq!(skeleton.get_slots()[0].get_color(), [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(
        skeleton.get_slots()[0].get_applied_color(),
        [0.1, 0.2, 0.3, 0.4]
    );

    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("base"));
    assert_eq!(
        skeleton.get_slots()[0].get_applied_attachment_name(),
        Some("alt")
    );
    assert_eq!(skeleton.get_slots()[0].get_color(), [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(
        skeleton.get_slots()[0].get_applied_color(),
        [0.2, 0.3, 0.4, 0.5]
    );
    assert_eq!(
        skeleton.get_slots()[0]
            .get_applied_attachment(&skeleton)
            .map(AttachmentData::get_name),
        Some("alt")
    );

    skeleton
        .find_constraint::<SliderConstraint>("slider")
        .unwrap()
        .set_mix(0.0);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("base"));
    assert_eq!(
        skeleton.get_slots()[0].get_applied_attachment_name(),
        Some("base")
    );
    assert_eq!(
        skeleton.get_slots()[0].get_applied_color(),
        [0.1, 0.2, 0.3, 0.4]
    );
}

#[test]
fn skeleton_accessors_expose_runtime_controls_without_public_vec_fields() {
    let mut skeleton = Skeleton::new(empty_skeleton_data());

    assert_eq!(skeleton.get_data().bones.len(), 0);
    assert_eq!(skeleton.get_bones().len(), 0);
    assert_eq!(skeleton.get_bones_mut().len(), 0);
    assert_eq!(skeleton.get_slots().len(), 0);
    assert_eq!(skeleton.get_slots_mut().len(), 0);
    assert!(skeleton.get_draw_order().is_empty());
    assert!(skeleton.get_draw_order_mut().is_empty());
    assert!(skeleton.get_skin().is_none());
    assert_eq!(skeleton.get_constraints().len(), 0);
    assert_eq!(skeleton.get_constraints_mut().len(), 0);
    assert_eq!(skeleton.get_physics_constraints().len(), 0);
    assert_eq!(skeleton.get_physics_constraints_mut().len(), 0);
    assert!(skeleton.get_update_cache().is_empty());

    assert_eq!(skeleton.get_color(), [1.0, 1.0, 1.0, 1.0]);
    skeleton.set_color([0.25, 0.5, 0.75, 0.875]);
    assert_eq!(skeleton.get_color(), [0.25, 0.5, 0.75, 0.875]);

    assert_eq!((skeleton.get_x(), skeleton.get_y()), (0.0, 0.0));
    skeleton.set_position(10.0, -2.0);
    assert_eq!((skeleton.get_x(), skeleton.get_y()), (10.0, -2.0));
    assert_eq!(skeleton.get_x(), 10.0);
    assert_eq!(skeleton.get_y(), -2.0);

    skeleton.set_x(3.0);
    skeleton.set_y(4.0);
    assert_eq!((skeleton.get_x(), skeleton.get_y()), (3.0, 4.0));

    assert_eq!((skeleton.get_scale_x(), skeleton.get_scale_y()), (1.0, 1.0));
    skeleton.set_scale(2.0, -3.0);
    assert_eq!(
        (skeleton.get_scale_x(), skeleton.get_scale_y()),
        (2.0, -3.0)
    );
    assert_eq!(skeleton.get_scale_x(), 2.0);
    assert_eq!(skeleton.get_scale_y(), -3.0);

    skeleton.set_scale_x(5.0);
    skeleton.set_scale_y(6.0);
    assert_eq!((skeleton.get_scale_x(), skeleton.get_scale_y()), (5.0, 6.0));
}

#[test]
fn skeleton_update_cache_items_expose_read_only_solver_order() {
    let skeleton = Skeleton::new(constraint_lookup_skeleton_data());

    assert_eq!(
        skeleton.get_update_cache(),
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

    assert_eq!(skeleton.get_root_bone().unwrap().data_index, 0);
    assert_eq!(skeleton.get_bones_mut().first_mut().unwrap().data_index, 0);
    assert_eq!(skeleton.find_bone("root").unwrap().data_index, 0);
    assert_eq!(skeleton.find_bone("child").unwrap().data_index, 1);
    assert!(skeleton.find_bone("").is_none());
    assert!(skeleton.find_slot("").is_none());
    assert_eq!(skeleton.find_bone("child").unwrap().parent, Some(0));

    skeleton.get_bones_mut().first_mut().unwrap().set_y(3.0);
    assert_eq!(skeleton.get_bones()[0].get_y(), 3.0);

    skeleton.get_bones_mut()[1].set_x(7.0);
    assert_eq!(skeleton.get_bones()[1].get_x(), 7.0);

    assert_eq!(skeleton.find_slot("slot1").unwrap().data_index, 1);
    assert_eq!(skeleton.find_slot("slot0").unwrap().bone, 0);
    *skeleton.get_slots_mut()[1].get_color_mut() = [0.2, 0.3, 0.4, 0.5];
    assert_eq!(skeleton.get_slots()[1].get_color(), [0.2, 0.3, 0.4, 0.5]);
}

#[test]
fn skeleton_constraint_finders_match_data_names() {
    let mut skeleton = Skeleton::new(constraint_lookup_skeleton_data());

    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .data_index,
        0
    );
    skeleton
        .find_constraint::<IkConstraint>("ik")
        .unwrap()
        .set_mix(0.25);
    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .get_mix(),
        0.25
    );

    assert_eq!(
        skeleton
            .find_constraint::<TransformConstraint>("transform")
            .unwrap()
            .data_index,
        0
    );
    skeleton
        .find_constraint::<TransformConstraint>("transform")
        .unwrap()
        .set_mix_x(0.5);
    assert_eq!(
        skeleton
            .find_constraint::<TransformConstraint>("transform")
            .unwrap()
            .get_mix_x(),
        0.5
    );

    assert_eq!(
        skeleton
            .find_constraint::<PathConstraint>("path")
            .unwrap()
            .data_index,
        0
    );
    skeleton
        .find_constraint::<PathConstraint>("path")
        .unwrap()
        .set_position(2.0);
    assert_eq!(
        skeleton
            .find_constraint::<PathConstraint>("path")
            .unwrap()
            .get_position(),
        2.0
    );

    assert_eq!(
        skeleton
            .find_constraint::<PhysicsConstraint>("physics")
            .unwrap()
            .data_index,
        0
    );
    skeleton.get_physics_constraints_mut()[0].set_mix(0.75);
    assert_eq!(skeleton.get_physics_constraints()[0].get_mix(), 0.75);

    assert_eq!(
        skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap()
            .data_index,
        0
    );
    skeleton
        .find_constraint::<SliderConstraint>("slider")
        .unwrap()
        .set_time(3.0);
    assert_eq!(
        skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap()
            .get_time(),
        3.0
    );

    assert!(skeleton.find_constraint::<IkConstraint>("").is_none());
    assert!(
        skeleton
            .find_constraint::<TransformConstraint>("missing")
            .is_none()
    );
    assert!(skeleton.find_constraint::<PathConstraint>("").is_none());
    assert!(
        skeleton
            .find_constraint::<PhysicsConstraint>("missing")
            .is_none()
    );
    assert!(skeleton.find_constraint::<SliderConstraint>("").is_none());
}

#[test]
fn skeleton_constraints_expose_ordered_typed_constraint_refs() {
    let skeleton = Skeleton::new(constraint_lookup_skeleton_data());
    let constraints = skeleton.get_constraints();

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
            assert_eq!(ik.data_index, 0);
            assert_eq!(transform.data_index, 0);
            assert_eq!(path.data_index, 0);
            assert_eq!(physics.data_index, 0);
            assert_eq!(slider.data_index, 0);
        }
        other => panic!("unexpected constraint order: {other:?}"),
    }
}

#[test]
fn skeleton_constraints_mut_expose_ordered_typed_constraint_refs() {
    let mut skeleton = Skeleton::new(constraint_lookup_skeleton_data());
    let mut constraints = skeleton.get_constraints_mut();

    assert_eq!(constraints.len(), 5);
    assert!(constraints.iter().all(ConstraintRefMut::is_active));
    let names = constraints
        .iter()
        .map(|constraint| constraint.get_data().get_name().to_string())
        .collect::<Vec<_>>();
    assert_eq!(names, ["ik", "transform", "path", "physics", "slider"]);

    match constraints.as_mut_slice() {
        [
            ConstraintRefMut::Ik(ik, ik_data),
            ConstraintRefMut::Transform(transform, transform_data),
            ConstraintRefMut::Path(path, path_data),
            ConstraintRefMut::Physics(physics, physics_data),
            ConstraintRefMut::Slider(slider, slider_data),
        ] => {
            assert_eq!(ik_data.get_name(), "ik");
            assert_eq!(transform_data.get_name(), "transform");
            assert_eq!(path_data.get_name(), "path");
            assert_eq!(physics_data.get_name(), "physics");
            assert_eq!(slider_data.get_name(), "slider");

            ik.set_mix(0.25);
            transform.set_mix_x(0.5);
            path.set_position(2.0);
            physics.set_mix(0.75);
            slider.set_time(3.0);
        }
        other => panic!("unexpected mutable constraint order: {other:?}"),
    }
    drop(constraints);

    assert_eq!(
        skeleton
            .find_constraint::<IkConstraint>("ik")
            .unwrap()
            .get_mix(),
        0.25
    );
    assert_eq!(
        skeleton
            .find_constraint::<TransformConstraint>("transform")
            .unwrap()
            .get_mix_x(),
        0.5
    );
    assert_eq!(
        skeleton
            .find_constraint::<PathConstraint>("path")
            .unwrap()
            .get_position(),
        2.0
    );
    assert_eq!(skeleton.get_physics_constraints()[0].get_mix(), 0.75);
    assert_eq!(
        skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap()
            .get_time(),
        3.0
    );
}

#[test]
fn slider_invalid_bone_index_does_not_activate_constraint() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: Vec::new(),
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
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
            bone: Some(99),
            property: None,
            property_offset: 0.0,
            offset: 0.0,
            max: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
            animation_name: None,
        }],
    });

    let mut skeleton = Skeleton::new(data);
    assert!(
        !skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap()
            .is_active()
    );
}

#[test]
fn skeleton_attachment_lookup_prefers_current_skin_then_default_skin() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());

    assert_eq!(
        skeleton.get_attachment(0, "shared").unwrap().get_name(),
        "default-shared"
    );
    assert_eq!(
        skeleton.get_attachment(0, "shared").unwrap().get_name(),
        "default-shared"
    );
    assert_eq!(
        skeleton.get_attachment(1, "fallback").unwrap().get_name(),
        "default-fallback"
    );
    assert!(skeleton.get_attachment(0, "").is_none());

    skeleton.set_skin(Some("missing"));
    assert!(skeleton.get_skin().is_none());
    assert_eq!(
        skeleton.get_attachment(0, "shared").unwrap().get_name(),
        "default-shared"
    );

    skeleton.set_skin(Some("custom"));
    assert_eq!(
        skeleton.get_skin().map(|skin| skin.name.as_str()),
        Some("custom")
    );
    assert_eq!(
        skeleton.get_attachment(0, "shared").unwrap().get_name(),
        "custom-shared"
    );
    assert_eq!(
        skeleton.get_attachment(1, "fallback").unwrap().get_name(),
        "default-fallback"
    );

    skeleton.set_skin(Some("custom"));
    let before = (
        skeleton.get_slots()[0].attachment.clone(),
        skeleton.get_slots()[0].attachment_skin.clone(),
        skeleton.get_slots()[0].get_sequence_index(),
    );
    skeleton.set_skin(Some("custom"));
    assert_eq!(
        (
            skeleton.get_slots()[0].attachment.clone(),
            skeleton.get_slots()[0].attachment_skin.clone(),
            skeleton.get_slots()[0].get_sequence_index(),
        ),
        before
    );
}

#[test]
fn skeleton_set_attachment_updates_source_skin_and_pose_state() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());

    skeleton.set_skin(Some("custom"));
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("custom")
    );

    {
        let slot = &mut skeleton.get_slots_mut()[0];
        slot.get_deform_mut().extend_from_slice(&[1.0, 2.0]);
        slot.set_sequence_index(7);
    }

    skeleton.set_skin(None);
    skeleton.set_attachment("slot0", "shared");
    assert_eq!(
        skeleton.get_slots()[0].get_attachment_name(),
        Some("shared")
    );
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("default")
    );
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    assert!(skeleton.get_slots()[0].get_deform().is_empty());

    {
        let slot = &mut skeleton.get_slots_mut()[0];
        slot.get_deform_mut().extend_from_slice(&[3.0]);
        slot.set_sequence_index(9);
    }

    skeleton.set_attachment("slot0", "shared");
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("default")
    );
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), 9);
    assert_eq!(skeleton.get_slots()[0].get_deform(), &[3.0]);

    skeleton.set_attachment("slot0", "");
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), None);
    assert_eq!(skeleton.get_slots()[0].attachment_skin, None);
    assert!(skeleton.get_slots()[0].get_deform().is_empty());
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    skeleton.set_attachment("missing", "shared");
    skeleton.set_attachment("", "shared");
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), None);
    assert_eq!(skeleton.get_slots()[0].attachment_skin, None);
}

#[test]
fn skeleton_default_skin_uses_explicit_index_not_magic_name() {
    let mut fallback_skin = SkinData::new("fallback");
    fallback_skin.set_attachment(
        0,
        "shared",
        region_attachment("fallback-shared", [0.8, 0.2, 0.1, 1.0]),
    );

    let mut data = SkeletonData::default();
    data.bones.push(BoneData {
        name: "root".to_string(),
        parent: None,
        length: 0.0,
        skin_required: false,
        ..Default::default()
    });
    data.slots.push(SlotData {
        name: "slot0".to_string(),
        bone: 0,
        attachment: Some("shared".to_string()),
        ..Default::default()
    });
    data.skins.push(fallback_skin);

    assert!(data.get_default_skin().is_none());
    let fallback = data.find_skin("fallback").cloned();
    data.set_default_skin(fallback.as_ref());
    assert_eq!(
        data.get_default_skin().map(SkinData::get_name),
        Some("fallback")
    );

    let mut skeleton = Skeleton::new(Arc::new(data));
    assert_eq!(
        skeleton
            .get_attachment(0, "shared")
            .map(AttachmentData::get_name),
        Some("fallback-shared")
    );

    skeleton.setup_pose_slots();
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("fallback")
    );
}

#[test]
fn skeleton_set_attachment_preserves_deform_for_matching_timeline_attachment() {
    let mut skeleton = Skeleton::new(linked_mesh_attachment_skeleton_data());

    {
        let slot = &mut skeleton.get_slots_mut()[0];
        slot.get_deform_mut()
            .extend_from_slice(&[1.0, 2.0, 3.0, 4.0]);
        slot.set_sequence_index(7);
    }

    skeleton.set_attachment("slot0", "child");
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("child"));
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    assert_eq!(skeleton.get_slots()[0].get_deform(), &[1.0, 2.0, 3.0, 4.0]);

    skeleton.get_slots_mut()[0].set_sequence_index(9);
    skeleton.set_attachment("slot0", "other");
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("other"));
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    assert!(skeleton.get_slots()[0].get_deform().is_empty());
}

#[test]
fn skeleton_set_skin_preserves_deform_for_matching_timeline_attachment() {
    let mut skeleton = Skeleton::new(skin_switch_linked_mesh_skeleton_data());

    skeleton.set_skin(Some("old"));
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("mesh"));
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("old")
    );

    {
        let slot = &mut skeleton.get_slots_mut()[0];
        slot.get_deform_mut().extend_from_slice(&[5.0, 6.0]);
        slot.set_sequence_index(4);
    }

    skeleton.set_skin(Some("new"));
    assert_eq!(
        skeleton.get_skin().map(|skin| skin.name.as_str()),
        Some("new")
    );
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("new")
    );
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    assert_eq!(skeleton.get_slots()[0].get_deform(), &[5.0, 6.0]);
}

#[test]
fn skeleton_set_skin_from_default_preserves_deform_for_matching_timeline_attachment() {
    let mut skeleton = Skeleton::new(skin_switch_linked_mesh_skeleton_data());

    assert!(skeleton.get_skin().is_none());
    assert_eq!(skeleton.get_slots()[0].get_attachment_name(), Some("mesh"));
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("default")
    );

    {
        let slot = &mut skeleton.get_slots_mut()[0];
        slot.get_deform_mut().extend_from_slice(&[7.0, 8.0]);
        slot.set_sequence_index(6);
    }

    skeleton.set_skin(Some("new"));
    assert_eq!(
        skeleton.get_skin().map(|skin| skin.name.as_str()),
        Some("new")
    );
    assert_eq!(
        skeleton.get_slots()[0].attachment_skin.as_deref(),
        Some("new")
    );
    assert_eq!(skeleton.get_slots()[0].get_sequence_index(), -1);
    assert_eq!(skeleton.get_slots()[0].get_deform(), &[7.0, 8.0]);
}

#[test]
fn skeleton_bounds_cover_region_and_mesh_attachments() {
    let mut default_skin = SkinData::new("default");
    default_skin.attachments.resize_with(2, Default::default);
    default_skin.attachments[0].insert(
        "region".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "region".to_string(),
            path: "region".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            timeline_attachment: "region".to_string(),
            timeline_slots: Vec::new(),
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
            hull_length: 0,
            edges: Vec::new(),
            width: 0.0,
            height: 0.0,
        }),
    );

    let skins = vec![default_skin];

    let data = Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        default_skin: Some(0),
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert_eq!(skeleton.get_bounds(), Some((-1.0, -1.0, 6.0, 7.0)));
}

#[test]
fn skeleton_bounds_with_clipping_respects_clip_polygons_and_end_slots() {
    let mut skeleton = Skeleton::new(clipping_bounds_skeleton_data());
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_eq!(skeleton.get_bounds(), Some((-2.0, -2.0, 7.0, 4.0)));
    assert_eq!(
        skeleton.get_bounds_with_clipping(),
        Some((-1.0, -1.0, 6.0, 2.0))
    );
}

#[test]
fn bone_accessors_expose_local_applied_and_world_pose() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: Vec::new(),
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    let bone = &mut skeleton.get_bones_mut()[0];

    assert!(bone.is_active());
    bone.set_active(false);
    assert!(!bone.is_active());

    bone.set_inherit(Inherit::OnlyTranslation);
    bone.set_position(1.0, 2.0);
    bone.set_rotation(3.0);
    bone.set_scale(4.0);
    bone.set_shear_x(6.0);
    bone.set_shear_y(7.0);
    assert_eq!(bone.get_inherit(), Inherit::OnlyTranslation);
    assert_eq!((bone.get_x(), bone.get_y()), (1.0, 2.0));
    assert_eq!(bone.get_rotation(), 3.0);
    assert_eq!((bone.get_scale_x(), bone.get_scale_y()), (4.0, 4.0));
    bone.set_scale_xy(4.0, 5.0);
    assert_eq!((bone.get_scale_x(), bone.get_scale_y()), (4.0, 5.0));
    assert_eq!(bone.get_shear_x(), 6.0);
    assert_eq!(bone.get_shear_y(), 7.0);

    bone.set_applied_x(8.0);
    bone.set_applied_y(9.0);
    bone.set_applied_rotation(10.0);
    bone.set_applied_scale_x(11.0);
    bone.set_applied_scale_y(12.0);
    bone.set_applied_shear_x(13.0);
    bone.set_applied_shear_y(14.0);
    assert_eq!((bone.get_applied_x(), bone.get_applied_y()), (8.0, 9.0));
    assert_eq!(bone.get_applied_rotation(), 10.0);
    assert_eq!(
        (bone.get_applied_scale_x(), bone.get_applied_scale_y()),
        (11.0, 12.0)
    );
    assert_eq!(
        (bone.get_applied_shear_x(), bone.get_applied_shear_y()),
        (13.0, 14.0)
    );

    bone.set_a(3.0);
    bone.set_b(0.0);
    bone.set_c(4.0);
    bone.set_d(2.0);
    bone.set_world_x(15.0);
    bone.set_world_y(16.0);
    assert_eq!(bone.get_a(), 3.0);
    assert_eq!(bone.get_b(), 0.0);
    assert_eq!(bone.get_c(), 4.0);
    assert_eq!(bone.get_d(), 2.0);
    assert_eq!((bone.get_world_x(), bone.get_world_y()), (15.0, 16.0));
    assert_approx(bone.get_world_scale_x(), 5.0);
    assert_approx(bone.get_world_scale_y(), 2.0);
    assert_approx_angle(bone.get_world_rotation_x(), 4.0f32.atan2(3.0).to_degrees());
    assert_approx(bone.get_world_rotation_y(), 90.0);
    assert_eq!(
        skeleton.get_bones()[0].get_data(&skeleton).get_name(),
        "root"
    );
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
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_scale(2.0, 3.0);
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert!(!Bone::is_y_down());
    assert_eq!((skeleton.get_scale_x(), skeleton.get_scale_y()), (2.0, 3.0));
    assert_eq!(skeleton.get_scale_y(), 3.0);
    assert_approx_pair(
        (
            skeleton.get_bones()[0].get_world_x(),
            skeleton.get_bones()[0].get_world_y(),
        ),
        (2.0, 6.0),
    );
    assert_approx(skeleton.get_bones()[0].get_d(), 3.0);

    Bone::set_y_down(true);

    let mut skeleton = Skeleton::new(data);
    skeleton.set_scale(2.0, 3.0);
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert!(Bone::is_y_down());
    assert_eq!(
        (skeleton.get_scale_x(), skeleton.get_scale_y()),
        (2.0, -3.0)
    );
    assert_eq!(skeleton.get_scale_y(), -3.0);
    assert_approx_pair(
        (
            skeleton.get_bones()[0].get_world_x(),
            skeleton.get_bones()[0].get_world_y(),
        ),
        (2.0, -6.0),
    );
    assert_approx(skeleton.get_bones()[0].get_d(), -3.0);
}

#[test]
fn bone_parent_and_children_expose_skeleton_hierarchy() {
    let skeleton = Skeleton::new(named_attachment_skeleton_data());

    assert!(skeleton.get_bones()[0].get_parent(&skeleton).is_none());
    assert_eq!(
        skeleton.get_bones()[0]
            .get_children(&skeleton)
            .iter()
            .map(|bone| bone.data_index)
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(
        skeleton.get_bones()[1]
            .get_parent(&skeleton)
            .map(|bone| bone.data_index),
        Some(0)
    );
    assert!(skeleton.get_bones()[1].get_children(&skeleton).is_empty());
}

#[test]
fn bone_parent_space_helpers_follow_parent_world_transform() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    {
        let root = &mut skeleton.get_bones_mut()[0];
        root.set_a(2.0);
        root.set_b(0.25);
        root.set_c(0.5);
        root.set_d(3.0);
        root.set_world_x(10.0);
        root.set_world_y(20.0);
    }

    let root = &skeleton.get_bones()[0];
    let child = &skeleton.get_bones()[1];

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
    let bone = &mut skeleton.get_bones_mut()[0];

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

    assert_approx(bone.get_a(), 0.0);
    assert_approx(bone.get_b(), -1.0);
    assert_approx(bone.get_c(), 1.0);
    assert_approx(bone.get_d(), 0.0);
}

#[test]
fn skeleton_bone_world_transform_helper_recomputes_modified_local_pose() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    skeleton.get_bones_mut()[0].set_applied_x(12.0);
    skeleton.get_bones_mut()[0].set_applied_y(34.0);
    skeleton.get_bones_mut()[0].set_applied_rotation(90.0);
    skeleton.modify_bone_local(0);
    skeleton.update_bone_world_transform(0);

    let bone = &skeleton.get_bones()[0];
    assert_approx_pair((bone.get_world_x(), bone.get_world_y()), (12.0, 34.0));
    assert_approx(bone.get_world_rotation_x(), 90.0);
}

#[test]
fn skeleton_bone_local_transform_helper_rebuilds_applied_pose_from_world() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    skeleton.set_position(2.0, 3.0);
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    skeleton.modify_bone_local(0);

    let bone = &mut skeleton.get_bones_mut()[0];
    bone.set_world_x(12.0);
    bone.set_world_y(18.0);
    bone.set_a(0.0);
    bone.set_b(-1.0);
    bone.set_c(1.0);
    bone.set_d(0.0);

    skeleton.update_bone_local_transform(0);

    let bone = &skeleton.get_bones()[0];
    assert_approx_pair((bone.get_applied_x(), bone.get_applied_y()), (10.0, 15.0));
    assert_approx(bone.get_applied_rotation(), 90.0);
    assert_approx_pair(
        (bone.get_applied_scale_x(), bone.get_applied_scale_y()),
        (1.0, 1.0),
    );
    assert_approx_pair(
        (bone.get_applied_shear_x(), bone.get_applied_shear_y()),
        (0.0, 0.0),
    );

    skeleton.get_bones_mut()[0].set_applied_x(99.0);
    skeleton.get_bones_mut()[0].set_applied_y(99.0);
    skeleton.update_bone_world_transform(0);
    assert_approx_pair(
        (
            skeleton.get_bones()[0].get_world_x(),
            skeleton.get_bones()[0].get_world_y(),
        ),
        (12.0, 18.0),
    );
}

#[test]
fn skeleton_validate_bone_local_transform_uses_world_modified_marker() {
    let mut skeleton = Skeleton::new(named_attachment_skeleton_data());
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    skeleton.get_bones_mut()[0].set_world_x(21.0);
    skeleton.get_bones_mut()[0].set_world_y(22.0);
    skeleton.validate_bone_local_transform(0);
    assert_approx_pair(
        (
            skeleton.get_bones()[0].get_applied_x(),
            skeleton.get_bones()[0].get_applied_y(),
        ),
        (0.0, 0.0),
    );

    skeleton.modify_bone_world(0);
    skeleton.validate_bone_local_transform(0);
    assert_approx_pair(
        (
            skeleton.get_bones()[0].get_applied_x(),
            skeleton.get_bones()[0].get_applied_y(),
        ),
        (21.0, 22.0),
    );
}

#[test]
fn slot_accessors_expose_attachment_tint_and_deform_state() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    let slot = &mut skeleton.get_slots_mut()[0];

    assert_eq!(slot.bone, 0);

    *slot.get_color_mut() = [0.1, 0.2, 0.3, 0.4];
    slot.set_has_dark_color(true);
    *slot.get_dark_color_mut() = [0.5, 0.6, 0.7];
    assert_eq!(slot.get_color(), [0.1, 0.2, 0.3, 0.4]);
    assert!(slot.has_dark_color());
    assert_eq!(slot.get_dark_color(), [0.5, 0.6, 0.7]);

    slot.set_sequence_index(4);
    slot.get_deform_mut().extend_from_slice(&[1.0, 2.0, 3.0]);
    assert_eq!(slot.get_sequence_index(), 4);
    assert_eq!(slot.get_deform(), &[1.0, 2.0, 3.0]);
}

#[test]
fn constraint_accessors_expose_pose_state() {
    let skeleton = Skeleton::new(Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
                name: "bone-1".to_string(),
                parent: Some(0),
                ..Default::default()
            },
            BoneData {
                name: "bone-2".to_string(),
                parent: Some(1),
                ..Default::default()
            },
            BoneData {
                name: "bone-3".to_string(),
                parent: Some(2),
                ..Default::default()
            },
        ],
        slots: vec![
            SlotData {
                name: "slot-0".to_string(),
                bone: 0,
                attachment: None,
                ..Default::default()
            },
            SlotData {
                name: "slot-1".to_string(),
                bone: 1,
                attachment: None,
                ..Default::default()
            },
            SlotData {
                name: "slot-2".to_string(),
                bone: 2,
                attachment: None,
                ..Default::default()
            },
        ],
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    }));

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
    assert_eq!(ik.data_index, 1);
    ik.get_bones_mut().push(1);
    ik.set_target(&skeleton.bones[3]);
    ik.set_scale_y_mode(ScaleYMode::Volume);
    ik.set_mix(0.5);
    ik.set_softness(2.0);
    ik.set_compress(true);
    ik.set_stretch(false);
    ik.set_bend_direction(-1);
    ik.set_active(false);
    assert_eq!(ik.get_bones(), &[0, 1]);
    assert_eq!(ik.get_target(&skeleton).data_index, 3);
    assert_eq!(ik.get_scale_y_mode(), ScaleYMode::Volume);
    assert_eq!(ik.get_mix(), 0.5);
    assert_eq!(ik.get_softness(), 2.0);
    assert!(ik.get_compress());
    assert!(!ik.get_stretch());
    assert_eq!(ik.get_bend_direction(), -1);
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
    transform.get_bones_mut().push(2);
    transform.set_source(&skeleton.bones[3]);
    transform.set_mix_rotate(1.1);
    transform.set_mix_x(1.2);
    transform.set_mix_y(1.3);
    transform.set_mix_scale_x(1.4);
    transform.set_mix_scale_y(1.5);
    transform.set_mix_shear_y(1.6);
    transform.set_active(false);
    assert_eq!(transform.data_index, 2);
    assert_eq!(transform.get_bones(), &[1, 2]);
    assert_eq!(transform.get_source(&skeleton).data_index, 3);
    assert_eq!(transform.get_mix_rotate(), 1.1);
    assert_eq!(transform.get_mix_x(), 1.2);
    assert_eq!(transform.get_mix_y(), 1.3);
    assert_eq!(transform.get_mix_scale_x(), 1.4);
    assert_eq!(transform.get_mix_scale_y(), 1.5);
    assert_eq!(transform.get_mix_shear_y(), 1.6);
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
    path.get_bones_mut().push(1);
    path.set_slot(&skeleton.slots[2]);
    path.set_position(7.0);
    path.set_spacing(8.0);
    path.set_mix_rotate(9.0);
    path.set_mix_x(10.0);
    path.set_mix_y(11.0);
    path.set_active(false);
    assert_eq!(path.data_index, 3);
    assert_eq!(path.get_bones(), &[0, 1]);
    assert_eq!(path.get_slot(&skeleton).data_index, 2);
    assert_eq!(path.get_position(), 7.0);
    assert_eq!(path.get_spacing(), 8.0);
    assert_eq!(path.get_mix_rotate(), 9.0);
    assert_eq!(path.get_mix_x(), 10.0);
    assert_eq!(path.get_mix_y(), 11.0);
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
    physics.bone = 2;
    physics.set_inertia(1.1);
    physics.set_strength(1.2);
    physics.set_damping(1.3);
    physics.set_mass_inverse(1.4);
    physics.set_wind(1.5);
    physics.set_gravity(1.6);
    physics.set_mix(1.7);
    physics.set_scale_y_mode(ScaleYMode::Uniform);
    physics.set_active(false);
    assert_eq!(physics.data_index, 4);
    assert_eq!(physics.bone, 2);
    assert_eq!(physics.get_inertia(), 1.1);
    assert_eq!(physics.get_strength(), 1.2);
    assert_eq!(physics.get_damping(), 1.3);
    assert_eq!(physics.get_mass_inverse(), 1.4);
    assert_eq!(physics.get_wind(), 1.5);
    assert_eq!(physics.get_gravity(), 1.6);
    assert_eq!(physics.get_mix(), 1.7);
    assert_eq!(physics.get_scale_y_mode(), ScaleYMode::Uniform);
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
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: Vec::new(),
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
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
            property_offset: 0.0,
            offset: 0.0,
            max: 0.0,
            scale: 1.0,
            local: false,
            animation: None,
            animation_name: None,
        }],
    });
    let mut skeleton = Skeleton::new(data);
    {
        let bone = skeleton.bones[0].clone();
        let slider = skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap();
        slider.set_bone(Some(&bone));
        slider.set_time(2.5);
        slider.set_mix(0.75);
        slider.set_active(false);
        assert_eq!(slider.bone, Some(0));
        assert_eq!(slider.data_index, 0);
        assert_eq!(slider.get_time(), 2.5);
        assert_eq!(slider.get_mix(), 0.75);
        assert!(!slider.is_active());
    }
    assert_eq!(
        skeleton
            .find_constraint::<SliderConstraint>("slider")
            .unwrap()
            .bone,
        Some(0)
    );
    let slider = skeleton
        .get_constraints()
        .into_iter()
        .find_map(|constraint| match constraint {
            ConstraintRef::Slider(slider) => Some(slider),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        slider.get_bone(&skeleton).map(|bone| bone.data_index),
        Some(0)
    );
}

#[test]
fn skeleton_physics_controls_broadcast_to_all_constraints() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
        reference_scale: 100.0,
        bones: vec![BoneData {
            name: "root".to_string(),
            parent: None,
            length: 0.0,
            skin_required: false,
            ..Default::default()
        }],
        slots: Vec::new(),
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
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
        let constraints = skeleton.get_physics_constraints_mut();
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
            skeleton.get_physics_constraints()[0].ux,
            skeleton.get_physics_constraints()[0].uy,
            skeleton.get_physics_constraints()[0].cx,
            skeleton.get_physics_constraints()[0].cy,
        ),
        (7.0, 22.0, 27.0, 42.0)
    );
    assert_eq!(
        (
            skeleton.get_physics_constraints()[1].ux,
            skeleton.get_physics_constraints()[1].uy,
            skeleton.get_physics_constraints()[1].cx,
            skeleton.get_physics_constraints()[1].cy,
        ),
        (-8.0, -4.0, -10.0, -6.0)
    );

    {
        let constraints = skeleton.get_physics_constraints_mut();
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
    assert_approx(skeleton.get_physics_constraints()[0].ux, 1.0);
    assert_approx(skeleton.get_physics_constraints()[0].uy, -1.0);
    assert_approx(skeleton.get_physics_constraints()[0].cx, 2.0);
    assert_approx(skeleton.get_physics_constraints()[0].cy, -1.0);
    assert_approx(skeleton.get_physics_constraints()[1].ux, 3.0);
    assert_approx(skeleton.get_physics_constraints()[1].uy, 2.0);
    assert_approx(skeleton.get_physics_constraints()[1].cx, 2.0);
    assert_approx(skeleton.get_physics_constraints()[1].cy, -1.0);
}

#[test]
fn update_world_transform_root_and_child() {
    let data = Arc::new(SkeletonData {
        spine_version: None,
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

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
        name: String::new(),
        hash: String::new(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        fps: crate::SkeletonData::DEFAULT_FPS,
        images_path: String::new(),
        audio_path: String::new(),
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
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let child = &skeleton.bones[1];
    assert_approx(child.world_x, 0.0);
    assert_approx(child.world_y, 1.0);
}
