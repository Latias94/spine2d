use crate::runtime::{AnimationState, AnimationStateData, MixBlend};
use crate::{SequenceMode, Skeleton, SkeletonData, apply_animation, build_draw_list};

const CROSS_SLOT_LINKED_MESH_SEQUENCE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "source" },
    { "name": "slot1", "bone": "root", "attachment": "linked" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "source": {
          "type": "mesh",
          "path": "source",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0],
          "sequence": { "count": 3, "start": 1, "digits": 2, "setup": 0 }
        }
      },
      "slot1": {
        "linked": {
          "type": "linkedmesh",
          "source": "source",
          "slot": "slot0"
        }
      }
    }
  },
  "animations": {
    "sequence": {
      "attachments": {
        "default": {
          "slot0": {
            "source": {
              "sequence": [
                { "time": 0.25, "mode": "hold", "index": 2, "delay": 0.1 }
              ]
            }
          }
        }
      }
    }
  }
}
"#;

#[test]
fn sequence_timeline_drives_slot_sequence_index_and_render_path() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "wing" } ],
  "skins": {
    "default": {
      "slot0": {
        "wing": {
          "type": "region",
          "path": "wing",
          "sequence": { "count": 3, "start": 1, "digits": 2, "setup": 1 },
          "width": 2,
          "height": 2
        }
      }
    }
  },
  "animations": {
    "fly": {
      "attachments": {
        "default": {
          "slot0": {
            "wing": {
              "sequence": [
                { "time": 0, "mode": "loop", "index": 0, "delay": 0.1 },
                { "time": 1, "mode": "loop", "index": 0, "delay": 0.1 }
              ]
            }
          }
        }
      }
    }
  }
}
"#,
    )
    .unwrap();

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    assert_eq!(skeleton.slots[0].sequence_index, -1);
    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].texture_path, "wing02");

    let state_data = AnimationStateData::new(data);
    let mut state = AnimationState::new(state_data);
    state.set_animation(0, "fly", true);

    state.update(0.0);
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].sequence_index, 0);
    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws[0].texture_path, "wing01");

    state.update(0.15);
    state.apply(&mut skeleton);
    assert_eq!(skeleton.slots[0].sequence_index, 1);
    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws[0].texture_path, "wing02");
}

#[test]
fn sequence_timeline_applies_to_cross_slot_linked_mesh_timeline_slots() {
    let data = SkeletonData::from_json_str(CROSS_SLOT_LINKED_MESH_SEQUENCE).unwrap();
    let animation = data.find_animation("sequence").unwrap();
    assert_eq!(animation.sequence_timelines.len(), 1);
    let timeline = &animation.sequence_timelines[0];
    assert_eq!(timeline.skin, "default");
    assert_eq!(timeline.slot_index, 0);
    assert_eq!(timeline.attachment, "source");
    assert_eq!(timeline.frames.len(), 1);
    assert_eq!(timeline.frames[0].mode, SequenceMode::Hold);
    assert_eq!(timeline.frames[0].index, 2);

    let source = data
        .find_skin("default")
        .and_then(|skin| skin.get_attachment(0, "source"))
        .and_then(|attachment| match attachment {
            crate::AttachmentData::Mesh(mesh) => Some(mesh),
            _ => None,
        })
        .expect("source mesh");
    assert_eq!(source.timeline_slots, vec![1]);

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    assert_eq!(skeleton.slots[0].sequence_index, -1);
    assert_eq!(skeleton.slots[1].sequence_index, -1);

    apply_animation(
        animation,
        &mut skeleton,
        0.25,
        false,
        1.0,
        MixBlend::Replace,
    );
    assert_eq!(skeleton.slots[0].sequence_index, 2);
    assert_eq!(skeleton.slots[1].sequence_index, 2);

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::Setup);
    assert_eq!(skeleton.slots[0].sequence_index, -1);
    assert_eq!(skeleton.slots[1].sequence_index, -1);
}
