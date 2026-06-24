use crate::runtime::{AnimationState, AnimationStateData};
use crate::{
    Animation, AttachmentData, AttachmentFrame, AttachmentTimeline, BlendMode, BoneData,
    RegionAttachmentData, Skeleton, SkeletonData, SkinData, SlotData,
};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

fn build_data() -> Arc<SkeletonData> {
    let bones = vec![BoneData {
        name: "bone".to_string(),
        parent: None,
        length: 0.0,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        shear_x: 0.0,
        shear_y: 0.0,
        inherit: Default::default(),
        skin_required: false,
        ..Default::default()
    }];

    let slots = vec![SlotData {
        name: "slot".to_string(),
        bone: 0,
        attachment: None,
        color: [1.0, 1.0, 1.0, 1.0],
        has_dark: false,
        dark_color: [0.0, 0.0, 0.0],
        blend: BlendMode::Normal,
        ..Default::default()
    }];

    let animation = crate::runtime::finalize_animation(Animation {
        name: "animation".to_string(),
        duration: 1.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: vec![
                AttachmentFrame {
                    time: 0.0,
                    name: Some("attachment1".to_string()),
                },
                AttachmentFrame {
                    time: 0.5,
                    name: Some("attachment2".to_string()),
                },
            ],
        }],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut animation_index = HashMap::new();
    animation_index.insert(animation.name.clone(), 0usize);

    let mut attachments = vec![IndexMap::new()];
    for name in ["attachment1", "attachment2"] {
        attachments[0].insert(
            name.to_string(),
            AttachmentData::Region(RegionAttachmentData {
                name: name.to_string(),
                path: format!("{name}.png"),
                sequence: None,
                color: [1.0, 1.0, 1.0, 1.0],
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                width: 1.0,
                height: 1.0,
            }),
        );
    }
    let mut skins = HashMap::new();
    skins.insert(
        "default".to_string(),
        SkinData {
            name: "default".to_string(),
            attachments,
            bones: Vec::new(),
            ik_constraints: Vec::new(),
            transform_constraints: Vec::new(),
            path_constraints: Vec::new(),
            physics_constraints: Vec::new(),
            slider_constraints: Vec::new(),
        },
    );

    Arc::new(SkeletonData {
        spine_version: None,
        reference_scale: 100.0,
        bones,
        slots,
        skins,
        events: HashMap::new(),
        animations: vec![animation],
        animation_index,
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    })
}

#[test]
fn attachment_timeline_libgdx_upstream_tests() {
    let data = build_data();
    let mut skeleton = Skeleton::new(data.clone());

    let state_data = AnimationStateData::new(data);
    let mut state = AnimationState::new(state_data);

    state.set_animation(0, "animation", true);

    let mut test_step = |delta: f32, expected: &str| {
        state.update(delta);
        state.apply(&mut skeleton);
        assert_eq!(skeleton.slots[0].attachment.as_deref(), Some(expected));
    };

    test_step(0.0, "attachment1");
    test_step(0.0, "attachment1");
    test_step(0.25, "attachment1");
    test_step(0.0, "attachment1");
    test_step(0.25, "attachment2");
    test_step(0.25, "attachment2");
}
