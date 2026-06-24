use crate::{
    Animation, AttachmentTimeline, DrawOrderTimeline, Event, EventTimeline, SkeletonData,
    TimelineKind, TimelineRef,
};

#[test]
fn json_animation_preserves_object_order_in_timeline_order() {
    let json = r#"
    {
      "bones": [{"name": "root"}],
      "slots": [{"name": "slot", "bone": "root"}],
      "animations": {
        "mix": {
          "slots": {
            "slot": {
              "attachment": [
                {"time": 0.0, "name": "a"}
              ],
              "rgba": [
                {"time": 0.0, "color": "FFFFFFFF"}
              ]
            }
          },
          "bones": {
            "root": {
              "translate": [
                {"time": 0.0, "x": 1.0, "y": 2.0}
              ],
              "rotate": [
                {"time": 0.0, "angle": 3.0}
              ]
            }
          },
          "drawOrder": [
            {"time": 0.0}
          ]
        }
      }
    }
    "#;

    let data = SkeletonData::from_json_str(json).expect("parse json");
    let animation = data.find_animation("mix").expect("animation exists");
    assert_eq!(
        animation.timeline_order(),
        vec![
            TimelineKind::SlotAttachment(0),
            TimelineKind::SlotColor(0),
            TimelineKind::Bone(1),
            TimelineKind::Bone(0),
            TimelineKind::DrawOrder,
        ]
    );
}

#[test]
fn finalize_animation_keeps_explicit_timeline_order() {
    let animation = Animation {
        name: "mix".to_string(),
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
        timeline_order: vec![TimelineKind::DrawOrder],
    };

    let finalized = crate::runtime::finalize_animation(animation);
    assert_eq!(finalized.timeline_order(), vec![TimelineKind::DrawOrder]);
}

#[test]
fn animation_timelines_exposes_unified_cpp_order() {
    let animation = Animation {
        name: "mix".to_string(),
        duration: 0.0,
        event_timeline: Some(EventTimeline {
            events: vec![Event {
                time: 0.0,
                name: "hit".to_string(),
                int_value: 0,
                float_value: 0.0,
                string: String::new(),
                audio_path: String::new(),
                volume: 1.0,
                balance: 0.0,
            }],
        }),
        bone_timelines: Vec::new(),
        deform_timelines: Vec::new(),
        sequence_timelines: Vec::new(),
        slot_attachment_timelines: vec![AttachmentTimeline {
            slot_index: 0,
            frames: Vec::new(),
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
        draw_order_timeline: Some(DrawOrderTimeline { frames: Vec::new() }),
        draw_order_folder_timelines: Vec::new(),
        timeline_order: vec![TimelineKind::SlotAttachment(0), TimelineKind::DrawOrder],
    };

    let labels = animation
        .timelines()
        .map(|timeline| match timeline {
            TimelineRef::SlotAttachment { index, .. } => format!("SlotAttachment({index})"),
            TimelineRef::DrawOrder { .. } => "DrawOrder".to_string(),
            TimelineRef::Event { .. } => "Event".to_string(),
            other => format!("{other:?}"),
        })
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["SlotAttachment(0)", "DrawOrder", "Event"]);
}
