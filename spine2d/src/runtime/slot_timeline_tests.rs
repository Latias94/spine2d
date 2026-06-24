use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, apply_animation, build_draw_list};

const SKELETON_ATTACHMENT_TIMELINE: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "a" } ],
  "skins": {
    "default": {
      "slot0": {
        "a": { "type": "region", "path": "a.png", "width": 10, "height": 10 },
        "b": { "type": "region", "path": "b.png", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {
    "anim": {
      "slots": {
        "slot0": {
          "attachment": [
            { "time": 0.5, "name": "b" }
          ]
        }
      }
    }
  }
}
"#;

const SKELETON_DRAW_ORDER: &str = r#"
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
  "animations": {
    "anim": {
      "drawOrder": [
        {
          "time": 0,
          "offsets": [
            { "slot": "slot1", "offset": -1 }
          ]
        }
      ]
    }
  }
}
"#;

const SKELETON_DRAW_ORDER_FOLDER: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "a" },
    { "name": "slot1", "bone": "root", "attachment": "b" },
    { "name": "slot2", "bone": "root", "attachment": "c" },
    { "name": "slot3", "bone": "root", "attachment": "d" }
  ],
  "skins": {
    "default": {
      "slot0": { "a": { "type": "region", "path": "a.png", "width": 10, "height": 10 } },
      "slot1": { "b": { "type": "region", "path": "b.png", "width": 10, "height": 10 } },
      "slot2": { "c": { "type": "region", "path": "c.png", "width": 10, "height": 10 } },
      "slot3": { "d": { "type": "region", "path": "d.png", "width": 10, "height": 10 } }
    }
  },
  "animations": {
    "anim": {
      "drawOrder": [
        {
          "time": 0,
          "offsets": [
            { "slot": "slot3", "offset": -3 }
          ]
        }
      ],
      "drawOrderFolder": {
        "middle": {
          "slots": ["slot1", "slot2"],
          "keys": [
            {
              "time": 0,
              "offsets": [
                { "slot": "slot2", "offset": -1 }
              ]
            },
            { "time": 1 }
          ]
        }
      }
    }
  }
}
"#;

#[test]
fn slot_attachment_timeline_switches_attachment() {
    let data = SkeletonData::from_json_str(SKELETON_ATTACHMENT_TIMELINE).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("anim").unwrap();
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    apply_animation(animation, &mut skeleton, 0.25, false, 1.0, MixBlend::First);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("a"));

    apply_animation(animation, &mut skeleton, 0.75, false, 1.0, MixBlend::First);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("b"));

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].texture_path, "b.png");
}

#[test]
fn draw_order_timeline_reorders_slots() {
    let data = SkeletonData::from_json_str(SKELETON_DRAW_ORDER).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("anim").unwrap();
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    assert_eq!(skeleton.draw_order, vec![0, 1]);

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::First);
    assert_eq!(skeleton.draw_order, vec![1, 0]);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 2);
    assert_eq!(draw_list.draws[0].texture_path, "b.png");
    assert_eq!(draw_list.draws[1].texture_path, "a.png");
}

#[test]
fn draw_order_folder_timeline_reorders_only_folder_slots() {
    let data = SkeletonData::from_json_str(SKELETON_DRAW_ORDER_FOLDER).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    let (_, animation) = data.animation("anim").unwrap();
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    apply_animation(animation, &mut skeleton, 0.0, false, 1.0, MixBlend::First);
    assert_eq!(skeleton.draw_order, vec![3, 0, 2, 1]);

    apply_animation(animation, &mut skeleton, 1.0, false, 1.0, MixBlend::First);
    assert_eq!(skeleton.draw_order, vec![3, 0, 1, 2]);
}
