use crate::{
    Animation, BoneData, BoneTimeline, ConstraintDataRef, EventData, IkConstraintData,
    PathConstraintData, PhysicsConstraintData, PositionMode, RotateMode, RotateTimeline,
    ScaleXTimeline, ScaleYMode, SkeletonData, SkinData, SliderConstraintData, SlotData,
    SpacingMode, TransformConstraintData,
};

#[test]
fn skeleton_data_named_lookup_helpers_match_cpp_surface() {
    let mut data = SkeletonData::default();
    data.bones.push(BoneData {
        name: "root".to_string(),
        ..BoneData::default()
    });
    data.bones.push(BoneData {
        name: "child".to_string(),
        parent: Some(0),
        ..BoneData::default()
    });
    data.slots.push(SlotData {
        name: "slot".to_string(),
        bone: 1,
        setup_pose: crate::SlotSetupPose::default(),
        ..SlotData::default()
    });
    data.skins.push(SkinData::new("skin"));
    data.events.push(EventData::with_setup_pose(
        "event", 1, 2.0, "payload", "", 1.0, 0.0,
    ));
    data.animations.push(Animation {
        name: "animation".to_string(),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });
    data.ik_constraints.push(IkConstraintData {
        name: "ik".to_string(),
        order: 2,
        skin_required: false,
        bones: vec![0],
        target: 0,
        scale_y_mode: ScaleYMode::None,
        mix: 1.0,
        softness: 0.0,
        compress: false,
        stretch: false,
        bend_direction: 1,
    });
    data.transform_constraints.push(TransformConstraintData {
        name: "transform".to_string(),
        order: 4,
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
    });
    data.path_constraints.push(PathConstraintData {
        name: "path".to_string(),
        order: 1,
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
    });
    data.physics_constraints.push(PhysicsConstraintData {
        name: "physics".to_string(),
        order: 3,
        skin_required: false,
        bone: 0,
        x: 0.0,
        y: 0.0,
        rotate: 0.0,
        scale_x: 0.0,
        scale_y_mode: ScaleYMode::None,
        shear_x: 0.0,
        limit: 0.0,
        step: 1.0 / 60.0,
        inertia: 1.0,
        strength: 100.0,
        damping: 1.0,
        mass_inverse: 1.0,
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
    });
    data.slider_constraints.push(SliderConstraintData {
        name: "slider".to_string(),
        order: 0,
        skin_required: false,
        setup_time: 0.0,
        setup_mix: 1.0,
        additive: false,
        looped: false,
        bone: Some(0),
        property: None,
        property_from: 0.0,
        to: 0.0,
        scale: 0.0,
        local: false,
        animation: None,
    });

    assert_eq!(data.find_bone("root").unwrap().get_name(), "root");
    assert_eq!(
        data.bones[data.find_bone("child").unwrap().parent.unwrap()].get_name(),
        "root"
    );
    assert_eq!(data.find_slot("slot").unwrap().get_name(), "slot");
    assert_eq!(
        data.bones[data.find_slot("slot").unwrap().bone].get_name(),
        "child"
    );
    assert_eq!(data.find_skin("skin").unwrap().get_name(), "skin");
    assert_eq!(
        data.find_event("event")
            .unwrap()
            .get_setup_pose()
            .get_string(),
        "payload"
    );
    assert_eq!(data.find_animation("animation").unwrap().name, "animation");
    assert_eq!(data.find_ik_constraint("ik").unwrap().target, 0);
    assert_eq!(
        data.find_transform_constraint("transform").unwrap().order,
        4
    );
    assert_eq!(data.find_path_constraint("path").unwrap().order, 1);
    assert_eq!(data.find_physics_constraint("physics").unwrap().order, 3);
    assert_eq!(data.find_slider_constraint("slider").unwrap().order, 0);

    let constraints = data.get_constraints();
    assert_eq!(
        constraints
            .iter()
            .map(|constraint| (constraint.get_order(), constraint.get_name()))
            .collect::<Vec<_>>(),
        vec![
            (0, "slider"),
            (1, "path"),
            (2, "ik"),
            (3, "physics"),
            (4, "transform"),
        ]
    );
    assert!(matches!(
        constraints[0],
        ConstraintDataRef::Slider(data) if data.name == "slider"
    ));

    assert!(data.find_bone("").is_none());
    assert!(data.find_slot("").is_none());
    assert!(data.find_skin("").is_none());
    assert!(data.find_event("").is_none());
    assert!(data.find_animation("").is_none());
    assert!(data.find_ik_constraint("").is_none());
    assert!(data.find_transform_constraint("").is_none());
    assert!(data.find_path_constraint("").is_none());
    assert!(data.find_physics_constraint("").is_none());
    assert!(data.find_slider_constraint("").is_none());
}

#[test]
fn skeleton_data_header_getters_match_cpp_names() {
    let mut data = SkeletonData::default();
    data.name = "hero".to_string();
    data.spine_version = Some("4.3.8".to_string());
    data.hash = "123".to_string();
    data.x = 1.0;
    data.y = 2.0;
    data.width = 3.0;
    data.height = 4.0;
    data.reference_scale = 50.0;
    data.fps = 24.0;
    data.images_path = "images/".to_string();
    data.audio_path = "audio/".to_string();

    assert_eq!(data.get_name(), "hero");
    assert_eq!(data.get_version(), Some("4.3.8"));
    assert_eq!(data.get_hash(), "123");
    assert_eq!(data.get_x(), 1.0);
    assert_eq!(data.get_y(), 2.0);
    assert_eq!(data.get_width(), 3.0);
    assert_eq!(data.get_height(), 4.0);
    assert_eq!(data.get_reference_scale(), 50.0);
    assert_eq!(data.get_fps(), 24.0);
    assert_eq!(data.get_images_path(), "images/");
    assert_eq!(data.get_audio_path(), "audio/");
}

