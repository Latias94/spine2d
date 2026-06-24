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
    data.slots.push(SlotData {
        name: "slot".to_string(),
        ..SlotData::default()
    });
    data.skins
        .insert("skin-key".to_string(), SkinData::new("skin", 0));
    data.events.insert(
        "event-key".to_string(),
        EventData {
            name: "event".to_string(),
            int_value: 1,
            float_value: 2.0,
            string: "payload".to_string(),
            audio_path: String::new(),
            volume: 1.0,
            balance: 0.0,
        },
    );
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

    assert_eq!(data.find_bone("root").unwrap().name, "root");
    assert_eq!(data.find_slot("slot").unwrap().name, "slot");
    assert_eq!(data.find_skin("skin").unwrap().name, "skin");
    assert_eq!(data.find_event("event").unwrap().string, "payload");
    assert_eq!(data.find_animation("animation").unwrap().name, "animation");
    assert_eq!(data.find_ik_constraint("ik").unwrap().target, 0);
    assert_eq!(
        data.find_transform_constraint("transform").unwrap().order,
        4
    );
    assert_eq!(data.find_path_constraint("path").unwrap().order, 1);
    assert_eq!(data.find_physics_constraint("physics").unwrap().order, 3);
    assert_eq!(data.find_slider_constraint("slider").unwrap().order, 0);

    let constraints = data.constraints();
    assert_eq!(
        constraints
            .iter()
            .map(|constraint| (constraint.order(), constraint.name()))
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
        ConstraintDataRef::Slider(0, data) if data.name == "slider"
    ));
    assert_eq!(constraints[0].data_index(), 0);
    assert!(!constraints[0].skin_required());

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

    assert_eq!(animation.bones(), vec![2, 0]);
}

#[test]
fn skeleton_data_skins_and_events_preserve_cpp_array_order() {
    let mut data = SkeletonData::default();
    for skin_name in ["skin-b", "default", "skin-a"] {
        data.skins
            .insert(skin_name.to_string(), SkinData::new(skin_name, 0));
    }
    for event_name in ["event-b", "event-a"] {
        data.events.insert(
            event_name.to_string(),
            EventData {
                name: event_name.to_string(),
                int_value: 0,
                float_value: 0.0,
                string: String::new(),
                audio_path: String::new(),
                volume: 1.0,
                balance: 0.0,
            },
        );
    }

    assert_eq!(
        data.skins
            .values()
            .map(|skin| skin.name.as_str())
            .collect::<Vec<_>>(),
        vec!["skin-b", "default", "skin-a"]
    );
    assert_eq!(
        data.events
            .values()
            .map(|event| event.name.as_str())
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
        data.constraints()
            .iter()
            .map(|constraint| (constraint.order(), constraint.name()))
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
            .values()
            .map(|skin| skin.name.as_str())
            .collect::<Vec<_>>(),
        vec!["skin-b", "default", "skin-a"]
    );
    assert_eq!(
        data.events
            .values()
            .map(|event| event.name.as_str())
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
