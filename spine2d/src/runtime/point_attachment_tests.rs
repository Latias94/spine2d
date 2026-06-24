use crate::runtime::Skeleton;
use crate::{AttachmentData, SkeletonData};

const JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root", "x": 2, "y": 3, "rotation": 90 } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "p" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "p": { "type": "point", "x": 10, "y": 0, "rotation": 30 }
      }
    }
  },
  "animations": {}
}
"#;

#[test]
fn point_attachment_computes_world_position_and_rotation() {
    let data = SkeletonData::from_json_str(JSON).unwrap();
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let attachment = skeleton.attachment(0, "p").unwrap();
    let AttachmentData::Point(p) = attachment else {
        panic!("expected point attachment");
    };

    let bone = &skeleton.bones[skeleton.slots[0].bone];
    let pos = p.compute_world_position(bone);
    let rot = p.compute_world_rotation(bone);

    assert!((pos[0] - 2.0).abs() <= 1.0e-6);
    assert!((pos[1] - 13.0).abs() <= 1.0e-6);
    assert!((rot - 120.0).abs() <= 1.0e-6);
}
