use crate::{
    BoneData, EventData, IkConstraintData, PathConstraintData, PhysicsConstraintData, PositionMode,
    RotateMode, ScaleYMode, SkeletonData, SliderConstraintData, SlotData, SpacingMode,
    TransformConstraintData,
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
    data.events.insert(
        "event".to_string(),
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
    data.ik_constraints.push(IkConstraintData {
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
    });
    data.transform_constraints.push(TransformConstraintData {
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
    });
    data.path_constraints.push(PathConstraintData {
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
        scale: 0.0,
        local: false,
        animation: None,
    });

    assert_eq!(data.find_bone("root").unwrap().name, "root");
    assert_eq!(data.find_slot("slot").unwrap().name, "slot");
    assert_eq!(data.find_event("event").unwrap().string, "payload");
    assert_eq!(data.find_ik_constraint("ik").unwrap().target, 0);
    assert_eq!(
        data.find_transform_constraint("transform").unwrap().order,
        1
    );
    assert_eq!(data.find_path_constraint("path").unwrap().order, 2);
    assert_eq!(data.find_physics_constraint("physics").unwrap().order, 3);
    assert_eq!(data.find_slider_constraint("slider").unwrap().order, 4);

    assert!(data.find_bone("").is_none());
    assert!(data.find_slot("").is_none());
    assert!(data.find_event("").is_none());
    assert!(data.find_ik_constraint("").is_none());
    assert!(data.find_transform_constraint("").is_none());
    assert!(data.find_path_constraint("").is_none());
    assert!(data.find_physics_constraint("").is_none());
    assert!(data.find_slider_constraint("").is_none());
}
