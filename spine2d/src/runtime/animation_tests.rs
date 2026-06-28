use crate::runtime::{
    MixBlend, MixDirection, MixFrom, apply_inherit, apply_rotate_mixed, apply_rotate_with,
};
use crate::{
    Animation, AttachmentData, AttachmentFrame, AttachmentTimeline, BlendMode, BoneData,
    BoneTimeline, Curve, Event, EventData, EventTimeline, Inherit, InheritFrame, InheritTimeline,
    RegionAttachmentData, RotateFrame, RotateTimeline, ScaleTimeline, ShearTimeline, Skeleton,
    SkeletonData, SkinData, SlotData, TranslateTimeline, Vec2Frame, apply_animation,
};
use std::sync::Arc;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-6,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

fn animation_with_event_timeline() -> Animation {
    let event_data = std::sync::Arc::new(EventData::new("event"));
    let mut event = Event::new(0.5, event_data);
    event.set_string("tick".to_string());

    crate::runtime::finalize_animation(Animation {
        name: "event".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: Some(EventTimeline::from_events(vec![event])),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    })
}

fn animation_with_looping_event_timeline() -> Animation {
    let event_data = std::sync::Arc::new(EventData::new("event"));
    let mut early = Event::new(0.1, event_data.clone());
    early.set_string("early".to_string());
    let mut late = Event::new(0.75, event_data);
    late.set_string("late".to_string());

    crate::runtime::finalize_animation(Animation {
        name: "loop-event".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: Some(EventTimeline::from_events(vec![early, late])),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    })
}

#[test]
fn animation_apply_collects_events() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data);
    let animation = animation_with_event_timeline();

    let mut events = Vec::new();
    animation.apply(
        &mut skeleton,
        0.0,
        0.75,
        false,
        Some(&mut events),
        1.0,
        MixFrom::Setup,
        false,
        false,
        false,
    );

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].get_string(), "tick");
}

#[test]
fn animation_apply_collects_events_across_loop_wrap() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data);
    let animation = animation_with_looping_event_timeline();

    let mut events = Vec::new();
    animation.apply(
        &mut skeleton,
        0.6,
        0.2,
        true,
        Some(&mut events),
        1.0,
        MixFrom::Setup,
        false,
        false,
        false,
    );

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].get_string(), "late");
    assert_eq!(events[1].get_string(), "early");
}

#[test]
fn animation_apply_uses_previous_time_for_physics_reset() {
    let data = crate::SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "physics": [
    { "name": "physics0", "bone": "root" }
  ],
  "animations": {
    "reset": {
      "physics": {
        "physics0": {
          "reset": [
            { "time": 0.5 },
            { "time": 1.0 }
          ]
        }
      }
    }
  }
}
"#,
    )
    .unwrap();
    let animation = data.find_animation("reset").unwrap().clone();
    let mut skeleton = Skeleton::new(Arc::clone(&data));

    animation.apply(
        &mut skeleton,
        0.75,
        1.25,
        true,
        None,
        1.0,
        MixFrom::Setup,
        false,
        false,
        false,
    );

    assert!(skeleton.physics_constraints[0].reset);
}

#[test]
fn animation_apply_applies_attachment_timelines() {
    let data = skeleton_data_with_attachment_slot();
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    let animation = data.find_animation("attach").unwrap().clone();

    animation.apply(
        &mut skeleton,
        0.0,
        0.75,
        false,
        None,
        1.0,
        MixFrom::Setup,
        false,
        false,
        false,
    );

    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("attachment2"));
}

#[test]
fn animation_apply_writes_applied_pose_bones() {
    let data = skeleton_data_with_applied_pose_bone();
    let mut skeleton = Skeleton::new(Arc::clone(&data));
    skeleton.setup_pose();
    skeleton.bones[0].set_applied_x(6.0);
    skeleton.bones[0].set_applied_y(2.0);

    let animation = data.find_animation("bone").unwrap().clone();
    animation.apply(
        &mut skeleton,
        0.0,
        0.75,
        false,
        None,
        1.0,
        MixFrom::Setup,
        false,
        false,
        true,
    );

    assert_eq!(skeleton.get_bones()[0].get_x(), 1.0);
    assert_eq!(skeleton.get_bones()[0].get_y(), 2.0);
    assert_eq!(skeleton.get_bones()[0].get_applied_x(), 6.0);
    assert_eq!(skeleton.get_bones()[0].get_applied_y(), 2.0);
}

