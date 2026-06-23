use crate::runtime::{AnimationState, AnimationStateData, MixInterpolation};
use crate::{
    Animation, AttachmentData, AttachmentFrame, AttachmentTimeline, BlendMode, BoneData,
    BoneTimeline, Curve, DrawOrderFolderFrame, DrawOrderFolderTimeline, DrawOrderFrame,
    DrawOrderTimeline, Inherit, MixBlend, RegionAttachmentData, Skeleton, SkeletonData, SkinData,
    SlotData, TranslateTimeline, Vec2Frame, apply_animation,
};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-6,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

fn base_skeleton_data() -> SkeletonData {
    SkeletonData {
        spine_version: None,
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
        skins: HashMap::new(),
        events: HashMap::new(),
        animations: Vec::new(),
        animation_index: HashMap::new(),
        ik_constraints: Vec::new(),
        transform_constraints: Vec::new(),
        path_constraints: Vec::new(),
        physics_constraints: Vec::new(),
        slider_constraints: Vec::new(),
    }
}

fn additive_path_physics_json() -> String {
    r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "b", "parent": "root" }
  ],
  "slots": [
    { "name": "pathSlot", "bone": "root", "attachment": "p" }
  ],
  "skins": {
    "default": {
      "pathSlot": {
        "p": {
          "type": "path",
          "vertexCount": 3,
          "vertices": [ 0, 0, 10, 0, 20, 0 ],
          "lengths": [ 20 ],
          "closed": false,
          "constantSpeed": true
        }
      }
    }
  },
  "path": [
    {
      "name": "pc",
      "bones": ["b"],
      "target": "pathSlot",
      "positionMode": "percent",
      "spacingMode": "length",
      "rotateMode": "tangent",
      "position": 0,
      "spacing": 0,
      "mixRotate": 0,
      "mixX": 0,
      "mixY": 0
    }
  ],
  "physics": [
    { "name": "p0", "bone": "b", "wind": 0 },
    { "name": "p1", "bone": "b", "inertia": 0.2 }
  ],
  "animations": {
    "base": {
      "path": {
        "pc": {
          "position": [
            { "time": 0.0, "value": 10.0 },
            { "time": 1.0, "value": 10.0 }
          ],
          "spacing": [
            { "time": 0.0, "value": 6.0 },
            { "time": 1.0, "value": 6.0 }
          ],
          "mix": [
            { "time": 0.0, "mixRotate": 0.25, "mixX": 0.25, "mixY": 0.25 },
            { "time": 1.0, "mixRotate": 0.25, "mixX": 0.25, "mixY": 0.25 }
          ]
        }
      },
      "physics": {
        "p0": {
          "wind": [
            { "time": 0.0, "value": 5.0 },
            { "time": 1.0, "value": 5.0 }
          ]
        },
        "p1": {
          "inertia": [
            { "time": 0.0, "value": 0.4 },
            { "time": 1.0, "value": 0.4 }
          ]
        }
      }
    },
    "overlay": {
      "path": {
        "pc": {
          "position": [
            { "time": 0.0, "value": 3.0 },
            { "time": 1.0, "value": 3.0 }
          ],
          "spacing": [
            { "time": 0.0, "value": 2.0 },
            { "time": 1.0, "value": 2.0 }
          ],
          "mix": [
            { "time": 0.0, "mixRotate": 0.5, "mixX": 0.5, "mixY": 0.5 },
            { "time": 1.0, "mixRotate": 0.5, "mixX": 0.5, "mixY": 0.5 }
          ]
        }
      },
      "physics": {
        "p0": {
          "wind": [
            { "time": 0.0, "value": 2.0 },
            { "time": 1.0, "value": 2.0 }
          ]
        },
        "p1": {
          "inertia": [
            { "time": 0.0, "value": 0.8 },
            { "time": 1.0, "value": 0.8 }
          ]
        }
      }
    }
  }
}
"#
    .to_string()
}

fn setup_additive_path_physics_state() -> (AnimationState, Skeleton) {
    let data = SkeletonData::from_json_str(&additive_path_physics_json()).unwrap();
    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let skeleton = Skeleton::new(data);

    state.set_animation(0, "base", false).unwrap();
    let overlay = state.set_animation(1, "overlay", false).unwrap();
    overlay.set_additive(&mut state, true);

    (state, skeleton)
}

