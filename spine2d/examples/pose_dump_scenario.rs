use serde_json::json;
use spine2d::{
    AnimationState, AnimationStateData, Curve, Physics, Skeleton, SkeletonData, TrackEntryHandle,
};
use std::path::PathBuf;
use std::sync::Arc;

fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage:\n  pose_dump_scenario <skeleton.(json|skel)> <commands...>\n\nCommands:\n  --set-skin <name|none>\n  --dump-slot-vertices <slotName>\n  --dump-update-cache\n  --dump-animation-data <name>\n  --mix <from> <to> <duration>\n  --set <track> <animation> <loop 0|1>\n  --add <track> <animation> <loop 0|1> <delay>\n  --set-empty <track> <mixDuration>\n  --add-empty <track> <mixDuration> <delay>\n  --entry-alpha <alpha>\n  --entry-mix-attachment-threshold <threshold>\n  --entry-mix-draw-order-threshold <threshold>\n  --entry-additive <0|1>\n  --entry-reverse <0|1>\n  --entry-shortest-rotation <0|1>\n  --entry-reset-rotation-directions\n  --physics <none|reset|update|pose>\n  --step <dt>\n"
    );
    std::process::exit(2);
}

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

fn parse_physics(s: &str) -> Option<Physics> {
    match s {
        "none" => Some(Physics::None),
        "reset" => Some(Physics::Reset),
        "update" => Some(Physics::Update),
        "pose" => Some(Physics::Pose),
        _ => None,
    }
}

