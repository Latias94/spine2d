use crate::{Atlas, SkeletonData, build_draw_list, build_draw_list_with_atlas};

fn assert_approx(actual: f32, expected: f32) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= 1.0e-6,
        "expected {expected}, got {actual} (diff {diff})"
    );
}

#[test]
fn build_draw_list_region_attachment_quad() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "head" } ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head.png", "x": 1, "y": 2, "rotation": 0, "scaleX": 1, "scaleY": 1, "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.vertices.len(), 4);
    assert_eq!(draw_list.indices.len(), 6);
    assert_eq!(draw_list.draws[0].texture_path, "head.png");
    assert_eq!(draw_list.draws[0].blend, crate::BlendMode::Normal);
    assert!(!draw_list.draws[0].premultiplied_alpha);

    // Vertex order matches spine-cpp `RegionAttachment.computeWorldVertices`: BR, BL, UL, UR.
    let br = draw_list.vertices[0].position;
    let bl = draw_list.vertices[1].position;
    let ul = draw_list.vertices[2].position;
    let ur = draw_list.vertices[3].position;
    assert_approx(br[0], 2.0);
    assert_approx(br[1], 1.0);
    assert_approx(bl[0], 0.0);
    assert_approx(bl[1], 1.0);
    assert_approx(ul[0], 0.0);
    assert_approx(ul[1], 3.0);
    assert_approx(ur[0], 2.0);
    assert_approx(ur[1], 3.0);
}

#[test]
fn build_draw_list_batches_draws_by_texture_path() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "a" },
    { "name": "slot1", "bone": "root", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "slot0": { "a": { "type": "region", "path": "page.png", "width": 2, "height": 2 } },
      "slot1": { "b": { "type": "region", "path": "page.png", "width": 2, "height": 2 } }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert_eq!(draw_list.draws[0].blend, crate::BlendMode::Normal);
    assert!(!draw_list.draws[0].premultiplied_alpha);
    assert_eq!(draw_list.indices.len(), 12);
    assert_eq!(draw_list.draws[0].first_index, 0);
    assert_eq!(draw_list.draws[0].index_count, 12);
}

#[test]
fn build_draw_list_splits_draws_by_vertex_color() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "a", "color": "ffffffff" },
    { "name": "slot1", "bone": "root", "attachment": "b", "color": "808080ff" }
  ],
  "skins": {
    "default": {
      "slot0": { "a": { "type": "region", "path": "page.png", "width": 2, "height": 2 } },
      "slot1": { "b": { "type": "region", "path": "page.png", "width": 2, "height": 2 } }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 2);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert_eq!(draw_list.draws[0].first_index, 0);
    assert_eq!(draw_list.draws[0].index_count, 6);
    assert_eq!(draw_list.draws[1].texture_path, "page.png");
    assert_eq!(draw_list.draws[1].first_index, 6);
    assert_eq!(draw_list.draws[1].index_count, 6);
}

#[test]
fn build_draw_list_with_atlas_sets_uv_and_page_texture() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "head" } ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "x": 0, "y": 0, "rotation": 0, "scaleX": 1, "scaleY": 1, "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
head
  rotate: false
  xy: 16, 32
  size: 16, 16
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert!(!draw_list.draws[0].premultiplied_alpha);

    let uv0 = draw_list.vertices[0].uv;
    let uv2 = draw_list.vertices[2].uv;
    // Vertex order: BR, BL, UL, UR.
    assert_approx(uv0[0], 32.0 / 64.0);
    assert_approx(uv0[1], 48.0 / 64.0);
    assert_approx(uv2[0], 16.0 / 64.0);
    assert_approx(uv2[1], 32.0 / 64.0);
}

#[test]
fn build_draw_list_with_rotated_atlas_region_rotates_uvs() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "head" } ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "x": 0, "y": 0, "rotation": 0, "scaleX": 1, "scaleY": 1, "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
head
  rotate: true
  xy: 16, 32
  size: 16, 8
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert!(!draw_list.draws[0].premultiplied_alpha);

    let u = 16.0 / 64.0;
    let v = 32.0 / 64.0;
    let u2 = 24.0 / 64.0;
    let v2 = 48.0 / 64.0;

    let uv0 = draw_list.vertices[0].uv;
    let uv1 = draw_list.vertices[1].uv;
    let uv2 = draw_list.vertices[2].uv;
    let uv3 = draw_list.vertices[3].uv;

    // Vertex order: BR, BL, UL, UR.
    // For degrees=90, spine-cpp maps:
    // BR=(u2,v), BL=(u2,v2), UL=(u,v2), UR=(u,v).
    assert_approx(uv0[0], u2);
    assert_approx(uv0[1], v);
    assert_approx(uv1[0], u2);
    assert_approx(uv1[1], v2);
    assert_approx(uv2[0], u);
    assert_approx(uv2[1], v2);
    assert_approx(uv3[0], u);
    assert_approx(uv3[1], v);
}