#[test]
fn track_entry_additive_adds_current_pose_without_entry_mix_blend() {
    let anim_base = crate::runtime::finalize_animation(Animation {
        name: "base".to_string(),
        duration: 0.0,
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
    let anim_overlay = crate::runtime::finalize_animation(Animation {
        name: "overlay".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 3.0,
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

    let mut data = base_skeleton_data();
    data.animations = vec![anim_base, anim_overlay];
    data.animation_index.insert("base".to_string(), 0);
    data.animation_index.insert("overlay".to_string(), 1);
    let data = Arc::new(data);

    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    state.set_animation(0, "base", false).unwrap();
    let overlay = state.set_animation(1, "overlay", false).unwrap();
    overlay.set_additive(&mut state, true);

    skeleton.setup_pose();
    state.apply(&mut skeleton);

    assert_approx(skeleton.bones[0].x, 13.0);
}

#[test]
fn track_entry_additive_mixes_out_as_additive() {
    let anim_base = crate::runtime::finalize_animation(Animation {
        name: "base".to_string(),
        duration: 0.0,
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
    let anim_overlay = crate::runtime::finalize_animation(Animation {
        name: "overlay".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: vec![BoneTimeline::Translate(TranslateTimeline {
            bone_index: 0,
            frames: vec![Vec2Frame {
                time: 0.0,
                x: 3.0,
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

    let mut data = base_skeleton_data();
    data.animations = vec![anim_base, anim_overlay];
    data.animation_index.insert("base".to_string(), 0);
    data.animation_index.insert("overlay".to_string(), 1);
    let data = Arc::new(data);

    let state_data = AnimationStateData::new(data.clone());
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    state.set_animation(0, "base", false).unwrap();
    let overlay = state.set_animation(1, "overlay", false).unwrap();
    overlay.set_additive(&mut state, true);

    state.set_empty_animation(1, 1.0);
    state.update(0.5);

    skeleton.setup_pose();
    state.apply(&mut skeleton);

    assert_approx(skeleton.bones[0].x, 11.5);
}

#[test]
fn additive_next_entry_does_not_hold_outgoing_numeric_timeline() {
    let anim_base = crate::runtime::finalize_animation(Animation {
        name: "base".to_string(),
        duration: 0.0,
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
    let anim_overlay = crate::runtime::finalize_animation(Animation {
        name: "overlay".to_string(),
        duration: 0.0,
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

    let mut data = base_skeleton_data();
    data.animations = vec![anim_base, anim_overlay];
    data.animation_index.insert("base".to_string(), 0);
    data.animation_index.insert("overlay".to_string(), 1);
    let data = Arc::new(data);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("base", "overlay", 1.0).unwrap();
    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    state.set_animation(0, "base", false).unwrap();
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_approx(skeleton.bones[0].x, 10.0);

    let overlay = state.set_animation(0, "overlay", false).unwrap();
    overlay.set_additive(&mut state, true);
    state.update(0.5);

    skeleton.setup_pose();
    state.apply(&mut skeleton);

    // Upstream does not hold the outgoing base timeline against an additive incoming
    // timeline, but both entries still use the half-complete mix weight.
    assert_approx(skeleton.bones[0].x, 7.5);
}

#[test]
fn additive_path_and_physics_timelines_follow_upstream_add_rules() {
    let (mut state, mut skeleton) = setup_additive_path_physics_state();

    skeleton.setup_pose();
    state.update(0.5);
    state.apply(&mut skeleton);

    assert_approx(skeleton.path_constraints[0].position, 13.0);
    assert_approx(skeleton.path_constraints[0].spacing, 2.0);
    assert_approx(skeleton.path_constraints[0].mix_rotate, 0.75);
    assert_approx(skeleton.path_constraints[0].mix_x, 0.75);
    assert_approx(skeleton.path_constraints[0].mix_y, 0.75);

    assert_approx(skeleton.physics_constraints[0].wind, 7.0);
    assert_approx(skeleton.physics_constraints[1].inertia, 0.8);
}

#[test]
fn physics_constraint_timeline_applies_negative_alpha_like_cpp() {
    let data = SkeletonData::from_json_str(&additive_path_physics_json()).unwrap();
    let (_, anim) = data.animation("base").unwrap();
    let mut skeleton = Skeleton::new(data.clone());

    skeleton.setup_pose();
    apply_animation(anim, &mut skeleton, 0.5, false, -0.5, MixBlend::Replace);

    assert_approx(skeleton.physics_constraints[0].wind(), -2.5);
    assert_approx(skeleton.physics_constraints[1].inertia(), 0.1);
}

#[test]
fn mix_interpolation_fast_slow_affects_mix_out_alpha() {
    let anim_a = crate::runtime::finalize_animation(Animation {
        name: "a".to_string(),
        duration: 0.0,
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
    let anim_b = crate::runtime::finalize_animation(Animation {
        name: "b".to_string(),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut data = base_skeleton_data();
    data.animations = vec![anim_a, anim_b];
    data.animation_index.insert("a".to_string(), 0);
    data.animation_index.insert("b".to_string(), 1);
    let data = Arc::new(data);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("a", "b", 1.0).unwrap();

    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    state.set_animation(0, "a", false).unwrap();
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_approx(skeleton.bones[0].x, 10.0);

    let b = state.set_animation(0, "b", false).unwrap();
    b.set_mix_interpolation(&mut state, MixInterpolation::FastSlow);

    state.update(0.5);
    skeleton.setup_pose();
    state.apply(&mut skeleton);

    // FastSlow maps raw 0.5 to 0.75, so the outgoing A timeline keeps 25% alpha.
    assert_approx(skeleton.bones[0].x, 2.5);
}

#[test]
fn mixing_thresholds_gate_attachment_and_draw_order_from_mixing_from() {
    let anim_a = crate::runtime::finalize_animation(Animation {
        name: "a".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: vec![AttachmentFrame {
                time: 0.0,
                name: Some("A".to_string()),
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
        draw_order_timeline: Some(DrawOrderTimeline {
            frames: vec![DrawOrderFrame {
                time: 0.0,
                draw_order_to_setup_index: Some(vec![1, 0]),
            }],
        }),
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    // B does not key attachments/draw order, so A can be held via mix thresholds.
    let anim_b = crate::runtime::finalize_animation(Animation {
        name: "b".to_string(),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut data = base_skeleton_data();
    data.slots = vec![
        SlotData {
            name: "s0".to_string(),
            bone: 0,
            attachment: Some("setup0".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
        SlotData {
            name: "s1".to_string(),
            bone: 0,
            attachment: Some("setup1".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
    ];

    // Attachments must exist in a skin to be applied by attachment timelines or setup poses.
    let mut attachments = vec![IndexMap::new(), IndexMap::new()];
    attachments[0].insert(
        "setup0".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup0".to_string(),
            path: "setup0.png".to_string(),
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
    attachments[0].insert(
        "A".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "A".to_string(),
            path: "A.png".to_string(),
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
    attachments[1].insert(
        "setup1".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup1".to_string(),
            path: "setup1.png".to_string(),
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
    data.skins.insert(
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

    data.animations = vec![anim_a, anim_b];
    data.animation_index.insert("a".to_string(), 0);
    data.animation_index.insert("b".to_string(), 1);
    let data = Arc::new(data);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("a", "b", 1.0).unwrap();

    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    let a = state.set_animation(0, "a", false).unwrap();
    a.set_mix_attachment_threshold(&mut state, 0.5);
    a.set_mix_draw_order_threshold(&mut state, 0.5);

    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("A"));
    assert_eq!(skeleton.draw_order, vec![1, 0]);

    state.set_animation(0, "b", false).unwrap();

    // mix=0.4: mixingFrom(A) still applies attachment/draw order.
    state.update(0.4);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("A"));
    assert_eq!(skeleton.draw_order, vec![1, 0]);

    // mix=0.6: mixingFrom(A) no longer applies attachment/draw order.
    state.update(0.2);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("setup0"));
    assert_eq!(skeleton.draw_order, vec![0, 1]);
}

#[test]
fn mix_interpolation_gates_attachment_and_draw_order_thresholds() {
    let anim_a = crate::runtime::finalize_animation(Animation {
        name: "a".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: vec![AttachmentFrame {
                time: 0.0,
                name: Some("A".to_string()),
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
        draw_order_timeline: Some(DrawOrderTimeline {
            frames: vec![DrawOrderFrame {
                time: 0.0,
                draw_order_to_setup_index: Some(vec![1, 0]),
            }],
        }),
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });
    let anim_b = crate::runtime::finalize_animation(Animation {
        name: "b".to_string(),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut data = base_skeleton_data();
    data.slots = vec![
        SlotData {
            name: "s0".to_string(),
            bone: 0,
            attachment: Some("setup0".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
        SlotData {
            name: "s1".to_string(),
            bone: 0,
            attachment: Some("setup1".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
    ];

    let mut attachments = vec![IndexMap::new(), IndexMap::new()];
    attachments[0].insert(
        "setup0".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup0".to_string(),
            path: "setup0.png".to_string(),
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
    attachments[0].insert(
        "A".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "A".to_string(),
            path: "A.png".to_string(),
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
    attachments[1].insert(
        "setup1".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup1".to_string(),
            path: "setup1.png".to_string(),
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
    data.skins.insert(
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

    data.animations = vec![anim_a, anim_b];
    data.animation_index.insert("a".to_string(), 0);
    data.animation_index.insert("b".to_string(), 1);
    let data = Arc::new(data);

    let mut state_data = AnimationStateData::new(data.clone());
    state_data.set_mix("a", "b", 1.0).unwrap();

    let mut state = AnimationState::new(state_data);
    let mut skeleton = Skeleton::new(data);

    let a = state.set_animation(0, "a", false).unwrap();
    a.set_mix_attachment_threshold(&mut state, 0.6);
    a.set_mix_draw_order_threshold(&mut state, 0.6);
    skeleton.setup_pose();
    state.apply(&mut skeleton);

    let b = state.set_animation(0, "b", false).unwrap();
    b.set_mix_interpolation(&mut state, MixInterpolation::FastSlow);

    state.update(0.5);
    skeleton.setup_pose();
    state.apply(&mut skeleton);

    // Raw 0.5 would still pass threshold 0.6; FastSlow interpolates to 0.75 and gates both.
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("setup0"));
    assert_eq!(skeleton.draw_order, vec![0, 1]);
}

#[test]
fn draw_order_folder_applies_after_draw_order_timeline() {
    let anim_a = crate::runtime::finalize_animation(Animation {
        name: "a".to_string(),
        duration: 1.0,
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
                draw_order_to_setup_index: Some(vec![2, 1, 0]),
            }],
        }),
        draw_order_folder_timelines: vec![DrawOrderFolderTimeline {
            slots: vec![1, 2],
            frames: vec![
                DrawOrderFolderFrame {
                    time: 0.5,
                    folder_draw_order: Some(vec![1, 0]),
                },
                DrawOrderFolderFrame {
                    time: 1.0,
                    folder_draw_order: None,
                },
            ],
        }],
        timeline_order: Vec::new(),
    });

    let mut data = base_skeleton_data();
    data.slots = vec![
        SlotData {
            name: "s0".to_string(),
            bone: 0,
            attachment: Some("setup0".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
        SlotData {
            name: "s1".to_string(),
            bone: 0,
            attachment: Some("setup1".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
        SlotData {
            name: "s2".to_string(),
            bone: 0,
            attachment: Some("setup2".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            has_dark: false,
            dark_color: [0.0, 0.0, 0.0],
            blend: BlendMode::Normal,
            ..Default::default()
        },
    ];

    let mut attachments = vec![IndexMap::new(), IndexMap::new(), IndexMap::new()];
    for (slot, name) in [
        ("setup0", "setup0"),
        ("setup1", "setup1"),
        ("setup2", "setup2"),
    ] {
        let slot_index = match slot {
            "setup0" => 0,
            "setup1" => 1,
            _ => 2,
        };
        attachments[slot_index].insert(
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

    data.skins.insert(
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
    data.animations = vec![anim_a];
    data.animation_index.insert("a".to_string(), 0);
    let data = Arc::new(data);

    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut skeleton = Skeleton::new(data);

    state.set_animation(0, "a", false).unwrap();

    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.draw_order, vec![1, 2, 0]);

    state.update(0.6);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.draw_order, vec![2, 1, 0]);

    state.update(0.5);
    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.draw_order, vec![1, 2, 0]);
}

#[test]
fn track0_additive_does_not_override_alpha_attachment_threshold_for_attachments() {
    let anim_a = crate::runtime::finalize_animation(Animation {
        name: "a".to_string(),
        duration: 0.0,
        event_timeline: None,
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: vec![AttachmentFrame {
                time: 0.0,
                name: Some("A".to_string()),
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
        timeline_order: Vec::new(),
    });
    let anim_b = crate::runtime::finalize_animation(Animation {
        name: "b".to_string(),
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
        draw_order_timeline: None,
        draw_order_folder_timelines: Vec::new(),
        timeline_order: Vec::new(),
    });

    let mut data = base_skeleton_data();
    data.slots = vec![SlotData {
        name: "slot".to_string(),
        bone: 0,
        attachment: Some("setup".to_string()),
        color: [1.0, 1.0, 1.0, 1.0],
        has_dark: false,
        dark_color: [0.0, 0.0, 0.0],
        blend: BlendMode::Normal,
        ..Default::default()
    }];

    let mut attachments = vec![IndexMap::new()];
    attachments[0].insert(
        "setup".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "setup".to_string(),
            path: "setup.png".to_string(),
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
    attachments[0].insert(
        "A".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "A".to_string(),
            path: "A.png".to_string(),
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
    data.skins.insert(
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
    data.animations = vec![anim_a, anim_b];
    data.animation_index.insert("a".to_string(), 0);
    data.animation_index.insert("b".to_string(), 1);
    let data = Arc::new(data);

    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut skeleton = Skeleton::new(data);

    let a = state.set_animation(0, "a", false).unwrap();
    a.set_alpha(&mut state, 0.5);
    a.set_alpha_attachment_threshold(&mut state, 0.6);
    a.set_additive(&mut state, true);

    skeleton.setup_pose();
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("setup"));
}