#[test]
fn animation_apply_writes_applied_draw_order() {
    let data = skeleton_data_with_applied_pose_draw_order();
    let mut skeleton = Skeleton::new(Arc::clone(&data));
    skeleton.setup_pose();

    let animation = data.find_animation("draw").unwrap().clone();
    animation.apply(
        &mut skeleton,
        0.0,
        0.75,
        false,
        None,
        1.0,
        MixFrom::Setup,
        false,
        false,
        true,
    );

    assert_eq!(skeleton.get_draw_order(), &[1, 0]);
    assert_eq!(skeleton.get_draw_order_pose(), &[0, 1]);
}

#[test]
fn animation_apply_writes_applied_pose_for_slider_rebuilds() {
    let data = skeleton_data_with_applied_pose_slider();
    let mut skeleton = Skeleton::new(Arc::clone(&data));

    animation_apply_to_applied_pose(&mut skeleton, data.find_animation("slider").unwrap());

    assert_eq!(skeleton.get_bones()[0].get_x(), 1.0);
    assert_eq!(skeleton.get_bones()[0].get_applied_x(), 6.0);
    assert_eq!(skeleton.get_draw_order(), &[1, 0]);
    assert_eq!(skeleton.get_draw_order_pose(), &[1, 0]);
}

#[test]
fn animation_apply_updates_slider_and_draw_order_in_out_mix() {
    let data = slider_draw_order_skeleton_data();
    let animation = data.find_animation("anim").unwrap().clone();
    let mut skeleton = Skeleton::new(Arc::clone(&data));
    skeleton.setup_pose();
    skeleton.get_draw_order_mut().swap(0, 1);

    animation.apply(
        &mut skeleton,
        0.0,
        0.5,
        false,
        None,
        1.0,
        MixFrom::Current,
        false,
        true,
        true,
    );

    assert_approx(skeleton.slider_constraints[0].get_time(), 3.0);
    assert_approx(skeleton.slider_constraints[0].get_mix(), 0.9);
    assert_eq!(skeleton.get_draw_order_pose(), &[1, 0]);
    assert_eq!(skeleton.get_draw_order(), &[0, 1]);
}

fn skeleton_data_with_root(setup_x: f32, setup_y: f32, setup_rotation: f32) -> Arc<SkeletonData> {
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
            x: setup_x,
            y: setup_y,
            rotation: setup_rotation,
            scale_x: 2.0,
            scale_y: 4.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::Normal,
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
    })
}

fn skeleton_data_with_attachment_slot() -> Arc<SkeletonData> {
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
        inherit: Inherit::Normal,
        skin_required: false,
        ..Default::default()
    }];

    let slots = vec![SlotData {
        name: "slot".to_string(),
        bone: 0,
        attachment: Some("attachment1".to_string()),
        setup_pose: Default::default(),
        blend: BlendMode::Normal,
        ..Default::default()
    }];

    let mut attachments = vec![indexmap::IndexMap::new()];
    for name in ["attachment1", "attachment2"] {
        attachments[0].insert(
            name.to_string(),
            AttachmentData::Region(RegionAttachmentData {
                name: name.to_string(),
                path: format!("{name}.png"),
                sequence: None,
                color: [1.0, 1.0, 1.0, 1.0],
                timeline_attachment: name.to_string(),
                timeline_slots: Vec::new(),
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

    let skins = vec![SkinData {
        name: "default".to_string(),
        color: SkinData::DEFAULT_COLOR,
        attachments,
        bones: Vec::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    }];

    let animation = crate::runtime::finalize_animation(Animation {
        name: "attach".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
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
        bones,
        slots,
        skins,
        default_skin: Some(0),
        events: Vec::new(),
        animations: vec![animation],
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    })
}

fn slider_draw_order_skeleton_data() -> Arc<SkeletonData> {
    SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "a" },
    { "name": "slot1", "bone": "root", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "slot0": { "a": { "type": "region", "path": "a.png", "width": 10, "height": 10 } },
      "slot1": { "b": { "type": "region", "path": "b.png", "width": 10, "height": 10 } }
    }
  },
  "slider": [
    {
      "name": "slider",
      "time": 1.0,
      "mix": 0.5,
      "animation": "anim"
    }
  ],
  "animations": {
    "idle": {},
    "anim": {
      "drawOrder": [
        { "time": 0.0, "offsets": [ { "slot": "slot1", "offset": -1 } ] }
      ],
      "slider": {
        "slider": {
          "time": [
            { "time": 0.0, "value": 3.0 },
            { "time": 1.0, "value": 3.0 }
          ],
          "mix": [
            { "time": 0.0, "value": 0.9 },
            { "time": 1.0, "value": 0.9 }
          ]
        }
      }
    }
  }
}
"#,
    )
    .unwrap()
}