#[test]
fn build_draw_list_mesh_attachment_outputs_vertices_and_indices() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.vertices.len(), 4);
    assert_eq!(draw_list.indices, vec![0, 1, 2, 2, 3, 0]);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].blend, crate::BlendMode::Normal);
    assert!(!draw_list.draws[0].premultiplied_alpha);
}

#[test]
fn build_draw_list_with_atlas_maps_mesh_uvs_to_region_rect() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
mesh0
  rotate: false
  xy: 16, 32
  size: 16, 16
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert!(!draw_list.draws[0].premultiplied_alpha);
    let uv0 = draw_list.vertices[0].uv;
    let uv2 = draw_list.vertices[2].uv;
    assert_approx(uv0[0], 16.0 / 64.0);
    assert_approx(uv0[1], 32.0 / 64.0);
    assert_approx(uv2[0], 32.0 / 64.0);
    assert_approx(uv2[1], 48.0 / 64.0);
}

#[test]
fn build_draw_list_with_atlas_maps_region_uvs_for_rotated_region_90() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "head" } ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
head
  rotate: true
  xy: 16, 32
  size: 10, 20
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.vertices.len(), 4);

    // Vertex order: BR, BL, UL, UR.
    let u = 16.0 / 64.0;
    let v = 32.0 / 64.0;
    let u2 = (16.0 + 20.0) / 64.0;
    let v2 = (32.0 + 10.0) / 64.0;

    let uv_br = draw_list.vertices[0].uv;
    let uv_bl = draw_list.vertices[1].uv;
    let uv_ul = draw_list.vertices[2].uv;
    let uv_ur = draw_list.vertices[3].uv;

    assert_approx(uv_br[0], u2);
    assert_approx(uv_br[1], v);
    assert_approx(uv_bl[0], u2);
    assert_approx(uv_bl[1], v2);
    assert_approx(uv_ul[0], u);
    assert_approx(uv_ul[1], v2);
    assert_approx(uv_ur[0], u);
    assert_approx(uv_ur[1], v);
}

#[test]
fn build_draw_list_skips_attachments_on_inactive_bones() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "weapon", "parent": "root", "skin": true }
  ],
  "slots": [
    { "name": "slot0", "bone": "weapon", "attachment": "a" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "a": { "type": "region", "path": "a", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    // `weapon` is skinRequired and no skin is set, so the bone is inactive and spine-cpp skips
    // rendering attachments in this slot.
    assert_eq!(skeleton.bones.len(), 2);
    assert!(!skeleton.bones[1].active);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 0);
    assert_eq!(draw_list.vertices.len(), 0);
    assert_eq!(draw_list.indices.len(), 0);
}

#[test]
fn build_draw_list_weighted_mesh_uses_multiple_bones() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "child", "parent": "root", "x": 10 }
  ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [
            2, 0, -1,-1, 0.5, 1, -1,-1, 0.5,
            2, 0,  1,-1, 0.5, 1,  1,-1, 0.5,
            2, 0,  1, 1, 0.5, 1,  1, 1, 0.5,
            2, 0, -1, 1, 0.5, 1, -1, 1, 0.5
          ],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.vertices.len(), 4);
    let p0 = draw_list.vertices[0].position;
    let p2 = draw_list.vertices[2].position;
    assert_approx(p0[0], 4.0);
    assert_approx(p0[1], -1.0);
    assert_approx(p2[0], 6.0);
    assert_approx(p2[1], 1.0);
}

