use crate::runtime::{AnimationState, AnimationStateData};
use crate::{AttachmentData, RegionAttachmentData, Skeleton, SkeletonData, SkinData};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

fn upstream_examples_root() -> PathBuf {
    if let Ok(dir) = std::env::var("SPINE2D_UPSTREAM_EXAMPLES_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return p;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../assets/spine-runtimes/examples"),
        manifest_dir.join("../third_party/spine-runtimes/examples"),
        manifest_dir.join("../.cache/spine-runtimes/examples"),
    ];
    for p in candidates {
        if p.is_dir() {
            return p;
        }
    }

    panic!(
        "Upstream Spine examples not found. Run `./scripts/import_spine_runtimes_examples.zsh --mode json` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn example_json_path(relative: &str) -> PathBuf {
    upstream_examples_root().join(relative)
}

fn bone_index(data: &SkeletonData, name: &str) -> usize {
    data.bones
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| panic!("missing bone: {name}"))
}

fn slot_index(data: &SkeletonData, name: &str) -> usize {
    data.slots
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("missing slot: {name}"))
}

fn transform_constraint_index(data: &SkeletonData, name: &str) -> usize {
    data.transform_constraints
        .iter()
        .position(|c| c.name == name)
        .unwrap_or_else(|| panic!("missing transform constraint: {name}"))
}

fn assert_approx(actual: f32, expected: f32) {
    let eps = 1.0e-6;
    let diff = (actual - expected).abs();
    assert!(
        diff <= eps,
        "expected {expected}, got {actual} (diff {diff}, eps {eps})"
    );
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct AttachmentSig {
    kind: &'static str,
    name: String,
    path: Option<String>,
}

fn attachment_sig(a: &AttachmentData) -> AttachmentSig {
    match a {
        AttachmentData::Region(r) => AttachmentSig {
            kind: "region",
            name: r.name.clone(),
            path: Some(r.path.clone()),
        },
        AttachmentData::Mesh(m) => AttachmentSig {
            kind: "mesh",
            name: m.name.clone(),
            path: Some(m.path.clone()),
        },
        AttachmentData::Point(p) => AttachmentSig {
            kind: "point",
            name: p.name.clone(),
            path: None,
        },
        AttachmentData::Path(p) => AttachmentSig {
            kind: "path",
            name: p.name.clone(),
            path: None,
        },
        AttachmentData::BoundingBox(b) => AttachmentSig {
            kind: "bounding_box",
            name: b.name.clone(),
            path: None,
        },
        AttachmentData::Clipping(c) => AttachmentSig {
            kind: "clipping",
            name: c.name.clone(),
            path: None,
        },
    }
}

fn region_attachment(name: &str) -> AttachmentData {
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
    })
}