#[test]
fn bone_and_slot_data_accessors_match_cpp_surface() {
    let mut bone = BoneData::default();
    bone.set_length(12.5);
    bone.set_skin_required(true);
    bone.set_icon("root-icon");
    bone.set_icon_size(2.5);
    bone.set_icon_rotation(45.0);
    bone.set_visible(false);

    assert_eq!(bone.get_name(), "");
    assert_eq!(bone.get_length(), 12.5);
    assert!(bone.get_skin_required());
    assert_eq!(bone.get_color(), [0.61, 0.61, 0.61, 1.0]);
    assert_eq!(bone.get_icon(), "root-icon");
    assert_eq!(bone.get_icon_size(), 2.5);
    assert_eq!(bone.get_icon_rotation(), 45.0);
    assert!(!bone.get_visible());

    let mut slot = SlotData::default();
    slot.set_attachment_name("head");
    slot.set_blend_mode(crate::BlendMode::Additive);
    slot.set_visible(false);

    assert_eq!(slot.get_name(), "");
    assert_eq!(slot.get_attachment_name(), "head");
    assert_eq!(slot.get_blend_mode(), crate::BlendMode::Additive);
    assert!(!slot.get_visible());
}

#[test]
fn animation_getters_and_duration_setter_match_cpp_surface() {
    let mut animation = Animation::new("walk");

    assert_eq!(animation.get_name(), "walk");
    assert_eq!(animation.get_duration(), 0.0);
    assert_eq!(animation.get_color(), Animation::DEFAULT_COLOR);
    assert!(animation.get_timelines().next().is_none());
    assert!(animation.get_bones().is_empty());

    animation.set_duration(1.25);
    animation.get_color_mut()[2] = 0.5;

    assert_eq!(animation.get_duration(), 1.25);
    assert_eq!(animation.get_color(), [1.0, 1.0, 0.5, 1.0]);
}

#[test]
fn animation_bones_reports_unique_affected_bone_indices_like_cpp() {
    let mut animation = empty_animation("bones");
    animation
        .bone_timelines
        .push(BoneTimeline::Rotate(RotateTimeline {
            bone_index: 2,
            frames: Vec::new(),
        }));
    animation
        .bone_timelines
        .push(BoneTimeline::ScaleX(ScaleXTimeline {
            bone_index: 0,
            frames: Vec::new(),
        }));
    animation
        .bone_timelines
        .push(BoneTimeline::Rotate(RotateTimeline {
            bone_index: 2,
            frames: Vec::new(),
        }));

    assert_eq!(animation.get_bones(), vec![2, 0]);
}

#[test]
fn skeleton_data_skins_and_events_preserve_cpp_array_order() {
    let mut data = SkeletonData::default();
    for skin_name in ["skin-b", "default", "skin-a"] {
        data.skins.push(SkinData::new(skin_name));
    }
    for event_name in ["event-b", "event-a"] {
        data.events.push(EventData::with_setup_pose(
            event_name, 0, 0.0, "", "", 1.0, 0.0,
        ));
    }

    assert_eq!(
        data.skins
            .iter()
            .map(|skin| skin.name.as_str())
            .collect::<Vec<_>>(),
        vec!["skin-b", "default", "skin-a"]
    );
    assert_eq!(
        data.events
            .iter()
            .map(|event| event.get_name())
            .collect::<Vec<_>>(),
        vec!["event-b", "event-a"]
    );
}

#[cfg(feature = "json")]
#[test]
fn skeleton_data_constraints_follow_cpp_unified_order_after_json_parse() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "child", "parent": "root" }
  ],
  "slots": [
    { "name": "slot", "bone": "root" }
  ],
  "skins": {},
  "constraints": [
    { "type": "physics", "name": "physics", "bone": "child" },
    { "type": "slider", "name": "slider", "bone": "child", "property": "x", "from": 0, "to": 1, "scale": 1, "animation": "anim" },
    { "type": "ik", "name": "ik", "bones": ["child"], "target": "root" },
    { "type": "path", "name": "path", "bones": ["child"], "slot": "slot" },
    { "type": "transform", "name": "transform", "bones": ["child"], "source": "root" }
  ],
  "animations": {
    "anim": {}
  }
}
"#,
    )
    .expect("parse skeleton json");

    assert_eq!(
        data.get_constraints()
            .iter()
            .map(|constraint| (constraint.get_order(), constraint.get_name()))
            .collect::<Vec<_>>(),
        vec![
            (0, "physics"),
            (1, "slider"),
            (2, "ik"),
            (3, "path"),
            (4, "transform"),
        ]
    );
}

#[cfg(feature = "json")]
#[test]
fn skeleton_data_skins_and_events_follow_cpp_order_after_json_parse() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" }
  ],
  "slots": [
    { "name": "slot", "bone": "root" }
  ],
  "skins": [
    { "name": "skin-b" },
    { "name": "default" },
    { "name": "skin-a" }
  ],
  "events": {
    "event-b": { "int": 2 },
    "event-a": { "int": 1 }
  },
  "animations": {}
}
"#,
    )
    .expect("parse skeleton json");

    assert_eq!(
        data.skins
            .iter()
            .map(|skin| skin.name.as_str())
            .collect::<Vec<_>>(),
        vec!["skin-b", "default", "skin-a"]
    );
    assert_eq!(
        data.events
            .iter()
            .map(|event| event.get_name())
            .collect::<Vec<_>>(),
        vec!["event-b", "event-a"]
    );
}

fn empty_animation(name: &str) -> Animation {
    Animation {
        name: name.to_string(),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    }
}