#[test]
fn build_draw_list_clipping_end_slot_ends_after_target_slot() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "clip", "bone": "root", "attachment": "clipper" },
    { "name": "a", "bone": "root", "attachment": "a" },
    { "name": "b", "bone": "root", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "clip": {
        "clipper": {
          "type": "clipping",
          "end": "a",
          "vertexCount": 3,
          "vertices": [ 0,0, 0.5,0, 0,0.5 ]
        }
      },
      "a": {
        "a": { "type": "region", "path": "a.png", "width": 2, "height": 2 }
      },
      "b": {
        "b": { "type": "region", "path": "b.png", "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 2);

    // The clipping attachment ends after slot `a`, so slot `b` must not be clipped.
    let draw_b = draw_list.draws.last().unwrap();
    assert_eq!(draw_b.texture_path, "b.png");
    assert_eq!(draw_b.index_count, 6);

    // Slot `b` is a normal region quad (BR, BL, UL, UR) and should match the un-clipped setup pose.
    assert!(draw_list.vertices.len() >= 4);
    let v = &draw_list.vertices[draw_list.vertices.len() - 4..];
    assert_approx(v[0].position[0], 1.0);
    assert_approx(v[0].position[1], -1.0);
    assert_approx(v[1].position[0], -1.0);
    assert_approx(v[1].position[1], -1.0);
    assert_approx(v[2].position[0], -1.0);
    assert_approx(v[2].position[1], 1.0);
    assert_approx(v[3].position[0], 1.0);
    assert_approx(v[3].position[1], 1.0);
}

#[test]
fn build_draw_list_clipping_end_slot_ends_on_non_rendered_attachment_slot() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "clip", "bone": "root", "attachment": "clipper" },
    { "name": "a", "bone": "root", "attachment": "a" },
    { "name": "b", "bone": "root", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "clip": {
        "clipper": {
          "type": "clipping",
          "end": "a",
          "vertexCount": 3,
          "vertices": [ 0,0, 0.5,0, 0,0.5 ]
        }
      },
      "a": {
        "a": { "type": "boundingbox", "vertexCount": 4, "vertices": [ 0,0, 1,0, 1,1, 0,1 ] }
      },
      "b": {
        "b": { "type": "region", "path": "b.png", "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);

    // Clipping ends after slot `a`, even though slot `a` is not renderable.
    let draw_b = draw_list.draws.last().unwrap();
    assert_eq!(draw_b.texture_path, "b.png");
    assert_eq!(draw_b.index_count, 6);

    assert!(draw_list.vertices.len() >= 4);
    let v = &draw_list.vertices[draw_list.vertices.len() - 4..];
    assert_approx(v[0].position[0], 1.0);
    assert_approx(v[0].position[1], -1.0);
    assert_approx(v[1].position[0], -1.0);
    assert_approx(v[1].position[1], -1.0);
    assert_approx(v[2].position[0], -1.0);
    assert_approx(v[2].position[1], 1.0);
    assert_approx(v[3].position[0], 1.0);
    assert_approx(v[3].position[1], 1.0);
}

#[test]
fn build_draw_list_clipping_slot_inactive_does_not_start_clipping() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "clipBone", "parent": "root" },
    { "name": "drawBone", "parent": "root" }
  ],
  "slots": [
    { "name": "clip", "bone": "clipBone", "attachment": "clipper" },
    { "name": "b", "bone": "drawBone", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "clip": {
        "clipper": {
          "type": "clipping",
          "end": "b",
          "vertexCount": 3,
          "vertices": [ 0,0, 0.5,0, 0,0.5 ]
        }
      },
      "b": {
        "b": { "type": "region", "path": "b.png", "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    // Simulate skinRequired exclusion: clip slot's bone is inactive.
    let clip_bone_index = 1usize;
    skeleton.bones[clip_bone_index].active = false;

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    let draw_b = draw_list.draws.last().unwrap();
    assert_eq!(draw_b.texture_path, "b.png");
    assert_eq!(draw_b.index_count, 6);

    // Slot `b` should not be clipped.
    assert!(draw_list.vertices.len() >= 4);
    let v = &draw_list.vertices[draw_list.vertices.len() - 4..];
    assert_approx(v[0].position[0], 1.0);
    assert_approx(v[0].position[1], -1.0);
    assert_approx(v[1].position[0], -1.0);
    assert_approx(v[1].position[1], -1.0);
    assert_approx(v[2].position[0], -1.0);
    assert_approx(v[2].position[1], 1.0);
    assert_approx(v[3].position[0], 1.0);
    assert_approx(v[3].position[1], 1.0);
}

#[test]
fn build_draw_list_clipping_end_slot_bone_inactive_still_ends_clipping() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [
    { "name": "root" },
    { "name": "skinRequired", "parent": "root", "skin": true }
  ],
  "slots": [
    { "name": "clip", "bone": "root", "attachment": "clipper" },
    { "name": "a", "bone": "skinRequired", "attachment": "a" },
    { "name": "b", "bone": "root", "attachment": "b" }
  ],
  "skins": {
    "default": {
      "clip": {
        "clipper": {
          "type": "clipping",
          "end": "a",
          "vertexCount": 3,
          "vertices": [ 0,0, 0.5,0, 0,0.5 ]
        }
      },
      "a": {
        "a": { "type": "region", "path": "a.png", "width": 2, "height": 2 }
      },
      "b": {
        "b": { "type": "region", "path": "b.png", "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    // `skinRequired` is inactive (no skin set), but it is the clip end slot. Clipping must still
    // end there so subsequent slots are not clipped.
    assert_eq!(skeleton.bones.len(), 2);
    assert!(!skeleton.bones[1].active);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);

    let draw_b = draw_list.draws.last().unwrap();
    assert_eq!(draw_b.texture_path, "b.png");
    assert_eq!(draw_b.index_count, 6);

    assert_eq!(draw_list.vertices.len(), 4);
    let v = &draw_list.vertices;
    assert_approx(v[0].position[0], 1.0);
    assert_approx(v[0].position[1], -1.0);
    assert_approx(v[1].position[0], -1.0);
    assert_approx(v[1].position[1], -1.0);
    assert_approx(v[2].position[0], -1.0);
    assert_approx(v[2].position[1], 1.0);
    assert_approx(v[3].position[0], 1.0);
    assert_approx(v[3].position[1], 1.0);
}

#[test]
fn build_draw_list_splits_draws_by_blend_mode() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "a", "blend": "additive" },
    { "name": "slot1", "bone": "root", "attachment": "b", "blend": "multiply" }
  ],
  "skins": {
    "default": {
      "slot0": { "a": { "type": "region", "path": "page.png", "width": 2, "height": 2 } },
      "slot1": { "b": { "type": "region", "path": "page.png", "width": 2, "height": 2 } }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 2);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert_eq!(draw_list.draws[0].blend, crate::BlendMode::Additive);
    assert_eq!(draw_list.draws[1].texture_path, "page.png");
    assert_eq!(draw_list.draws[1].blend, crate::BlendMode::Multiply);
}

#[test]
fn build_draw_list_with_atlas_marks_pma_and_premultiplies_vertex_color() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "head", "color": "ffffff80" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
pma: true
head
  rotate: false
  xy: 0, 0
  size: 16, 16
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws.len(), 1);
    assert!(draw_list.draws[0].premultiplied_alpha);

    let c = draw_list.vertices[0].color;
    assert_approx(c[3], 128.0 / 255.0);
    assert_approx(c[0], c[3]);
    assert_approx(c[1], c[3]);
    assert_approx(c[2], c[3]);
}

#[test]
fn build_draw_list_multiplies_skeleton_slot_and_attachment_tints() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "head", "color": "80808080" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head.png", "width": 2, "height": 2, "color": "ff000080" }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();

    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.color = [0.5, 1.0, 1.0, 0.5];
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.vertices.len(), 4);

    let half = 128.0 / 255.0;
    let expected_r = 0.5 * half;
    let expected_a = 0.5 * half * half;

    for v in &draw_list.vertices {
        assert_approx(v.color[0], expected_r);
        assert_approx(v.color[1], 0.0);
        assert_approx(v.color[2], 0.0);
        assert_approx(v.color[3], expected_a);
    }
}