#[test]
fn skin_required_active_and_gating_match_spine_cpp_semantics() {
    let path = example_json_path("mix-and-match/export/mix-and-match-pro.json");
    let json = std::fs::read_to_string(&path).expect("read mix-and-match-pro.json");
    let data = SkeletonData::from_json_str(&json).expect("parse mix-and-match-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    // Start from no skin, then set a skin. Upstream applies setup attachments from the new skin.
    skeleton.set_skin(Some("accessories/backpack"));

    let backpack_bone = bone_index(&data, "backpack");
    assert!(skeleton.bones[backpack_bone].active);
    let hat_control_bone = bone_index(&data, "hat-control");
    assert!(!skeleton.bones[hat_control_bone].active);

    let hat_control_constraint = transform_constraint_index(&data, "hat-control");
    assert!(!skeleton.transform_constraints[hat_control_constraint].active);

    let backpack_slot = slot_index(&data, "backpack");
    let key = skeleton.slots[backpack_slot]
        .attachment
        .as_deref()
        .expect("backpack setup attachment should be applied from skin");
    assert_eq!(key, "backpack");
    let resolved = skeleton.slots[backpack_slot]
        .get_attachment(&skeleton)
        .expect("resolve backpack attachment");
    assert_eq!(resolved.name(), "boy/backpack");

    // Bone timeline gating: `aware` anim drives `hat-control.translate` but the bone is inactive
    // under this skin, so its local transform must remain at setup values.
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    state.set_animation(0, "aware", true);
    state.update(0.1667);
    state.apply(&mut skeleton);

    let setup = &data.bones[hat_control_bone];
    let bone = &skeleton.bones[hat_control_bone];
    assert_approx(bone.x, setup.x);
    assert_approx(bone.y, setup.y);
}

#[test]
fn mix_and_match_add_skin_composition_matches_upstream_demo_semantics() {
    // Based on `spine-libgdx` `MixAndMatchTest.java`:
    // it builds a custom skin by unioning multiple item skins.
    let path = example_json_path("mix-and-match/export/mix-and-match-pro.json");
    let json = std::fs::read_to_string(&path).expect("read mix-and-match-pro.json");
    let data = SkeletonData::from_json_str(&json).expect("parse mix-and-match-pro.json");
    let mut data = (*data).clone();

    let component_skins = [
        "skin-base",
        "nose/short",
        "eyelids/girly",
        "eyes/violet",
        "hair/brown",
        "clothes/hoodie-orange",
        "legs/pants-jeans",
        "accessories/bag",
        "accessories/hat-red-yellow",
    ];

    let mut expected_bones = Vec::new();
    let mut expected_ik = Vec::new();
    let mut expected_transform = Vec::new();
    let mut expected_path = Vec::new();
    let mut expected_physics = Vec::new();
    let mut expected_slider = Vec::new();

    let mut expected_attachments: HashMap<(usize, String), AttachmentSig> = HashMap::new();

    let mut custom = SkinData::new("custom-girl");
    custom
        .attachments
        .resize_with(data.slots.len(), Default::default);
    for skin_name in component_skins {
        let skin = data
            .find_skin(skin_name)
            .unwrap_or_else(|| panic!("missing skin: {skin_name}"));

        for &i in &skin.bones {
            if !expected_bones.contains(&i) {
                expected_bones.push(i);
            }
        }
        for &i in &skin.ik_constraints {
            if !expected_ik.contains(&i) {
                expected_ik.push(i);
            }
        }
        for &i in &skin.transform_constraints {
            if !expected_transform.contains(&i) {
                expected_transform.push(i);
            }
        }
        for &i in &skin.path_constraints {
            if !expected_path.contains(&i) {
                expected_path.push(i);
            }
        }
        for &i in &skin.physics_constraints {
            if !expected_physics.contains(&i) {
                expected_physics.push(i);
            }
        }
        for &i in &skin.slider_constraints {
            if !expected_slider.contains(&i) {
                expected_slider.push(i);
            }
        }

        for (slot_index, slot_map) in skin.attachments.iter().enumerate() {
            for (key, attachment) in slot_map {
                expected_attachments.insert((slot_index, key.clone()), attachment_sig(attachment));
            }
        }

        custom.add_skin(skin);
    }

    assert_eq!(custom.bones, expected_bones);
    assert_eq!(custom.ik_constraints, expected_ik);
    assert_eq!(custom.transform_constraints, expected_transform);
    assert_eq!(custom.path_constraints, expected_path);
    assert_eq!(custom.physics_constraints, expected_physics);
    assert_eq!(custom.slider_constraints, expected_slider);

    let expected_keys: HashSet<(usize, String)> = expected_attachments.keys().cloned().collect();
    let mut actual_keys = HashSet::new();
    for (slot_index, slot_map) in custom.attachments.iter().enumerate() {
        for key in slot_map.keys() {
            actual_keys.insert((slot_index, key.clone()));
        }
    }
    assert_eq!(actual_keys, expected_keys);

    for ((slot_index, key), expected_sig) in expected_attachments {
        let Some(actual) = custom.get_attachment(slot_index, &key) else {
            panic!("custom skin missing attachment: slot={slot_index}, key={key}");
        };
        assert_eq!(
            attachment_sig(actual),
            expected_sig,
            "attachment mismatch: slot={slot_index}, key={key}"
        );
    }

    // Also verify `Skeleton::set_skin` correctly activates bones and applies setup attachments
    // from the runtime-composed skin.
    data.skins.push(custom.clone());
    let data = std::sync::Arc::new(data);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.set_skin(Some(custom.name.as_str()));

    let hat_control_bone = bone_index(&data, "hat-control");
    assert!(
        skeleton.bones[hat_control_bone].active,
        "hat-control should be active when the custom skin includes hat bones"
    );

    let some_setup_attachment_slot = data
        .slots
        .iter()
        .enumerate()
        .find_map(|(i, s)| {
            let setup = s.attachment.as_deref()?;
            if custom.get_attachment(i, setup).is_some() {
                Some((i, setup.to_string()))
            } else {
                None
            }
        })
        .expect("expected at least one setup attachment to exist in the custom skin");

    let (slot_index, setup_key) = some_setup_attachment_slot;
    assert_eq!(
        skeleton.slots[slot_index].attachment.as_deref(),
        Some(setup_key.as_str())
    );
    assert_eq!(
        skeleton.slots[slot_index].attachment_skin.as_deref(),
        Some("custom-girl")
    );
}

#[test]
fn set_skin_from_skin_to_skin_replaces_shared_attachments_and_preserves_missing_ones() {
    let path = example_json_path("mix-and-match/export/mix-and-match-pro.json");
    let json = std::fs::read_to_string(&path).expect("read mix-and-match-pro.json");
    let data = SkeletonData::from_json_str(&json).expect("parse mix-and-match-pro.json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();
    skeleton.set_skin(Some("full-skins/boy"));

    let mouth_slot = slot_index(&data, "mouth");
    let zip_slot = slot_index(&data, "zip-boy");

    assert_eq!(
        skeleton.slots[mouth_slot].attachment.as_deref(),
        Some("mouth-smile")
    );
    assert_eq!(
        skeleton.slots[mouth_slot].attachment_skin.as_deref(),
        Some("full-skins/boy")
    );
    assert_eq!(
        skeleton.slots[zip_slot].attachment.as_deref(),
        Some("zip-boy")
    );
    assert_eq!(
        skeleton.slots[zip_slot].attachment_skin.as_deref(),
        Some("full-skins/boy")
    );

    skeleton.set_skin(Some("full-skins/girl"));

    assert_eq!(
        skeleton.slots[mouth_slot].attachment.as_deref(),
        Some("mouth-smile")
    );
    assert_eq!(
        skeleton.slots[mouth_slot].attachment_skin.as_deref(),
        Some("full-skins/girl")
    );
    assert_eq!(
        skeleton.slots[zip_slot].attachment.as_deref(),
        Some("zip-boy")
    );
    assert_eq!(
        skeleton.slots[zip_slot].attachment_skin.as_deref(),
        Some("full-skins/boy")
    );
}

#[test]
fn add_skin_is_idempotent_for_lists_and_last_write_wins_for_attachments() {
    let mut base = SkinData::new("base");
    base.attachments.resize_with(2, Default::default);
    base.bones = vec![1, 2];
    base.ik_constraints = vec![3];
    base.transform_constraints = vec![4];
    base.path_constraints = vec![5];
    base.physics_constraints = vec![6];
    base.slider_constraints = vec![7];
    base.attachments[0].insert(
        "key".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "base".to_string(),
            path: "base.png".to_string(),
            sequence: None,
            color: [1.0, 0.0, 0.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 0.0,
            height: 0.0,
        }),
    );

    let mut overlay = SkinData::new("overlay");
    overlay.attachments.resize_with(2, Default::default);
    overlay.bones = vec![2, 3];
    overlay.ik_constraints = vec![3, 8];
    overlay.transform_constraints = vec![4, 9];
    overlay.path_constraints = vec![5, 10];
    overlay.physics_constraints = vec![6, 11];
    overlay.slider_constraints = vec![7, 12];
    overlay.attachments[0].insert(
        "key".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "overlay".to_string(),
            path: "overlay.png".to_string(),
            sequence: None,
            color: [0.0, 1.0, 0.0, 1.0],
            x: 1.0,
            y: 2.0,
            rotation: 3.0,
            scale_x: 0.5,
            scale_y: 0.75,
            width: 4.0,
            height: 5.0,
        }),
    );
    overlay.attachments[1].insert(
        "other".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "other".to_string(),
            path: "other.png".to_string(),
            sequence: None,
            color: [0.0, 0.0, 1.0, 1.0],
            x: 6.0,
            y: 7.0,
            rotation: 8.0,
            scale_x: 1.5,
            scale_y: 2.0,
            width: 9.0,
            height: 10.0,
        }),
    );

    base.add_skin(&overlay);
    base.add_skin(&overlay);

    assert_eq!(base.bones, vec![1, 2, 3]);
    assert_eq!(base.ik_constraints, vec![3, 8]);
    assert_eq!(base.transform_constraints, vec![4, 9]);
    assert_eq!(base.path_constraints, vec![5, 10]);
    assert_eq!(base.physics_constraints, vec![6, 11]);
    assert_eq!(base.slider_constraints, vec![7, 12]);

    let key = base.get_attachment(0, "key").expect("merged attachment");
    let AttachmentData::Region(region) = key else {
        panic!("expected region attachment");
    };
    assert_eq!(region.name, "overlay");
    assert_eq!(region.path, "overlay.png");
    assert_eq!(region.x, 1.0);
    assert_eq!(region.y, 2.0);
    assert_eq!(region.rotation, 3.0);
    assert_eq!(region.scale_x, 0.5);
    assert_eq!(region.scale_y, 0.75);

    let other = base
        .get_attachment(1, "other")
        .expect("second slot attachment");
    let AttachmentData::Region(region) = other else {
        panic!("expected region attachment");
    };
    assert_eq!(region.name, "other");
    assert_eq!(region.path, "other.png");
    assert_eq!(region.x, 6.0);
    assert_eq!(region.y, 7.0);
    assert_eq!(region.rotation, 8.0);
    assert_eq!(region.scale_x, 1.5);
    assert_eq!(region.scale_y, 2.0);
}

