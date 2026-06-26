use crate::{
    Animation, BoneData, BoneTimeline, ClippingAttachmentData, ConstraintDataRef, EventData,
    IkConstraintData, PathConstraintData, PhysicsConstraintData, PositionMode, RotateMode,
    RotateTimeline, ScaleXTimeline, ScaleYMode, SkeletonData, SkinData, SliderConstraintData,
    SlotData, SpacingMode, TransformConstraintData, TransformFromProperty, TransformProperty,
    TransformToProperty,
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
        property_offset: 0.0,
        offset: 0.0,
        max: 0.0,
        scale: 0.0,
        local: false,
        animation: None,
        animation_name: None,
    });

    let mut color_bone = BoneData::default();
    color_bone.get_color_mut()[0] = 0.25;
    assert_eq!(color_bone.get_color()[0], 0.25);

    assert_eq!(data.find_bone("root").unwrap().get_name(), "root");
    assert_eq!(
        data.find_bone("child")
            .unwrap()
            .get_parent(&data)
            .unwrap()
            .get_name(),
        "root"
    );
    assert_eq!(data.find_slot("slot").unwrap().get_name(), "slot");
    assert_eq!(
        data.find_slot("slot").unwrap().get_bone(&data).get_name(),
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
    assert_eq!(
        data.find_animation("animation").unwrap().get_name(),
        "animation"
    );
    assert_eq!(
        data.find_constraint::<IkConstraintData>("ik")
            .unwrap()
            .get_target(&data)
            .get_name(),
        "root"
    );
    assert_eq!(
        data.find_constraint::<TransformConstraintData>("transform")
            .unwrap()
            .get_source(&data)
            .get_name(),
        "root"
    );
    assert_eq!(
        data.find_constraint::<PathConstraintData>("path")
            .unwrap()
            .get_slot(&data)
            .get_name(),
        "slot"
    );
    assert_eq!(
        data.find_constraint::<PhysicsConstraintData>("physics")
            .unwrap()
            .get_bone(&data)
            .get_name(),
        "root"
    );
    assert_eq!(
        data.find_constraint::<SliderConstraintData>("slider")
            .unwrap()
            .get_bone(&data)
            .unwrap()
            .get_name(),
        "root"
    );

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
        ConstraintDataRef::Slider(_, data) if data.get_name() == "slider"
    ));

    assert!(data.find_bone("").is_none());
    assert!(data.find_slot("").is_none());
    assert!(data.find_skin("").is_none());
    assert!(data.find_event("").is_none());
    assert!(data.find_animation("").is_none());
    assert!(data.find_constraint::<IkConstraintData>("").is_none());
    assert!(
        data.find_constraint::<TransformConstraintData>("")
            .is_none()
    );
    assert!(data.find_constraint::<PathConstraintData>("").is_none());
    assert!(data.find_constraint::<PhysicsConstraintData>("").is_none());
    assert!(data.find_constraint::<SliderConstraintData>("").is_none());
}