#[test]
fn build_draw_list_two_color_tint_sets_dark_color_alpha_by_pma_and_premultiplies_dark_rgb() {
    let json = r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "head", "color": "ffffffff", "dark": "ffffff" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "width": 2, "height": 2 }
      }
    }
  },
  "animations": {}
}
"#;
    let data = SkeletonData::from_json_str(json).unwrap();

    let atlas_pma = Atlas::parse(
        r#"
page.png
size: 64,64
pma: true
head
  rotate: false
  xy: 0, 0
  size: 16, 16
"#,
    )
    .unwrap();

    let atlas_non_pma = Atlas::parse(
        r#"
page.png
size: 64,64
pma: false
head
  rotate: false
  xy: 0, 0
  size: 16, 16
"#,
    )
    .unwrap();

    // PMA: dark.rgb is premultiplied by final alpha, and dark.a=1.
    let mut skeleton = crate::Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.color = [1.0, 1.0, 1.0, 0.5];
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas_pma);
    assert_eq!(draw_list.vertices.len(), 4);
    for v in &draw_list.vertices {
        assert_approx(v.dark_color[0], 0.5);
        assert_approx(v.dark_color[1], 0.5);
        assert_approx(v.dark_color[2], 0.5);
        assert_approx(v.dark_color[3], 1.0);
    }

    // non-PMA: dark.rgb is not premultiplied, and dark.a=0.
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.color = [1.0, 1.0, 1.0, 0.5];
    skeleton.update_world_transform_with_physics(crate::Physics::None);
    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas_non_pma);
    assert_eq!(draw_list.vertices.len(), 4);
    for v in &draw_list.vertices {
        assert_approx(v.dark_color[0], 1.0);
        assert_approx(v.dark_color[1], 1.0);
        assert_approx(v.dark_color[2], 1.0);
        assert_approx(v.dark_color[3], 0.0);
    }
}

