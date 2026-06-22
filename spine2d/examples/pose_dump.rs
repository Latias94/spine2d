use serde_json::json;
use spine2d::{AnimationState, AnimationStateData, Skeleton, SkeletonData};
use std::path::PathBuf;
use std::sync::Arc;

fn load_skeleton_data(path: &PathBuf) -> Arc<SkeletonData> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("skel") {
        #[cfg(feature = "binary")]
        {
            let bytes = std::fs::read(path).expect("read skel");
            return SkeletonData::from_skel_bytes(&bytes).expect("parse skel");
        }
        #[cfg(not(feature = "binary"))]
        {
            panic!("Input is .skel but spine2d was built without feature `binary`.");
        }
    }

    let json = std::fs::read_to_string(path).expect("read json");
    SkeletonData::from_json_str(&json).expect("parse json")
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut positional = Vec::<String>::new();
    let mut dump_slot_vertices: Option<String> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--dump-slot-vertices" => {
                dump_slot_vertices = args.get(i + 1).cloned();
                i += 2;
            }
            other => {
                positional.push(other.to_string());
                i += 1;
            }
        }
    }

    let json_path = positional.first().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("./assets/spine-runtimes/examples/spineboy/export/spineboy-pro.json")
    });
    let animation = positional
        .get(1)
        .cloned()
        .unwrap_or_else(|| "run".to_string());
    let time: f32 = positional
        .get(2)
        .cloned()
        .unwrap_or_else(|| "0.5".to_string())
        .parse()
        .unwrap_or(0.5);

    let data: Arc<SkeletonData> = load_skeleton_data(&json_path);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data));

    state
        .set_animation(0, &animation, true)
        .expect("set animation");
    state.update(time.max(0.0));

    skeleton.set_to_setup_pose();
    state.apply(&mut skeleton);
    skeleton.update_world_transform();

    let bones: Vec<_> = skeleton
        .bones()
        .iter()
        .enumerate()
        .map(|(i, bone)| {
            let name = skeleton
                .data()
                .bones
                .get(i)
                .map(|b| b.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "active": if bone.active { 1 } else { 0 },
                "world": {"a": bone.a, "b": bone.b, "c": bone.c, "d": bone.d, "x": bone.world_x, "y": bone.world_y},
                "applied": {"x": bone.ax, "y": bone.ay, "rotation": bone.arotation, "scaleX": bone.ascale_x, "scaleY": bone.ascale_y, "shearX": bone.ashear_x, "shearY": bone.ashear_y},
            })
        })
        .collect();

    let slots: Vec<_> = skeleton
        .slots()
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let name = skeleton
                .data()
                .slots
                .get(i)
                .map(|s| s.name.as_str())
                .unwrap_or("<unknown>");
            let attachment = skeleton
                .slot_attachment_data(i)
                .map(|a| json!({"name": a.name()}));
            let has_dark = if slot.has_dark { 1 } else { 0 };
            let dark_color = if slot.has_dark {
                [
                    slot.dark_color[0],
                    slot.dark_color[1],
                    slot.dark_color[2],
                    1.0,
                ]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            json!({
                "i": i,
                "name": name,
                "color": slot.color,
                "hasDark": has_dark,
                "darkColor": dark_color,
                "attachment": attachment,
            })
        })
        .collect();

    let draw_order: Vec<_> = skeleton
        .draw_order()
        .iter()
        .copied()
        .map(|slot_index| slot_index as i32)
        .collect();

    let ik_constraints: Vec<_> = skeleton
        .ik_constraints()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data()
                .ik_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "mix": c.mix,
                "softness": c.softness,
                "bendDirection": c.bend_direction,
                "active": if c.active { 1 } else { 0 },
            })
        })
        .collect();

    let transform_constraints: Vec<_> = skeleton
        .transform_constraints()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data()
                .transform_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "mixRotate": c.mix_rotate,
                "mixX": c.mix_x,
                "mixY": c.mix_y,
                "mixScaleX": c.mix_scale_x,
                "mixScaleY": c.mix_scale_y,
                "mixShearY": c.mix_shear_y,
                "active": if c.active { 1 } else { 0 },
            })
        })
        .collect();

    let path_constraints: Vec<_> = skeleton
        .path_constraints()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let name = skeleton
                .data()
                .path_constraints
                .get(i)
                .map(|d| d.name.as_str())
                .unwrap_or("<unknown>");
            json!({
                "i": i,
                "name": name,
                "position": c.position,
                "spacing": c.spacing,
                "mixRotate": c.mix_rotate,
                "mixX": c.mix_x,
                "mixY": c.mix_y,
                "active": if c.active { 1 } else { 0 },
            })
        })
        .collect();

    let debug = dump_slot_vertices.as_deref().and_then(|slot_name| {
        let slot_index = skeleton
            .data()
            .slots
            .iter()
            .position(|s| s.name == slot_name)?;
        let world_vertices = skeleton.slot_vertex_attachment_world_vertices(slot_index)?;
        Some(json!({
            "slot": slot_name,
            "slotIndex": slot_index as i32,
            "worldVertices": world_vertices,
        }))
    });

    let out = json!({
        "mode": "legacy",
        "animation": animation,
        "time": time,
        "bones": bones,
        "slots": slots,
        "drawOrder": draw_order,
        "ikConstraints": ik_constraints,
        "transformConstraints": transform_constraints,
        "pathConstraints": path_constraints,
        "debug": debug,
    });

    println!("{}", serde_json::to_string(&out).expect("json"));
}
