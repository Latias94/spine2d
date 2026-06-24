use crate::runtime::MixBlend;
use crate::{Skeleton, SkeletonData, SkinData, apply_animation};

const JSON: &str = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root", "x": 1, "y": 2, "length": 3 }
  ],
  "slots": [
    { "name": "pathSlot", "bone": "root", "attachment": "p" }
  ],
  "skins": [
    {
      "name": "default",
      "attachments": {
        "pathSlot": {
          "p": {
            "type": "path",
            "vertexCount": 6,
            "vertices": [ 0, 0, 0, 0, 3.3333333, 0, 6.6666665, 0, 10, 0, 10, 0 ],
            "lengths": [ 10 ],
            "closed": false,
            "constantSpeed": true
          }
        }
      }
    }
  ],
  "path": [
    {
      "name": "pc_fixed",
      "order": 0,
      "bones": ["root"],
      "target": "pathSlot",
      "positionMode": "fixed",
      "spacingMode": "length",
      "rotateMode": "tangent",
      "position": 2,
      "spacing": 3,
      "mixRotate": 0,
      "mixX": 0,
      "mixY": 0
    },
    {
      "name": "pc_percent",
      "order": 0,
      "bones": ["root"],
      "target": "pathSlot",
      "positionMode": "percent",
      "spacingMode": "percent",
      "rotateMode": "tangent",
      "position": 0.25,
      "spacing": 0.1,
      "mixRotate": 0,
      "mixX": 0,
      "mixY": 0
    }
  ],
  "animations": {
    "anim": {
      "path": {
        "pc_fixed": {
          "position": [
            { "time": 0.0, "position": 2.0 },
            { "time": 1.0, "position": 4.0 }
          ],
          "spacing": [
            { "time": 0.0, "spacing": 3.0 },
            { "time": 1.0, "spacing": 5.0 }
          ]
        },
        "pc_percent": {
          "position": [
            { "time": 0.0, "position": 0.25 },
            { "time": 1.0, "position": 0.50 }
          ],
          "spacing": [
            { "time": 0.0, "spacing": 0.1 },
            { "time": 1.0, "spacing": 0.2 }
          ]
        }
      }
    }
  }
}
"#;

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-3,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn json_scale_applies_to_geometry_and_conditionally_to_path_fields() {
    let data = SkeletonData::from_json_str_with_scale(JSON, 2.0).unwrap();

    assert_approx(data.bones[0].x, 2.0);
    assert_approx(data.bones[0].y, 4.0);
    assert_approx(data.bones[0].length, 6.0);

    let fixed = &data.path_constraints[0];
    assert_approx(fixed.position, 4.0);
    assert_approx(fixed.spacing, 6.0);

    let percent = &data.path_constraints[1];
    assert_approx(percent.position, 0.25);
    assert_approx(percent.spacing, 0.1);

    let anim = data.find_animation("anim").unwrap().clone();
    let mut skeleton = Skeleton::new(data);
    skeleton.setup_pose();
    apply_animation(&anim, &mut skeleton, 1.0, false, 1.0, MixBlend::Replace);

    assert_approx(skeleton.path_constraints[0].position, 8.0);
    assert_approx(skeleton.path_constraints[0].spacing, 10.0);
    assert_approx(skeleton.path_constraints[1].position, 0.50);
    assert_approx(skeleton.path_constraints[1].spacing, 0.2);
}

#[test]
fn json_array_skin_color_matches_cpp_skin_color_field() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "skins": [
    { "name": "default", "attachments": {} },
    { "name": "accent", "color": "11223344", "attachments": {} }
  ],
  "animations": {}
}
"#,
    )
    .unwrap();

    let accent = data.find_skin("accent").unwrap();
    let expected = [
        0x11 as f32 / 255.0,
        0x22 as f32 / 255.0,
        0x33 as f32 / 255.0,
        0x44 as f32 / 255.0,
    ];
    for (actual, expected) in accent.color.into_iter().zip(expected) {
        assert_approx(actual, expected);
    }

    let default = data.find_skin("default").unwrap();
    for (actual, expected) in default.color.into_iter().zip(SkinData::DEFAULT_COLOR) {
        assert_approx(actual, expected);
    }
}

#[test]
fn json_animation_color_matches_cpp_animation_color_field() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "animations": {
    "default": {},
    "accent": { "color": "11223344" }
  }
}
"#,
    )
    .unwrap();

    let accent = data.find_animation("accent").unwrap();
    let expected = [
        0x11 as f32 / 255.0,
        0x22 as f32 / 255.0,
        0x33 as f32 / 255.0,
        0x44 as f32 / 255.0,
    ];
    for (actual, expected) in accent.color.into_iter().zip(expected) {
        assert_approx(actual, expected);
    }

    let default = data.find_animation("default").unwrap();
    for (actual, expected) in default
        .color
        .into_iter()
        .zip(crate::Animation::DEFAULT_COLOR)
    {
        assert_approx(actual, expected);
    }
}

#[test]
fn json_attachment_default_colors_match_cpp_constructors() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot", "bone": "root" } ],
  "skins": [
    {
      "name": "default",
      "attachments": {
        "slot": {
          "point": { "type": "point", "x": 1.0, "y": 2.0, "rotation": 3.0 },
          "path": { "type": "path", "vertexCount": 0, "vertices": [], "lengths": [], "closed": false, "constantSpeed": true },
          "bbox": { "type": "boundingbox", "vertexCount": 0, "vertices": [] },
          "clip": { "type": "clipping", "vertexCount": 0, "vertices": [], "end": "slot", "convex": true, "inverse": false }
        }
      }
    }
  ],
  "animations": {}
}
"#,
    )
    .unwrap();

    let skin = data.find_skin("default").unwrap();

    match skin.attachments[0].get("point").unwrap() {
        crate::AttachmentData::Point(point) => {
            for (actual, expected) in point
                .color
                .into_iter()
                .zip(crate::PointAttachmentData::DEFAULT_COLOR)
            {
                assert_approx(actual, expected);
            }
        }
        other => panic!("expected point, got {other:?}"),
    }
    match skin.attachments[0].get("path").unwrap() {
        crate::AttachmentData::Path(path) => {
            for (actual, expected) in path
                .color
                .into_iter()
                .zip(crate::PathAttachmentData::DEFAULT_COLOR)
            {
                assert_approx(actual, expected);
            }
        }
        other => panic!("expected path, got {other:?}"),
    }
    match skin.attachments[0].get("bbox").unwrap() {
        crate::AttachmentData::BoundingBox(bbox) => {
            for (actual, expected) in bbox
                .color
                .into_iter()
                .zip(crate::BoundingBoxAttachmentData::DEFAULT_COLOR)
            {
                assert_approx(actual, expected);
            }
        }
        other => panic!("expected bounding box, got {other:?}"),
    }
    match skin.attachments[0].get("clip").unwrap() {
        crate::AttachmentData::Clipping(clip) => {
            for (actual, expected) in clip
                .color
                .into_iter()
                .zip(crate::ClippingAttachmentData::DEFAULT_COLOR)
            {
                assert_approx(actual, expected);
            }
        }
        other => panic!("expected clipping, got {other:?}"),
    }
}