#[test]
fn build_draw_list_with_atlas_applies_region_trim_offset_and_orig() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "head" } ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
head
  rotate: false
  xy: 0, 0
  size: 10, 10
  orig: 20, 20
  offset: 5, 5
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.vertices.len(), 4);

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for v in &draw_list.vertices {
        min_x = min_x.min(v.position[0]);
        min_y = min_y.min(v.position[1]);
        max_x = max_x.max(v.position[0]);
        max_y = max_y.max(v.position[1]);
    }

    assert_approx(min_x, -2.5);
    assert_approx(min_y, -2.5);
    assert_approx(max_x, 2.5);
    assert_approx(max_y, 2.5);
}

#[test]
fn build_draw_list_with_atlas_applies_rotated_region_trim_using_packed_swap() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "head" } ],
  "skins": {
    "default": {
      "slot0": {
        "head": { "type": "region", "path": "head", "width": 10, "height": 10 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 64,64
head
  rotate: true
  xy: 0, 0
  size: 5, 9
  orig: 20, 20
  offset: 0, 0
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.vertices.len(), 4);

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for v in &draw_list.vertices {
        min_x = min_x.min(v.position[0]);
        min_y = min_y.min(v.position[1]);
        max_x = max_x.max(v.position[0]);
        max_y = max_y.max(v.position[1]);
    }

    assert_approx(min_x, -5.0);
    assert_approx(min_y, -5.0);
    assert_approx(max_x, -2.5);
    assert_approx(max_y, -0.5);
}

#[test]
fn build_draw_list_with_atlas_maps_mesh_uvs_with_trim_and_rotate_90() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 100,100
mesh0
  rotate: 90
  bounds: 30, 20, 30, 40
  orig: 50, 60
  offset: 7, 11
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");

    // Expected values from upstream `MeshAttachment.updateRegion()` (spine-ts) for degrees=90.
    let uv0 = draw_list.vertices[0].uv; // region_uv (0,0)
    let uv2 = draw_list.vertices[2].uv; // region_uv (1,1)
    assert_approx(uv0[0], 0.21);
    assert_approx(uv0[1], 0.57);
    assert_approx(uv2[0], 0.81);
    assert_approx(uv2[1], 0.07);
}

#[test]
fn build_draw_list_with_atlas_maps_mesh_uvs_with_trim_and_rotate_180() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 100,100
mesh0
  rotate: 180
  bounds: 30, 20, 30, 40
  orig: 50, 60
  offset: 7, 11
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");

    // Expected values from upstream `MeshAttachment.updateRegion()` (spine-ts) for degrees=180.
    let uv0 = draw_list.vertices[0].uv; // region_uv (0,0)
    let uv2 = draw_list.vertices[2].uv; // region_uv (1,1)
    assert_approx(uv0[0], 0.67);
    assert_approx(uv0[1], 0.69);
    assert_approx(uv2[0], 0.17);
    assert_approx(uv2[1], 0.09);
}