#[test]
fn copy_skin_matches_cpp_deep_copy_surface() {
    let mut source = SkinData::new("source");
    source.attachments.resize_with(1, Default::default);
    source.bones = vec![1];
    source.ik_constraints = vec![2];
    source.transform_constraints = vec![3];
    source.path_constraints = vec![4];
    source.physics_constraints = vec![5];
    source.slider_constraints = vec![6];
    source.set_attachment(0, "key", region_attachment("copied"));

    let mut target = SkinData::new("target");
    target.attachments.resize_with(1, Default::default);
    target.copy_skin(&source);

    assert_eq!(target.bones, vec![1]);
    assert_eq!(target.ik_constraints, vec![2]);
    assert_eq!(target.transform_constraints, vec![3]);
    assert_eq!(target.path_constraints, vec![4]);
    assert_eq!(target.physics_constraints, vec![5]);
    assert_eq!(target.slider_constraints, vec![6]);

    source.set_attachment(0, "key", region_attachment("replaced-source"));

    let AttachmentData::Region(region) = target.get_attachment(0, "key").unwrap() else {
        panic!("expected copied region attachment");
    };
    assert_eq!(region.name, "copied");
    assert_eq!(region.path, "copied.png");
}

#[test]
fn skin_attachment_iteration_preserves_insertion_order() {
    let mut skin = SkinData::new("ordered");
    skin.attachments.resize_with(1, Default::default);
    skin.set_attachment(
        0,
        "first".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "first".to_string(),
            path: "first.png".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 0.0,
            height: 0.0,
        }),
    );
    skin.set_attachment(
        0,
        "second".to_string(),
        AttachmentData::Region(RegionAttachmentData {
            name: "second".to_string(),
            path: "second.png".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 0.0,
            height: 0.0,
        }),
    );

    let slot0 = &skin.attachments[0];
    assert_eq!(
        slot0.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    assert_eq!(slot0.len(), 2);
    assert_eq!(slot0.get_index(0).unwrap().0.as_str(), "first");
    assert_eq!(slot0.get_index(1).unwrap().0.as_str(), "second");
}