fn bone_timeline_info(
    data: &SkeletonData,
    index: usize,
    timeline: &spine2d::BoneTimeline,
) -> serde_json::Value {
    let (kind, bone_index, frames) = match timeline {
        spine2d::BoneTimeline::Rotate(t) => (
            "Rotate",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "angle": frame.angle,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::Translate(t) => (
            "Translate",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "x": frame.x,
                            "y": frame.y,
                            "curve": [
                                curve_info(&frame.curve[0]),
                                curve_info(&frame.curve[1]),
                            ],
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::TranslateX(t) => (
            "TranslateX",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "value": frame.value,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::TranslateY(t) => (
            "TranslateY",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "value": frame.value,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::Scale(t) => (
            "Scale",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "x": frame.x,
                            "y": frame.y,
                            "curve": [
                                curve_info(&frame.curve[0]),
                                curve_info(&frame.curve[1]),
                            ],
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::ScaleX(t) => (
            "ScaleX",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "value": frame.value,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::ScaleY(t) => (
            "ScaleY",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "value": frame.value,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::Shear(t) => (
            "Shear",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "x": frame.x,
                            "y": frame.y,
                            "curve": [
                                curve_info(&frame.curve[0]),
                                curve_info(&frame.curve[1]),
                            ],
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::ShearX(t) => (
            "ShearX",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "value": frame.value,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::ShearY(t) => (
            "ShearY",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "value": frame.value,
                            "curve": curve_info(&frame.curve),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
        spine2d::BoneTimeline::Inherit(t) => (
            "Inherit",
            t.bone_index,
            json!(
                t.frames
                    .iter()
                    .map(|frame| {
                        json!({
                            "time": frame.time,
                            "inherit": format!("{:?}", frame.inherit),
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        ),
    };
    json!({
        "index": index,
        "kind": kind,
        "boneIndex": bone_index,
        "boneName": data.bones.get(bone_index).map(|b| b.name.as_str()).unwrap_or("<unknown>"),
        "frames": frames,
    })
}

fn curve_info(curve: &Curve) -> serde_json::Value {
    match curve {
        Curve::Linear => json!({"kind": "Linear"}),
        Curve::Stepped => json!({"kind": "Stepped"}),
        Curve::Bezier { cx1, cy1, cx2, cy2 } => json!({
            "kind": "Bezier",
            "cx1": cx1,
            "cy1": cy1,
            "cx2": cx2,
            "cy2": cy2,
        }),
    }
}

fn dump_animation_data(data: &SkeletonData, name: &str) {
    let Some((index, animation)) = data.animation(name) else {
        panic!("missing animation: {name}");
    };
    let ik_timelines: Vec<_> = animation
        .ik_constraint_timelines
        .iter()
        .map(|timeline| {
            let constraint_name = data
                .ik_constraints
                .get(timeline.constraint_index)
                .map(|c| c.name.as_str())
                .unwrap_or("<unknown>");
            let frames: Vec<_> = timeline
                .frames
                .iter()
                .map(|frame| {
                    json!({
                        "time": frame.time,
                        "mix": frame.mix,
                        "softness": frame.softness,
                        "bendDirection": frame.bend_direction,
                        "compress": frame.compress,
                        "stretch": frame.stretch,
                        "curve": [
                            curve_info(&frame.curve[0]),
                            curve_info(&frame.curve[1]),
                        ],
                    })
                })
                .collect();
            json!({
                "constraintIndex": timeline.constraint_index,
                "constraintName": constraint_name,
                "frames": frames,
            })
        })
        .collect();
    let bone_timelines: Vec<_> = animation
        .bone_timelines
        .iter()
        .enumerate()
        .map(|(i, timeline)| bone_timeline_info(data, i, timeline))
        .collect();
    let out = json!({
        "index": index,
        "name": animation.name,
        "duration": animation.duration,
        "timelineOrder": animation.timeline_order.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        "ikTimelines": ik_timelines,
        "boneTimelines": bone_timelines,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).expect("animation data json")
    );
}

fn debug_dump_bones(label: &str, skeleton: &Skeleton, total_time: f32) {
    let Ok(filter) = std::env::var("SPINE2D_DEBUG_BONES") else {
        return;
    };
    let names: Vec<&str> = filter
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    if names.is_empty() {
        return;
    }

    eprintln!("[DEBUG-runwalk] {label} t={total_time:.6}");
    for name in names {
        let Some((i, bone)) = skeleton.bones().iter().enumerate().find(|(i, _)| {
            skeleton
                .data()
                .bones
                .get(*i)
                .is_some_and(|b| b.name == name)
        }) else {
            eprintln!("[DEBUG-runwalk] missing bone {name}");
            continue;
        };
        eprintln!(
            "[DEBUG-runwalk] bone[{i}] {name} pose x={:.9} y={:.9} rot={:.9} sx={:.9} sy={:.9} shx={:.9} shy={:.9} world a={:.9} b={:.9} c={:.9} d={:.9} x={:.9} y={:.9} applied x={:.9} y={:.9} rot={:.9} sx={:.9} sy={:.9} shx={:.9} shy={:.9} world a={:.9} b={:.9} c={:.9} d={:.9} x={:.9} y={:.9}",
            bone.x(),
            bone.y(),
            bone.rotation(),
            bone.scale_x(),
            bone.scale_y(),
            bone.shear_x(),
            bone.shear_y(),
            bone.a(),
            bone.b(),
            bone.c(),
            bone.d(),
            bone.world_x(),
            bone.world_y(),
            bone.applied_x(),
            bone.applied_y(),
            bone.applied_rotation(),
            bone.applied_scale_x(),
            bone.applied_scale_y(),
            bone.applied_shear_x(),
            bone.applied_shear_y(),
            bone.a(),
            bone.b(),
            bone.c(),
            bone.d(),
            bone.world_x(),
            bone.world_y(),
        );
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_usage_and_exit();
    }

    let json_path = PathBuf::from(args.remove(0));
    let mut dump_slot_vertices: Option<String> = None;
    let mut dump_update_cache: bool = false;
    let mut dump_animation_data_name: Option<String> = None;
    let data: Arc<SkeletonData> = load_skeleton_data(&json_path);

    let mut skeleton = Skeleton::new(data.clone());
    let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
    let mut last_entry: Option<TrackEntryHandle> = None;
    let mut total_time = 0.0f32;
    let mut physics: Physics = Physics::None;

    // Setup pose once; scenario steps do not reset the skeleton each frame.
    skeleton.set_to_setup_pose();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dump-slot-vertices" if i + 1 < args.len() => {
                dump_slot_vertices = args.get(i + 1).cloned();
                i += 2;
            }
            "--dump-update-cache" => {
                dump_update_cache = true;
                i += 1;
            }
            "--dump-animation-data" if i + 1 < args.len() => {
                dump_animation_data_name = args.get(i + 1).cloned();
                i += 2;
            }
            "--set-skin" if i + 1 < args.len() => {
                let name = args[i + 1].as_str();
                if name == "none" {
                    skeleton.set_skin(None).expect("set skin");
                } else {
                    skeleton.set_skin(Some(name)).expect("set skin");
                }
                i += 2;
            }
            "--mix" if i + 3 < args.len() => {
                let from = args[i + 1].as_str();
                let to = args[i + 2].as_str();
                let duration: f32 = args[i + 3].parse().unwrap();
                state
                    .data_mut()
                    .set_mix(from, to, duration)
                    .expect("set mix");
                i += 4;
            }
            "--set" if i + 3 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let anim = args[i + 2].as_str();
                let looped: bool = args[i + 3].parse::<i32>().unwrap_or(0) != 0;
                last_entry = Some(
                    state
                        .set_animation(track, anim, looped)
                        .expect("set animation"),
                );
                i += 4;
            }
            "--add" if i + 4 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let anim = args[i + 2].as_str();
                let looped: bool = args[i + 3].parse::<i32>().unwrap_or(0) != 0;
                let delay: f32 = args[i + 4].parse().unwrap();
                last_entry = Some(
                    state
                        .add_animation(track, anim, looped, delay)
                        .expect("add animation"),
                );
                i += 5;
            }
            "--set-empty" if i + 2 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let mix_duration: f32 = args[i + 2].parse().unwrap();
                last_entry = Some(
                    state
                        .set_empty_animation(track, mix_duration)
                        .expect("set empty animation"),
                );
                i += 3;
            }
            "--add-empty" if i + 3 < args.len() => {
                let track: usize = args[i + 1].parse().unwrap();
                let mix_duration: f32 = args[i + 2].parse().unwrap();
                let delay: f32 = args[i + 3].parse().unwrap();
                last_entry = Some(
                    state
                        .add_empty_animation(track, mix_duration, delay)
                        .expect("add empty animation"),
                );
                i += 4;
            }
            "--entry-alpha" if i + 1 < args.len() => {
                let alpha: f32 = args[i + 1].parse().unwrap();
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| panic!("--entry-alpha requires a preceding --set/--add"))
                    .set_alpha(&mut state, alpha);
                i += 2;
            }
            "--entry-mix-attachment-threshold" if i + 1 < args.len() => {
                let threshold: f32 = args[i + 1].parse().unwrap();
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-mix-attachment-threshold requires a preceding --set/--add")
                    })
                    .set_mix_attachment_threshold(&mut state, threshold);
                i += 2;
            }
            "--entry-mix-draw-order-threshold" if i + 1 < args.len() => {
                let threshold: f32 = args[i + 1].parse().unwrap();
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-mix-draw-order-threshold requires a preceding --set/--add")
                    })
                    .set_mix_draw_order_threshold(&mut state, threshold);
                i += 2;
            }
            "--entry-additive" if i + 1 < args.len() => {
                let additive: bool = args[i + 1].parse::<i32>().unwrap_or(0) != 0;
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| panic!("--entry-additive requires a preceding --set/--add"))
                    .set_additive(&mut state, additive);
                i += 2;
            }
            "--entry-reverse" if i + 1 < args.len() => {
                let reverse: bool = args[i + 1].parse::<i32>().unwrap_or(0) != 0;
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| panic!("--entry-reverse requires a preceding --set/--add"))
                    .set_reverse(&mut state, reverse);
                i += 2;
            }
            "--entry-shortest-rotation" if i + 1 < args.len() => {
                let shortest_rotation: bool = args[i + 1].parse::<i32>().unwrap_or(0) != 0;
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-shortest-rotation requires a preceding --set/--add")
                    })
                    .set_shortest_rotation(&mut state, shortest_rotation);
                i += 2;
            }
            "--entry-reset-rotation-directions" => {
                last_entry
                    .as_ref()
                    .unwrap_or_else(|| {
                        panic!("--entry-reset-rotation-directions requires a preceding --set/--add")
                    })
                    .reset_rotation_directions(&mut state);
                i += 1;
            }
            "--physics" if i + 1 < args.len() => {
                let next = args[i + 1].as_str();
                physics =
                    parse_physics(next).unwrap_or_else(|| panic!("invalid physics mode: {next}"));
                i += 2;
            }
            "--step" if i + 1 < args.len() => {
                let dt: f32 = args[i + 1].parse().unwrap();
                state.update(dt);
                state.apply(&mut skeleton);
                debug_dump_bones("after-apply-before-world", &skeleton, total_time + dt);
                skeleton.update(dt);
                skeleton.update_world_transform_with_physics(physics);
                debug_dump_bones("after-world", &skeleton, total_time + dt);
                total_time += dt;
                i += 2;
            }
            _ => {
                print_usage_and_exit();
            }
        }
    }

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
                "active": if bone.is_active() { 1 } else { 0 },
                "world": {"a": bone.a(), "b": bone.b(), "c": bone.c(), "d": bone.d(), "x": bone.world_x(), "y": bone.world_y()},
                "applied": {"x": bone.applied_x(), "y": bone.applied_y(), "rotation": bone.applied_rotation(), "scaleX": bone.applied_scale_x(), "scaleY": bone.applied_scale_y(), "shearX": bone.applied_shear_x(), "shearY": bone.applied_shear_y()},
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

    let mut debug_map = serde_json::Map::new();
    if dump_update_cache {
        debug_map.insert(
            "updateCache".to_string(),
            json!(skeleton.debug_update_cache()),
        );
        let transform_constraint_data: Vec<_> = skeleton
            .data()
            .transform_constraints
            .iter()
            .map(|c| {
                let bone_names: Vec<_> = c
                    .bones
                    .iter()
                    .filter_map(|&i| skeleton.data().bones.get(i).map(|b| b.name.as_str()))
                    .collect();
                let source_name = skeleton
                    .data()
                    .bones
                    .get(c.source)
                    .map(|b| b.name.as_str())
                    .unwrap_or("<unknown>");
                json!({
                    "name": c.name,
                    "bones": c.bones.len(),
                    "boneNames": bone_names,
                    "source": source_name,
                    "properties": c.properties.len(),
                    "mixX": c.mix_x,
                    "mixY": c.mix_y,
                    "localSource": c.local_source,
                    "localTarget": c.local_target,
                    "additive": c.additive,
                    "clamp": c.clamp,
                })
            })
            .collect();
        debug_map.insert(
            "transformConstraintData".to_string(),
            json!(transform_constraint_data),
        );
    }
    if let Some(slot_name) = dump_slot_vertices.as_deref()
        && let Some(slot_index) = skeleton
            .data()
            .slots
            .iter()
            .position(|s| s.name == slot_name)
        && let Some(world_vertices) = skeleton.slot_vertex_attachment_world_vertices(slot_index)
    {
        debug_map.insert("slot".to_string(), json!(slot_name));
        debug_map.insert("slotIndex".to_string(), json!(slot_index as i32));
        debug_map.insert("worldVertices".to_string(), json!(world_vertices));
    }

    if let Some(animation_name) = dump_animation_data_name.as_deref() {
        dump_animation_data(&data, animation_name);
        return;
    }

    let debug = if debug_map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(debug_map))
    };

    let out = json!({
        "mode": "scenario",
        "time": total_time,
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
