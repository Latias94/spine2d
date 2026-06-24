use crate::SkeletonData;
use crate::runtime::Skeleton;

const JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "head" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head.png", "x": 1, "y": 2, "rotation": 30, "scaleX": 1, "scaleY": 1, "width": 10, "height": 20 }
      }
    }
  },
  "animations": {}
}
"#;

#[test]
fn parse_slots_and_default_skin_region_attachment() {
    let data = SkeletonData::from_json_str(JSON).unwrap();
    assert_eq!(data.slots.len(), 1);
    assert_eq!(data.skins.len(), 1);
    assert_eq!(
        data.default_skin().map(|skin| skin.name.as_str()),
        Some("default")
    );

    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();

    assert_eq!(skeleton.slots.len(), 1);
    assert_eq!(skeleton.draw_order, vec![0]);
    assert_eq!(skeleton.slots[0].attachment.as_deref(), Some("head"));
    assert_eq!(skeleton.skin.as_deref(), None);

    let skin = skeleton.data.find_skin("default").unwrap();
    let attachment = skin.attachment(0, "head").unwrap();
    match attachment {
        crate::AttachmentData::Region(region) => {
            assert_eq!(region.path, "head.png");
            assert_eq!(region.x, 1.0);
            assert_eq!(region.y, 2.0);
            assert_eq!(region.rotation, 30.0);
            assert_eq!(region.width, 10.0);
            assert_eq!(region.height, 20.0);
        }
        _ => panic!("expected region attachment"),
    }
}

#[test]
fn skeleton_data_default_skin_is_none_when_no_default_skin_exists() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "skins": {
    "custom": {}
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    assert!(data.default_skin().is_none());
}