#[test]
fn constraint_data_accessors_match_cpp_surface() {
    let mut ik = IkConstraintData::new("ik");
    assert_eq!(ik.get_name(), "ik");
    assert!(!ik.get_skin_required());
    assert!(ik.get_bones().is_empty());
    let mut relation_data = SkeletonData::default();
    relation_data.bones.push(BoneData {
        index: 0,
        name: "root".to_string(),
        ..BoneData::default()
    });
    relation_data.bones.push(BoneData {
        index: 1,
        name: "child".to_string(),
        parent: Some(0),
        ..BoneData::default()
    });
    relation_data.slots.push(SlotData {
        index: 0,
        name: "slot".to_string(),
        bone: 0,
        setup_pose: crate::SlotSetupPose::default(),
        ..SlotData::default()
    });
    relation_data
        .animations
        .push(Animation::new("slider-animation"));

    assert_eq!(ik.get_target(&relation_data).get_name(), "root");
    assert_eq!(ik.get_scale_y_mode(), ScaleYMode::None);
    assert_eq!(ik.get_mix(), 0.0);
    assert_eq!(ik.get_softness(), 0.0);
    assert_eq!(ik.get_bend_direction(), 0);
    assert!(!ik.get_compress());
    assert!(!ik.get_stretch());

    ik.set_skin_required(true);
    ik.get_bones_mut().extend([1, 2]);
    ik.set_target(&relation_data.bones[1]);
    ik.set_scale_y_mode(ScaleYMode::Volume);
    ik.set_mix(0.75);
    ik.set_softness(4.5);
    ik.set_bend_direction(-1);
    ik.set_compress(true);
    ik.set_stretch(true);

    assert!(ik.get_skin_required());
    assert_eq!(ik.get_bones(), [1, 2]);
    relation_data.bones.push(BoneData {
        index: 2,
        name: "target".to_string(),
        ..BoneData::default()
    });
    relation_data.bones.push(BoneData {
        index: 3,
        name: "new-target".to_string(),
        ..BoneData::default()
    });
    ik.set_target(&relation_data.bones[3]);
    assert_eq!(ik.get_target(&relation_data).get_name(), "new-target");
    assert_eq!(ik.get_scale_y_mode(), ScaleYMode::Volume);
    assert_eq!(ik.get_mix(), 0.75);
    assert_eq!(ik.get_softness(), 4.5);
    assert_eq!(ik.get_bend_direction(), -1);
    assert!(ik.get_compress());
    assert!(ik.get_stretch());

    let mut transform = TransformConstraintData::new("transform");
    assert_eq!(transform.get_name(), "transform");
    assert!(!transform.get_skin_required());
    assert!(transform.get_bones().is_empty());
    assert_eq!(transform.get_source(&relation_data).get_name(), "root");
    assert_eq!(TransformConstraintData::ROTATION, 0);
    assert_eq!(TransformConstraintData::X, 1);
    assert_eq!(TransformConstraintData::Y, 2);
    assert_eq!(TransformConstraintData::SCALE_X, 3);
    assert_eq!(TransformConstraintData::SCALE_Y, 4);
    assert_eq!(TransformConstraintData::SHEAR_Y, 5);
    assert_eq!(transform.get_offset_rotation(), 0.0);
    assert_eq!(transform.get_offset_x(), 0.0);
    assert_eq!(transform.get_offset_y(), 0.0);
    assert_eq!(transform.get_offset_scale_x(), 0.0);
    assert_eq!(transform.get_offset_scale_y(), 0.0);
    assert_eq!(transform.get_offset_shear_y(), 0.0);
    assert!(!transform.get_local_source());
    assert!(!transform.get_local_target());
    assert!(!transform.get_additive());
    assert!(!transform.get_clamp());
    assert!(transform.get_properties().is_empty());
    assert_eq!(transform.get_mix_rotate(), 0.0);
    assert_eq!(transform.get_mix_x(), 0.0);
    assert_eq!(transform.get_mix_y(), 0.0);
    assert_eq!(transform.get_mix_scale_x(), 0.0);
    assert_eq!(transform.get_mix_scale_y(), 0.0);
    assert_eq!(transform.get_mix_shear_y(), 0.0);

    transform.set_skin_required(true);
    transform.get_bones_mut().extend([4, 5]);
    let transform_base = relation_data.bones.len();
    relation_data.bones.extend((0..5).map(|index| BoneData {
        index: transform_base + index,
        name: format!("transform-{index}"),
        ..BoneData::default()
    }));
    transform.set_source(&relation_data.bones[6]);
    transform.set_offset_rotation(10.0);
    transform.set_offset_x(11.0);
    transform.set_offset_y(12.0);
    transform.set_offset_scale_x(13.0);
    transform.set_offset_scale_y(14.0);
    transform.set_offset_shear_y(15.0);
    transform.set_local_source(true);
    transform.set_local_target(true);
    transform.set_additive(true);
    transform.set_clamp(true);
    transform.get_properties_mut().push(TransformFromProperty {
        property: TransformProperty::Rotate,
        offset: 1.5,
        to: vec![TransformToProperty {
            property: TransformProperty::X,
            offset: 2.5,
            max: 3.5,
            scale: 4.5,
        }],
    });
    transform.set_mix_rotate(0.1);
    transform.set_mix_x(0.2);
    transform.set_mix_y(0.3);
    transform.set_mix_scale_x(0.4);
    transform.set_mix_scale_y(0.5);
    transform.set_mix_shear_y(0.6);

    assert!(transform.get_skin_required());
    assert_eq!(transform.get_bones(), [4, 5]);
    let extra_base = relation_data.bones.len();
    relation_data.bones.extend((0..3).map(|index| BoneData {
        index: extra_base + index,
        name: format!("extra-{index}"),
        ..BoneData::default()
    }));
    assert_eq!(
        transform.get_source(&relation_data).get_name(),
        "transform-2"
    );
    assert_eq!(transform.get_offset_rotation(), 10.0);
    assert_eq!(transform.get_offset_x(), 11.0);
    assert_eq!(transform.get_offset_y(), 12.0);
    assert_eq!(transform.get_offset_scale_x(), 13.0);
    assert_eq!(transform.get_offset_scale_y(), 14.0);
    assert_eq!(transform.get_offset_shear_y(), 15.0);
    assert!(transform.get_local_source());
    assert!(transform.get_local_target());
    assert!(transform.get_additive());
    assert!(transform.get_clamp());
    assert_eq!(
        transform.get_properties()[0].property,
        TransformProperty::Rotate
    );
    assert_eq!(transform.get_properties()[0].offset, 1.5);
    assert_eq!(
        transform.get_properties()[0].to[0].property,
        TransformProperty::X
    );
    assert_eq!(transform.get_mix_rotate(), 0.1);
    assert_eq!(transform.get_mix_x(), 0.2);
    assert_eq!(transform.get_mix_y(), 0.3);
    assert_eq!(transform.get_mix_scale_x(), 0.4);
    assert_eq!(transform.get_mix_scale_y(), 0.5);
    assert_eq!(transform.get_mix_shear_y(), 0.6);

    let mut path = PathConstraintData::new("path");
    assert_eq!(path.get_name(), "path");
    assert!(!path.get_skin_required());
    assert!(path.get_bones().is_empty());
    assert_eq!(path.get_slot(&relation_data).get_name(), "slot");
    assert_eq!(path.get_position_mode(), PositionMode::Fixed);
    assert_eq!(path.get_spacing_mode(), SpacingMode::Length);
    assert_eq!(path.get_rotate_mode(), RotateMode::Tangent);
    assert_eq!(path.get_offset_rotation(), 0.0);
    assert_eq!(path.get_position(), 0.0);
    assert_eq!(path.get_spacing(), 0.0);
    assert_eq!(path.get_mix_rotate(), 0.0);
    assert_eq!(path.get_mix_x(), 0.0);
    assert_eq!(path.get_mix_y(), 0.0);

    path.set_skin_required(true);
    path.get_bones_mut().extend([7, 8]);
    let slot_base = relation_data.slots.len();
    relation_data.slots.extend((0..10).map(|index| SlotData {
        index: slot_base + index,
        name: format!("slot-{index}"),
        bone: 0,
        setup_pose: crate::SlotSetupPose::default(),
        ..SlotData::default()
    }));
    path.set_slot(&relation_data.slots[10]);
    path.set_position_mode(PositionMode::Percent);
    path.set_spacing_mode(SpacingMode::Proportional);
    path.set_rotate_mode(RotateMode::ChainScale);
    path.set_offset_rotation(25.0);
    path.set_position(26.0);
    path.set_spacing(27.0);
    path.set_mix_rotate(0.7);
    path.set_mix_x(0.8);
    path.set_mix_y(0.9);

    assert!(path.get_skin_required());
    assert_eq!(path.get_bones(), [7, 8]);
    assert_eq!(path.get_slot(&relation_data).get_name(), "slot-9");
    assert_eq!(path.get_position_mode(), PositionMode::Percent);
    assert_eq!(path.get_spacing_mode(), SpacingMode::Proportional);
    assert_eq!(path.get_rotate_mode(), RotateMode::ChainScale);
    assert_eq!(path.get_offset_rotation(), 25.0);
    assert_eq!(path.get_position(), 26.0);
    assert_eq!(path.get_spacing(), 27.0);
    assert_eq!(path.get_mix_rotate(), 0.7);
    assert_eq!(path.get_mix_x(), 0.8);
    assert_eq!(path.get_mix_y(), 0.9);

    let mut physics = PhysicsConstraintData::new("physics");
    assert_eq!(physics.get_name(), "physics");
    assert!(!physics.get_skin_required());
    assert_eq!(physics.get_bone(&relation_data).get_name(), "root");
    assert_eq!(physics.get_step(), 0.0);
    assert_eq!(physics.get_x(), 0.0);
    assert_eq!(physics.get_y(), 0.0);
    assert_eq!(physics.get_rotate(), 0.0);
    assert_eq!(physics.get_scale_x(), 0.0);
    assert_eq!(physics.get_scale_y_mode(), ScaleYMode::None);
    assert_eq!(physics.get_shear_x(), 0.0);
    assert_eq!(physics.get_limit(), 0.0);
    assert_eq!(physics.get_inertia(), 0.0);
    assert_eq!(physics.get_strength(), 0.0);
    assert_eq!(physics.get_damping(), 0.0);
    assert_eq!(physics.get_mass_inverse(), 0.0);
    assert_eq!(physics.get_wind(), 0.0);
    assert_eq!(physics.get_gravity(), 0.0);
    assert_eq!(physics.get_mix(), 0.0);
    assert!(!physics.get_inertia_global());
    assert!(!physics.get_strength_global());
    assert!(!physics.get_damping_global());
    assert!(!physics.get_mass_global());
    assert!(!physics.get_wind_global());
    assert!(!physics.get_gravity_global());
    assert!(!physics.get_mix_global());

    physics.set_skin_required(true);
    let physics_base = relation_data.bones.len();
    relation_data.bones.extend((0..11).map(|index| BoneData {
        index: physics_base + index,
        name: format!("physics-{index}"),
        ..BoneData::default()
    }));
    physics.set_bone(&relation_data.bones[15]);
    physics.set_step(1.0 / 30.0);
    physics.set_x(0.11);
    physics.set_y(0.12);
    physics.set_rotate(0.13);
    physics.set_scale_x(0.14);
    physics.set_scale_y_mode(ScaleYMode::Uniform);
    physics.set_shear_x(0.15);
    physics.set_limit(20.0);
    physics.set_inertia(0.21);
    physics.set_strength(0.22);
    physics.set_damping(0.23);
    physics.set_mass_inverse(0.24);
    physics.set_wind(0.25);
    physics.set_gravity(0.26);
    physics.set_mix(0.27);
    physics.set_inertia_global(true);
    physics.set_strength_global(true);
    physics.set_damping_global(true);
    physics.set_mass_global(true);
    physics.set_wind_global(true);
    physics.set_gravity_global(true);
    physics.set_mix_global(true);

    assert!(physics.get_skin_required());
    assert_eq!(physics.get_bone(&relation_data).get_name(), "physics-3");
    assert_eq!(physics.get_step(), 1.0 / 30.0);
    assert_eq!(physics.get_x(), 0.11);
    assert_eq!(physics.get_y(), 0.12);
    assert_eq!(physics.get_rotate(), 0.13);
    assert_eq!(physics.get_scale_x(), 0.14);
    assert_eq!(physics.get_scale_y_mode(), ScaleYMode::Uniform);
    assert_eq!(physics.get_shear_x(), 0.15);
    assert_eq!(physics.get_limit(), 20.0);
    assert_eq!(physics.get_inertia(), 0.21);
    assert_eq!(physics.get_strength(), 0.22);
    assert_eq!(physics.get_damping(), 0.23);
    assert_eq!(physics.get_mass_inverse(), 0.24);
    assert_eq!(physics.get_wind(), 0.25);
    assert_eq!(physics.get_gravity(), 0.26);
    assert_eq!(physics.get_mix(), 0.27);
    assert!(physics.get_inertia_global());
    assert!(physics.get_strength_global());
    assert!(physics.get_damping_global());
    assert!(physics.get_mass_global());
    assert!(physics.get_wind_global());
    assert!(physics.get_gravity_global());
    assert!(physics.get_mix_global());

    let mut slider = SliderConstraintData::new("slider");
    assert_eq!(slider.get_name(), "slider");
    assert!(!slider.get_skin_required());
    assert!(slider.get_animation(&relation_data).is_none());
    assert!(!slider.get_additive());
    assert!(!slider.get_loop());
    assert!(slider.get_bone(&relation_data).is_none());
    assert_eq!(slider.get_property(), None);
    assert_eq!(slider.get_offset(), 0.0);
    assert_eq!(slider.get_property_offset(), 0.0);
    assert_eq!(slider.get_scale(), 0.0);
    assert_eq!(slider.get_max(), 0.0);
    assert!(!slider.get_local());
    assert_eq!(slider.get_time(), 0.0);
    assert_eq!(slider.get_mix(), 0.0);

    slider.set_skin_required(true);
    slider.set_additive(true);
    slider.set_loop(true);
    let slider_base = relation_data.bones.len();
    relation_data.bones.extend((0..13).map(|index| BoneData {
        index: slider_base + index,
        name: format!("slider-{index}"),
        ..BoneData::default()
    }));
    slider.set_bone(Some(&relation_data.bones[24]));
    slider.set_property(Some(TransformProperty::ShearY));
    slider.set_property_offset(7.0);
    slider.set_offset(31.0);
    slider.set_scale(32.0);
    slider.set_max(33.0);
    slider.set_local(true);
    slider.set_time(34.0);
    slider.set_mix(0.35);
    slider.set_animation(&relation_data.animations[0]);

    assert!(slider.get_skin_required());
    assert_eq!(
        slider
            .get_animation(&relation_data)
            .map(Animation::get_name),
        Some("slider-animation")
    );
    assert!(slider.get_additive());
    assert!(slider.get_loop());
    assert_eq!(
        slider.get_bone(&relation_data).map(BoneData::get_name),
        Some("slider-1")
    );
    assert_eq!(slider.get_property(), Some(TransformProperty::ShearY));
    assert_eq!(slider.get_property_offset(), 7.0);
    assert_eq!(slider.get_offset(), 31.0);
    assert_eq!(slider.get_scale(), 32.0);
    assert_eq!(slider.get_max(), 33.0);
    assert!(slider.get_local());
    assert_eq!(slider.get_time(), 34.0);
    assert_eq!(slider.get_mix(), 0.35);

    slider.animation = None;
    slider.animation_name = None;
    slider.set_bone(None);
    slider.set_property(None);
    assert!(slider.get_animation(&relation_data).is_none());
    assert!(slider.get_bone(&relation_data).is_none());
    assert_eq!(slider.get_property(), None);

    let mut clip_data = SkeletonData::default();
    clip_data.slots.push(SlotData {
        index: 0,
        name: "clip-start".to_string(),
        ..SlotData::default()
    });
    clip_data.slots.push(SlotData {
        index: 1,
        name: "clip-end".to_string(),
        ..SlotData::default()
    });
    let clip = ClippingAttachmentData {
        vertex_id: 0,
        name: "clip".to_string(),
        color: ClippingAttachmentData::DEFAULT_COLOR,
        vertices: crate::MeshVertices::Unweighted(Vec::new()),
        end_slot: Some(1),
        convex: true,
        inverse: false,
    };

    let mut clip_with_end = clip.clone();
    clip_with_end.set_end_slot(Some(&clip_data.slots[1]));

    assert_eq!(
        clip.get_end_slot(&clip_data).map(SlotData::get_name),
        Some("clip-end")
    );
    assert_eq!(
        clip_with_end
            .get_end_slot(&clip_data)
            .map(SlotData::get_name),
        Some("clip-end")
    );
}

#[test]
fn skeleton_data_header_getters_match_cpp_names() {
    let mut data = SkeletonData::default();
    data.set_name("hero");
    data.set_version("4.3.8");
    data.set_hash("123");
    data.set_x(1.0);
    data.set_y(2.0);
    data.set_width(3.0);
    data.set_height(4.0);
    data.set_reference_scale(50.0);
    data.set_fps(24.0);
    data.set_images_path("images/");
    data.set_audio_path("audio/");

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