#[test]
fn skin_find_slot_queries_append_like_cpp() {
    let mut skin = SkinData::new("ordered");
    skin.set_attachment(0, "first", region_attachment("first"));
    skin.set_attachment(0, "second", region_attachment("second"));
    skin.set_attachment(2, "other-slot", region_attachment("other-slot"));

    let mut names = vec!["sentinel".to_string()];
    skin.find_names_for_slot(0, &mut names);
    assert_eq!(names, vec!["sentinel", "first", "second"]);

    skin.find_names_for_slot(1, &mut names);
    assert_eq!(names, vec!["sentinel", "first", "second"]);

    let mut attachments = Vec::new();
    skin.find_attachments_for_slot(0, &mut attachments);
    assert_eq!(
        attachments
            .iter()
            .map(|attachment| attachment.name())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );

    skin.find_attachments_for_slot(1, &mut attachments);
    assert_eq!(
        attachments
            .iter()
            .map(|attachment| attachment.name())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
}

#[test]
fn skin_set_attachment_grows_slot_storage_and_remove_is_noop_on_missing_entries() {
    let mut skin = SkinData::new("growing");
    skin.set_attachment(
        2,
        "grown",
        AttachmentData::Region(RegionAttachmentData {
            name: "grown".to_string(),
            path: "grown.png".to_string(),
            sequence: None,
            color: [1.0, 1.0, 1.0, 1.0],
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            width: 0.0,
            height: 0.0,
        }),
    );
    assert_eq!(skin.attachments.len(), 3);
    assert_eq!(
        skin.attachments[2]
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["grown"]
    );
    assert_eq!(skin.attachments[2].len(), 1);

    skin.remove_attachment(2, "missing");
    assert_eq!(
        skin.attachments[2]
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["grown"]
    );
    skin.remove_attachment(2, "grown");
    assert!(skin.attachments[2].is_empty());
}
