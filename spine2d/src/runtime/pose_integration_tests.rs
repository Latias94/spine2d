use crate::runtime::{AnimationState, AnimationStateData};
use crate::{
    BoneData, BoneTimeline, Curve, Inherit, RotateFrame, RotateTimeline, TranslateTimeline,
    Vec2Frame,
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

#[test]
fn animation_state_apply_drives_skeleton_pose() {
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "move".to_string(),
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

    let mut animation_index = HashMap::new();
    animation_index.insert("move".to_string(), 0);

    let skeleton_data = Arc::new(crate::SkeletonData {
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
            x: 2.0,
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
        slots: Vec::new(),
        skins: indexmap::IndexMap::new(),
        events: indexmap::IndexMap::new(),
        animations: vec![animation],
        animation_index,
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut state = AnimationState::new(AnimationStateData::new(skeleton_data.clone()));
    let mut skeleton = crate::Skeleton::new(skeleton_data);

    state.set_animation(0, "move", false);
    state.update(0.5);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    assert_approx(skeleton.bones[0].x, 7.0);
    assert_approx(skeleton.bones[0].world_x, 7.0);
}

#[test]
fn animation_state_mixes_pose_between_entries() {
    let anim_a = crate::runtime::finalize_animation(crate::Animation {
        name: "a".to_string(),
        duration: 0.0,
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
    let anim_b = crate::runtime::finalize_animation(crate::Animation {
        name: "b".to_string(),
        duration: 0.0,
        color: crate::Animation::DEFAULT_COLOR,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 20.0,
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

    let mut animation_index = HashMap::new();
    animation_index.insert("a".to_string(), 0);
    animation_index.insert("b".to_string(), 1);

    let skeleton_data = Arc::new(crate::SkeletonData {
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
        slots: Vec::new(),
        skins: indexmap::IndexMap::new(),
        events: indexmap::IndexMap::new(),
        animations: vec![anim_a, anim_b],
        animation_index,
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut data = AnimationStateData::new(skeleton_data.clone());
    data.set_mix("a", "b", 1.0);

    let mut state = AnimationState::new(data);
    let mut skeleton = crate::Skeleton::new(skeleton_data);

    state.set_animation(0, "a", false);
    state.update(0.0);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_approx(skeleton.bones[0].x, 10.0);

    state.set_animation(0, "b", false);
    state.update(0.5);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_approx(skeleton.bones[0].x, 15.0);
}

#[test]
fn track_entry_shortest_rotation_disables_rotation_accumulator() {
    fn run_case(shortest_rotation: bool) -> f32 {
        let animation = crate::runtime::finalize_animation(crate::Animation {
            name: "spin".to_string(),
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

        let mut animation_index = HashMap::new();
        animation_index.insert("spin".to_string(), 0);

        let skeleton_data = Arc::new(crate::SkeletonData {
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
            slots: Vec::new(),
            skins: indexmap::IndexMap::new(),
            events: indexmap::IndexMap::new(),
            animations: vec![animation],
            animation_index,
            ik_constraints: Vec::new(),
            transform_constraints: Vec::new(),
            path_constraints: Vec::new(),
            physics_constraints: Vec::new(),
            slider_constraints: Vec::new(),
        });

        let mut state = AnimationState::new(AnimationStateData::new(skeleton_data.clone()));
        let mut skeleton = crate::Skeleton::new(skeleton_data);
        skeleton.setup_pose();

        let entry = state.set_animation(0, "spin", false);
        entry.set_alpha(&mut state, 0.5);
        entry.set_shortest_rotation(&mut state, shortest_rotation);

        state.update(0.0);
        state.apply(&mut skeleton);
        assert_approx(skeleton.bones[0].rotation, 85.0);

        state.update(1.0);
        state.apply(&mut skeleton);
        skeleton.bones[0].rotation
    }

    assert_approx(run_case(false), 95.0);
    assert_approx(run_case(true), -85.0);
}

#[test]
fn track_entry_reverse_samples_from_animation_end() {
    let animation = crate::runtime::finalize_animation(crate::Animation {
        name: "move".to_string(),
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

    let mut animation_index = HashMap::new();
    animation_index.insert("move".to_string(), 0);

    let skeleton_data = Arc::new(crate::SkeletonData {
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
        slots: Vec::new(),
        skins: indexmap::IndexMap::new(),
        events: indexmap::IndexMap::new(),
        animations: vec![animation],
        animation_index,
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    });

    let mut state = AnimationState::new(AnimationStateData::new(skeleton_data.clone()));
    let mut skeleton = crate::Skeleton::new(skeleton_data);
    skeleton.setup_pose();

    let entry = state.set_animation(0, "move", false);
    entry.set_reverse(&mut state, true);

    state.update(0.25);
    state.apply(&mut skeleton);

    // Reverse uses applyTime = duration - animationTime, so 0.25 samples at 0.75.
    assert_approx(skeleton.bones[0].x, 7.5);
}