fn skeleton_data_with_applied_pose_slider() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(Animation {
        name: "slider".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 5.0,
                y: 0.0,
                curve: [Curve::Linear; 2],
            }],
        })],
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
        draw_order_timeline: Some(crate::DrawOrderTimeline {
            frames: vec![crate::DrawOrderFrame {
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
            x: 1.0,
            y: 2.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::Normal,
            skin_required: false,
            ..Default::default()
        }],
        slots: vec![
            SlotData {
                name: "slot".to_string(),
                bone: 0,
                attachment: None,
                setup_pose: Default::default(),
                blend: BlendMode::Normal,
                ..Default::default()
            },
            SlotData {
                name: "slot1".to_string(),
                bone: 0,
                attachment: None,
                setup_pose: Default::default(),
                blend: BlendMode::Normal,
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
        physics_constraints: vec![crate::PhysicsConstraintData {
            name: "physics".to_string(),
            order: 0,
            skin_required: false,
            bone: 0,
            x: 0.0,
            y: 0.0,
            rotate: 0.0,
            scale_x: 0.0,
            inertia: 0.0,
            strength: 0.0,
            damping: 0.0,
            mass_inverse: 1.0,
            wind: 0.0,
            gravity: 0.0,
            mix: 0.0,
            scale_y_mode: crate::ScaleYMode::None,
            shear_x: 0.0,
            limit: 0.0,
            step: 0.0,
            inertia_global: false,
            strength_global: false,
            damping_global: false,
            mass_global: false,
            wind_global: false,
            gravity_global: false,
            mix_global: false,
        }],
        slider_constraints: vec![crate::SliderConstraintData {
            name: "slider".to_string(),
            order: 1,
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
            animation: Some(0),
            animation_name: Some("slider".to_string()),
        }],
    })
}

fn skeleton_data_with_applied_pose_bone() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(Animation {
        name: "bone".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 5.0,
                y: 0.0,
                curve: [Curve::Linear; 2],
            }],
        })],
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
        draw_order_timeline: None,
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
            x: 1.0,
            y: 2.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::Normal,
            skin_required: false,
            ..Default::default()
        }],
        slots: vec![SlotData {
            name: "slot".to_string(),
            bone: 0,
            attachment: None,
            setup_pose: Default::default(),
            blend: BlendMode::Normal,
            ..Default::default()
        }],
        skins: Vec::new(),
        default_skin: None,
        events: Vec::new(),
        animations: vec![animation],
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    })
}