#[test]
fn build_draw_list_with_atlas_maps_mesh_uvs_with_trim_and_rotate_270() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
  "skins": {
    "default": {
      "slot0": {
        "mesh0": {
          "type": "mesh",
          "path": "mesh0",
          "uvs": [0,0, 1,0, 1,1, 0,1],
          "vertices": [-1,-1, 1,-1, 1,1, -1,1],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let atlas = Atlas::parse(
        r#"
page.png
size: 100,100
mesh0
  rotate: 270
  bounds: 30, 20, 30, 40
  orig: 50, 60
  offset: 7, 11
"#,
    )
    .unwrap();

    let draw_list = build_draw_list_with_atlas(&skeleton, &atlas);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");

    // Expected values from upstream `MeshAttachment.updateRegion()` (spine-ts) for degrees=270.
    let uv0 = draw_list.vertices[0].uv; // region_uv (0,0)
    let uv2 = draw_list.vertices[2].uv; // region_uv (1,1)
    assert_approx(uv0[0], 0.79);
    assert_approx(uv0[1], 0.13);
    assert_approx(uv2[0], 0.19);
    assert_approx(uv2[1], 0.63);
}

#[test]
fn build_draw_list_clipping_attachment_clips_region_geometry() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "clip" },
    { "name": "slot1", "bone": "root", "attachment": "region0" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "clip": {
          "type": "clipping",
          "end": "slot1",
          "vertexCount": 4,
          "vertices": [-1,-1, 1,-1, 1,1, -1,1]
        }
      },
      "slot1": {
        "region0": { "type": "region", "path": "page.png", "width": 4, "height": 4 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert_eq!(draw_list.draws[0].blend, crate::BlendMode::Normal);
    assert!(!draw_list.draws[0].premultiplied_alpha);
    assert_eq!(draw_list.draws[0].first_index, 0);
    assert_eq!(draw_list.draws[0].index_count, draw_list.indices.len());
    assert_eq!(draw_list.indices.len() % 3, 0);
    assert!(
        draw_list.vertices.len() > 4,
        "expected clipping to increase vertex count (got {})",
        draw_list.vertices.len()
    );

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for v in &draw_list.vertices {
        assert!(v.position[0].is_finite());
        assert!(v.position[1].is_finite());
        min_x = min_x.min(v.position[0]);
        min_y = min_y.min(v.position[1]);
        max_x = max_x.max(v.position[0]);
        max_y = max_y.max(v.position[1]);
    }

    let eps = 1.0e-4;
    assert!(min_x >= -1.0 - eps, "min_x={min_x}");
    assert!(min_y >= -1.0 - eps, "min_y={min_y}");
    assert!(max_x <= 1.0 + eps, "max_x={max_x}");
    assert!(max_y <= 1.0 + eps, "max_y={max_y}");
}

#[test]
fn build_draw_list_inverse_clipping_attachment_keeps_outer_region_geometry() {
    let data = SkeletonData::from_json_str(
        r#"
{
  "skeleton": { "spine": "4.3.00" },
  "bones": [ { "name": "root" } ],
  "slots": [
    { "name": "slot0", "bone": "root", "attachment": "clip" },
    { "name": "slot1", "bone": "root", "attachment": "region0" }
  ],
  "skins": {
    "default": {
      "slot0": {
        "clip": {
          "type": "clipping",
          "inverse": true,
          "end": "slot1",
          "vertexCount": 4,
          "vertices": [-1,-1, 1,-1, 1,1, -1,1]
        }
      },
      "slot1": {
        "region0": { "type": "region", "path": "page.png", "width": 4, "height": 4 }
      }
    }
  },
  "animations": {}
}
"#,
    )
    .unwrap();
    let mut skeleton = crate::Skeleton::new(data);
    skeleton.setup_pose();
    skeleton.update_world_transform_with_physics(crate::Physics::None);

    let draw_list = build_draw_list(&skeleton);
    assert_eq!(draw_list.draws.len(), 1);
    assert_eq!(draw_list.draws[0].texture_path, "page.png");
    assert!(draw_list.vertices.len() > 4);

    let eps = 1.0e-4;
    assert!(
        draw_list
            .vertices
            .iter()
            .any(|v| { v.position[0].abs() > 1.0 + eps || v.position[1].abs() > 1.0 + eps }),
        "expected inverse clipping to keep vertices outside the clip polygon: {:?}",
        draw_list.vertices
    );
}