fn skeleton_data_with_applied_pose_draw_order() -> Arc<SkeletonData> {
    let animation = crate::runtime::finalize_animation(Animation {
        name: "draw".to_string(),
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
        draw_order_timeline: Some(crate::DrawOrderTimeline {
            frames: vec![crate::DrawOrderFrame {
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
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::Normal,
            skin_required: false,
            ..Default::default()
        }],
        slots: vec![
            SlotData {
                name: "slot0".to_string(),
                bone: 0,
                attachment: None,
                setup_pose: Default::default(),
                blend: BlendMode::Normal,
                ..Default::default()
            },
            SlotData {
                name: "slot1".to_string(),
                bone: 0,
                attachment: None,
                setup_pose: Default::default(),
                blend: BlendMode::Normal,
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
        slider_constraints: Vec::new(),
    })
}

fn animation_apply_to_applied_pose(skeleton: &mut Skeleton, animation: &Animation) {
    skeleton.setup_pose();
    skeleton.bones[0].set_x(1.0);
    skeleton.bones[0].set_applied_x(6.0);
    skeleton.get_draw_order_mut().swap(0, 1);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    animation.apply(
        skeleton,
        0.0,
        0.75,
        false,
        None,
        1.0,
        MixFrom::Setup,
        false,
        false,
        true,
    );
}

#[test]
fn translate_timeline_interpolates() {
    let data = skeleton_data_with_root(2.0, 3.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 7.0);
    assert_approx(skeleton.bones[0].y, 3.0);
}

#[test]
fn rotate_timeline_interpolates_linearly() {
    let data = skeleton_data_with_root(0.0, 0.0, 20.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Rotate(RotateTimeline {
            bone_index: 0,
            frames: vec![
                RotateFrame {
                    time: 0.0,
                    angle: 170.0,
                    curve: Curve::Linear,
                },
                RotateFrame {
                    time: 1.0,
                    angle: -170.0,
                    curve: Curve::Linear,
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].rotation, 20.0);
}

#[test]
fn rotate_timeline_mixes_shortest_path() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.bones[0].rotation = 170.0;

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Rotate(RotateTimeline {
            bone_index: 0,
            frames: vec![RotateFrame {
                time: 0.0,
                angle: -170.0,
                curve: Curve::Linear,
            }],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.0,
        false,
        0.5,
        MixBlend::Replace,
    );

    // Upstream RotateTimeline does not normalize the final rotation, it uses CurveTimeline1
    // relative blending (see spine-cpp `RotateTimeline::apply`).
    assert_approx(skeleton.bones[0].rotation, 0.0);
}

#[test]
fn rotate_timeline_before_first_frame_distinguishes_current_and_first() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.bones[0].rotation = 20.0;

    let timeline = RotateTimeline {
        bone_index: 0,
        frames: vec![RotateFrame {
            time: 0.5,
            angle: 30.0,
            curve: Curve::Linear,
        }],
    };
    let mut state = [0.0f32; 2];

    apply_rotate_with(&timeline, &mut skeleton, 0.0, 0.5, MixFrom::Current, false);
    assert_approx(skeleton.bones[0].rotation, 20.0);

    skeleton.bones[0].rotation = 20.0;
    apply_rotate_with(&timeline, &mut skeleton, 0.0, 0.5, MixFrom::First, false);
    assert_approx(skeleton.bones[0].rotation, 10.0);

    skeleton.bones[0].rotation = 20.0;
    apply_rotate_mixed(
        &timeline,
        &mut skeleton,
        0.0,
        0.5,
        MixFrom::First,
        &mut state,
        0,
        true,
    );
    assert_approx(skeleton.bones[0].rotation, 10.0);
}

#[test]
fn inherit_timeline_applies_to_the_keyed_bone() {
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
                x: 0.0,
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

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "inherit".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Inherit(InheritTimeline {
            bone_index: 1,
            frames: vec![InheritFrame {
                time: 0.0,
                inherit: Inherit::NoScaleOrReflection,
            }],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    apply_animation(
        &animation,
        &mut skeleton,
        0.0,
        false,
        1.0,
        MixBlend::Replace,
    );

    assert_eq!(skeleton.bones[0].inherit, Inherit::Normal);
    assert_eq!(skeleton.bones[1].inherit, Inherit::NoScaleOrReflection);
}

#[test]
fn inherit_timeline_out_restores_setup_for_first_and_setup_blends() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.bones[0].inherit = Inherit::OnlyTranslation;

    let timeline = InheritTimeline {
        bone_index: 0,
        frames: vec![InheritFrame {
            time: 0.0,
            inherit: Inherit::NoScaleOrReflection,
        }],
    };

    apply_inherit(
        &timeline,
        &mut skeleton,
        0.5,
        MixBlend::First,
        MixDirection::Out,
    );
    assert_eq!(skeleton.bones[0].inherit, Inherit::Normal);

    skeleton.bones[0].inherit = Inherit::OnlyTranslation;
    apply_inherit(
        &timeline,
        &mut skeleton,
        0.5,
        MixBlend::Setup,
        MixDirection::Out,
    );
    assert_eq!(skeleton.bones[0].inherit, Inherit::Normal);
}

#[test]
fn inherit_timeline_mix_out_restores_setup_for_first_and_setup_modes() {
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
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 2.0,
            scale_y: 4.0,
            shear_x: 0.0,
            shear_y: 0.0,
            inherit: Inherit::NoScaleOrReflection,
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

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "inherit".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Inherit(InheritTimeline {
            bone_index: 0,
            frames: vec![InheritFrame {
                time: 0.0,
                inherit: Inherit::OnlyTranslation,
            }],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();

    apply_animation(&animation, &mut skeleton, 0.0, false, 1.0, MixBlend::First);
    assert_eq!(skeleton.bones[0].inherit, Inherit::OnlyTranslation);

    apply_animation(
        &animation,
        &mut skeleton,
        0.0,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_eq!(skeleton.bones[0].inherit, Inherit::OnlyTranslation);

    apply_animation(&animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Setup);
    assert_eq!(skeleton.bones[0].inherit, Inherit::OnlyTranslation);
}

#[test]
fn looping_wraps_time_by_duration() {
    let data = skeleton_data_with_root(1.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        1.25,
        true,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 3.5);
}

#[test]
fn setup_blend_uses_setup_as_base() {
    let data = skeleton_data_with_root(2.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.bones[0].x = 100.0;

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 10.0,
                y: 0.0,
                curve: [Curve::Linear; 2],
            }],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(&animation, &mut skeleton, 0.0, false, 0.5, MixBlend::Setup);
    assert_approx(skeleton.bones[0].x, 7.0);
}

#[test]
fn scale_timeline_applies() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Scale(ScaleTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 1.0,
                    y: 1.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 3.0,
                    y: 5.0,
                    curve: [Curve::Linear; 2],
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.25,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].scale_x, 3.0);
    assert_approx(skeleton.bones[0].scale_y, 8.0);
}

#[test]
fn stepped_curve_holds_previous_value() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Stepped; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 0.0);
}

#[test]
fn shear_timeline_applies_to_bone() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Shear(ShearTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 20.0,
                    curve: [Curve::Linear; 2],
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].shear_x, 5.0);
    assert_approx(skeleton.bones[0].shear_y, 10.0);
}

#[test]
fn bezier_curve_interpolates_in_value_space() {
    let data = skeleton_data_with_root(0.0, 0.0, 0.0);
    let mut skeleton = Skeleton::new(data.clone());

    // When x control points are at 1/3 and 2/3 of the segment, x(t) becomes linear.
    // With value1=0, value2=10 and cy1=cy2=0, then y(t)=10*t^3, so y(0.5)=1.25.
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 1.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![
                Vec2Frame {
                    time: 0.0,
                    x: 0.0,
                    y: 0.0,
                    curve: [
                        Curve::Bezier {
                            cx1: 1.0 / 3.0,
                            cy1: 0.0,
                            cx2: 2.0 / 3.0,
                            cy2: 0.0,
                        },
                        Curve::Linear,
                    ],
                },
                Vec2Frame {
                    time: 1.0,
                    x: 10.0,
                    y: 0.0,
                    curve: [Curve::Linear; 2],
                },
            ],
        })],
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    apply_animation(
        &animation,
        &mut skeleton,
        0.5,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_approx(skeleton.bones[0].x, 1.25);
}

#[test]
fn finalize_animation_fills_missing_timeline_order_from_shared_helper() {
    let animation = crate::Animation {
        name: "order".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Rotate(RotateTimeline {
            bone_index: 0,
            frames: vec![RotateFrame {
                time: 0.0,
                angle: 0.0,
                curve: Curve::Linear,
            }],
        })],
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![crate::AttachmentTimeline {
            slot_index: 0,
            frames: vec![crate::AttachmentFrame {
                time: 0.0,
                name: None,
            }],
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
        draw_order_timeline: Some(crate::DrawOrderTimeline { frames: Vec::new() }),
        draw_order_folder_timelines: vec![crate::DrawOrderFolderTimeline {
            slots: vec![0],
            frames: Vec::new(),
        }],
        timeline_order: Vec::new(),
    };

    let finalized = crate::runtime::finalize_animation(animation);
    assert_eq!(
        finalized.timeline_order,
        vec![
            crate::TimelineKind::SlotAttachment(0),
            crate::TimelineKind::Bone(0),
            crate::TimelineKind::DrawOrder,
            crate::TimelineKind::DrawOrderFolder(0),
        ]
    );
}

#[test]
fn finalize_animation_preserves_parse_time_timeline_order() {
    let animation = crate::Animation {
        name: "order".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Rotate(RotateTimeline {
            bone_index: 0,
            frames: vec![RotateFrame {
                time: 0.0,
                angle: 0.0,
                curve: Curve::Linear,
            }],
        })],
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![crate::AttachmentTimeline {
            slot_index: 0,
            frames: vec![crate::AttachmentFrame {
                time: 0.0,
                name: None,
            }],
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
        timeline_order: vec![
            crate::TimelineKind::Bone(0),
            crate::TimelineKind::SlotAttachment(0),
        ],
    };

    let finalized = crate::runtime::finalize_animation(animation);
    assert_eq!(
        finalized.timeline_order,
        vec![
            crate::TimelineKind::Bone(0),
            crate::TimelineKind::SlotAttachment(0),
        ]
    );
}
