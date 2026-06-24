use spine2d::{
    AnimationState, AnimationStateData, Atlas, Bone, MixInterpolation, Physics, Skeleton,
    SkeletonData, TrackEntryHandle,
};
use std::{collections::HashMap, env, fs, path::Path, sync::Arc};

fn usage() -> ! {
    eprintln!(
        "Usage:\n  render_dump <atlas.atlas> <skeleton.(json|skel)> [--y-down 0|1] <commands...>\n\nCommands:\n  --set-skin <name|none>\n  --physics <none|reset|update|pose>\n  --mix <from> <to> <duration>\n  --set <track> <animation> <loop 0|1>\n  --add <track> <animation> <loop 0|1> <delay>\n  --set-empty <track> <mixDuration>\n  --add-empty <track> <mixDuration> <delay>\n  --entry-alpha <alpha>\n  --entry-event-threshold <threshold>\n  --entry-alpha-attachment-threshold <threshold>\n  --entry-mix-attachment-threshold <threshold>\n  --entry-mix-draw-order-threshold <threshold>\n  --entry-additive <0|1>\n  --entry-mix-interpolation <linear|smooth|slow-fast|fast-slow|circle>\n  --entry-reverse <0|1>\n  --entry-shortest-rotation <0|1>\n  --entry-reset-rotation-directions\n  --step <dt>\n"
    );
    std::process::exit(2);
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))
}

#[cfg(feature = "binary")]
fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))
}

fn load_skeleton_data(path: &Path) -> Result<Arc<SkeletonData>, String> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("skel") {
        #[cfg(feature = "binary")]
        {
            let bytes = read_bytes(path)?;
            return SkeletonData::from_skel_bytes(&bytes)
                .map_err(|e| format!("failed to parse {}: {e}", path.display()));
        }
        #[cfg(not(feature = "binary"))]
        {
            return Err("loading .skel requires `--features binary`".to_string());
        }
    }

    #[cfg(feature = "json")]
    {
        let json = read_to_string(path)?;
        SkeletonData::from_json_str(&json)
            .map_err(|e| format!("failed to parse {}: {e}", path.display()))
    }
    #[cfg(not(feature = "json"))]
    {
        let _ = path;
        Err("loading .json requires `--features json`".to_string())
    }
}

fn parse_physics(s: &str) -> Result<Physics, String> {
    match s {
        "none" => Ok(Physics::None),
        "reset" => Ok(Physics::Reset),
        "update" => Ok(Physics::Update),
        "pose" => Ok(Physics::Pose),
        _ => Err(format!("invalid --physics {s}")),
    }
}

fn physics_name(p: Physics) -> &'static str {
    match p {
        Physics::None => "none",
        Physics::Reset => "reset",
        Physics::Update => "update",
        Physics::Pose => "pose",
    }
}

fn parse_mix_interpolation(s: &str) -> Result<MixInterpolation, String> {
    match s {
        "linear" => Ok(MixInterpolation::Linear),
        "smooth" => Ok(MixInterpolation::Smooth),
        "slow-fast" => Ok(MixInterpolation::SlowFast),
        "fast-slow" => Ok(MixInterpolation::FastSlow),
        "circle" => Ok(MixInterpolation::Circle),
        _ => Err(format!("invalid --entry-mix-interpolation {s}")),
    }
}

fn parse_bool_flag(s: &str) -> bool {
    s.parse::<i32>().unwrap_or(0) != 0
}

fn require_last_entry(last_entry: Option<TrackEntryHandle>, command: &str) -> TrackEntryHandle {
    last_entry.unwrap_or_else(|| {
        eprintln!("{command} requires a preceding --set/--add command");
        std::process::exit(2);
    })
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn clamp_u8_from_f32(v: f32) -> u8 {
    if !v.is_finite() {
        return 0;
    }
    let x = (v.clamp(0.0, 1.0) * 255.0) as i32;
    x.clamp(0, 255) as u8
}

fn pack_aarrggbb(rgba: [f32; 4]) -> u32 {
    let r = clamp_u8_from_f32(rgba[0]) as u32;
    let g = clamp_u8_from_f32(rgba[1]) as u32;
    let b = clamp_u8_from_f32(rgba[2]) as u32;
    let a = clamp_u8_from_f32(rgba[3]) as u32;
    (a << 24) | (r << 16) | (g << 8) | b
}

fn atlas_uses_pma(atlas: &Atlas) -> bool {
    atlas.pages.iter().any(|page| page.pma)
}

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 3 {
        usage();
    }

    let atlas_path = Path::new(&args[0]).to_path_buf();
    let skeleton_path = Path::new(&args[1]).to_path_buf();
    args.drain(0..2);

    let mut y_down = false;
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--y-down" {
            if i + 1 >= args.len() {
                usage();
            }
            y_down = parse_bool_flag(args[i + 1].as_str());
            i += 2;
        } else {
            i += 1;
        }
    }
    Bone::set_y_down(y_down);

    let mut skin: Option<String> = None;
    let mut physics = Physics::None;
    let mut total_time = 0.0f32;
    let mut last_entry: Option<TrackEntryHandle> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--y-down" if i + 1 < args.len() => {
                i += 2;
            }
            _ => break,
        }
    }

    let atlas_text = read_to_string(&atlas_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    let atlas = Atlas::parse(&atlas_text).unwrap_or_else(|e| {
        eprintln!("failed to parse {}: {e}", atlas_path.display());
        std::process::exit(2);
    });

    let data = load_skeleton_data(&skeleton_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    let mut state = AnimationState::new(AnimationStateData::new(data));

    while i < args.len() {
        match args[i].as_str() {
            "--set-skin" if i + 1 < args.len() => {
                let name = args[i + 1].as_str();
                skin = if name == "none" {
                    None
                } else {
                    Some(name.to_string())
                };
                skeleton.set_skin(skin.as_deref());
                skeleton.setup_pose_slots();
                skeleton.update_cache();
                i += 2;
            }
            "--physics" if i + 1 < args.len() => {
                physics = parse_physics(args[i + 1].as_str()).unwrap_or_else(|e| {
                    eprintln!("{e}");
                    std::process::exit(2);
                });
                i += 2;
            }
            "--mix" if i + 3 < args.len() => {
                let duration = args[i + 3].parse::<f32>().unwrap_or(0.0);
                state
                    .data_mut()
                    .set_mix(args[i + 1].as_str(), args[i + 2].as_str(), duration);
                i += 4;
            }
            "--set" if i + 3 < args.len() => {
                let track = args[i + 1].parse::<usize>().unwrap_or(0);
                let looped = parse_bool_flag(args[i + 3].as_str());
                last_entry = Some(state.set_animation(track, args[i + 2].as_str(), looped));
                i += 4;
            }
            "--add" if i + 4 < args.len() => {
                let track = args[i + 1].parse::<usize>().unwrap_or(0);
                let looped = parse_bool_flag(args[i + 3].as_str());
                let delay = args[i + 4].parse::<f32>().unwrap_or(0.0);
                last_entry = Some(state.add_animation(track, args[i + 2].as_str(), looped, delay));
                i += 5;
            }
            "--set-empty" if i + 2 < args.len() => {
                let track = args[i + 1].parse::<usize>().unwrap_or(0);
                let mix_duration = args[i + 2].parse::<f32>().unwrap_or(0.0);
                last_entry = Some(state.set_empty_animation(track, mix_duration));
                i += 3;
            }
            "--add-empty" if i + 3 < args.len() => {
                let track = args[i + 1].parse::<usize>().unwrap_or(0);
                let mix_duration = args[i + 2].parse::<f32>().unwrap_or(0.0);
                let delay = args[i + 3].parse::<f32>().unwrap_or(0.0);
                last_entry = Some(state.add_empty_animation(track, mix_duration, delay));
                i += 4;
            }
            "--entry-alpha" if i + 1 < args.len() => {
                let alpha = args[i + 1].parse::<f32>().unwrap_or(0.0);
                require_last_entry(last_entry, "--entry-alpha").set_alpha(&mut state, alpha);
                i += 2;
            }
            "--entry-event-threshold" if i + 1 < args.len() => {
                let threshold = args[i + 1].parse::<f32>().unwrap_or(0.0);
                require_last_entry(last_entry, "--entry-event-threshold")
                    .set_event_threshold(&mut state, threshold);
                i += 2;
            }
            "--entry-alpha-attachment-threshold" if i + 1 < args.len() => {
                let threshold = args[i + 1].parse::<f32>().unwrap_or(0.0);
                require_last_entry(last_entry, "--entry-alpha-attachment-threshold")
                    .set_alpha_attachment_threshold(&mut state, threshold);
                i += 2;
            }
            "--entry-mix-attachment-threshold" if i + 1 < args.len() => {
                let threshold = args[i + 1].parse::<f32>().unwrap_or(0.0);
                require_last_entry(last_entry, "--entry-mix-attachment-threshold")
                    .set_mix_attachment_threshold(&mut state, threshold);
                i += 2;
            }
            "--entry-mix-draw-order-threshold" if i + 1 < args.len() => {
                let threshold = args[i + 1].parse::<f32>().unwrap_or(0.0);
                require_last_entry(last_entry, "--entry-mix-draw-order-threshold")
                    .set_mix_draw_order_threshold(&mut state, threshold);
                i += 2;
            }
            "--entry-additive" if i + 1 < args.len() => {
                let additive = parse_bool_flag(args[i + 1].as_str());
                require_last_entry(last_entry, "--entry-additive")
                    .set_additive(&mut state, additive);
                i += 2;
            }
            "--entry-mix-interpolation" if i + 1 < args.len() => {
                let interpolation =
                    parse_mix_interpolation(args[i + 1].as_str()).unwrap_or_else(|e| {
                        eprintln!("{e}");
                        std::process::exit(2);
                    });
                require_last_entry(last_entry, "--entry-mix-interpolation")
                    .set_mix_interpolation(&mut state, interpolation);
                i += 2;
            }
            "--entry-reverse" if i + 1 < args.len() => {
                let reverse = parse_bool_flag(args[i + 1].as_str());
                require_last_entry(last_entry, "--entry-reverse").set_reverse(&mut state, reverse);
                i += 2;
            }
            "--entry-shortest-rotation" if i + 1 < args.len() => {
                let shortest_rotation = parse_bool_flag(args[i + 1].as_str());
                require_last_entry(last_entry, "--entry-shortest-rotation")
                    .set_shortest_rotation(&mut state, shortest_rotation);
                i += 2;
            }
            "--entry-reset-rotation-directions" => {
                require_last_entry(last_entry, "--entry-reset-rotation-directions")
                    .reset_rotation_directions(&mut state);
                i += 1;
            }
            "--step" if i + 1 < args.len() => {
                let dt = args[i + 1].parse::<f32>().unwrap_or(0.0);
                state.update(dt);
                state.apply(&mut skeleton);
                skeleton.update(dt);
                skeleton.update_world_transform_with_physics(physics);
                total_time += dt;
                i += 2;
            }
            _ => usage(),
        }
    }

    let draw_list = spine2d::build_draw_list_with_atlas(&skeleton, &atlas);

    let mut page_index_by_name: HashMap<&str, usize> = HashMap::new();
    for (i, page) in atlas.pages.iter().enumerate() {
        page_index_by_name.insert(page.name.as_str(), i);
    }

    // Manual JSON writing keeps this example dependency-free and avoids `serde_json` feature
    // coupling, while still providing a stable oracle format.
    let skin_json = skin
        .as_ref()
        .map(|s| format!("\"{}\"", json_escape(s)))
        .unwrap_or_else(|| "null".to_string());
    println!(
        "{{\"mode\":\"scenario\",\"y_down\":{},\"pma\":{},\"physics\":\"{}\",\"skin\":{},\"anim\":\"<scenario>\",\"time\":{},\"draws\":[",
        if y_down { 1 } else { 0 },
        if atlas_uses_pma(&atlas) { 1 } else { 0 },
        physics_name(physics),
        skin_json,
        total_time
    );

    for (draw_i, draw) in draw_list.draws.iter().enumerate() {
        if draw_i != 0 {
            print!(",");
        }

        let page_index = page_index_by_name
            .get(draw.texture_path.as_str())
            .copied()
            .map(|i| i as i32)
            .unwrap_or(-1);
        let blend = match draw.blend {
            spine2d::BlendMode::Normal => "normal",
            spine2d::BlendMode::Additive => "additive",
            spine2d::BlendMode::Multiply => "multiply",
            spine2d::BlendMode::Screen => "screen",
        };

        let indices = &draw_list.indices[draw.first_index..(draw.first_index + draw.index_count)];

        // Compute the vertex range used by this draw (conservative: scan indices).
        let mut min_v = u32::MAX;
        let mut max_v = 0u32;
        for &idx in indices {
            min_v = min_v.min(idx);
            max_v = max_v.max(idx);
        }
        let start_v = min_v as usize;
        let end_v = (max_v as usize).saturating_add(1);
        let vertices = &draw_list.vertices[start_v..end_v];

        print!(
            "{{\"page\":{page_index},\"texture\":\"{}\",\"blend\":\"{blend}\",\"num_vertices\":{},\"num_indices\":{},",
            json_escape(&draw.texture_path),
            vertices.len(),
            indices.len()
        );

        print!("\"positions\":[");
        for (i, v) in vertices.iter().enumerate() {
            if i != 0 {
                print!(",");
            }
            print!("{},{}", v.position[0], v.position[1]);
        }
        print!("],");

        print!("\"uvs\":[");
        for (i, v) in vertices.iter().enumerate() {
            if i != 0 {
                print!(",");
            }
            print!("{},{}", v.uv[0], v.uv[1]);
        }
        print!("],");

        print!("\"colors\":[");
        for (i, v) in vertices.iter().enumerate() {
            if i != 0 {
                print!(",");
            }
            print!("{}", pack_aarrggbb(v.color));
        }
        print!("],");

        print!("\"dark_colors\":[");
        for (i, v) in vertices.iter().enumerate() {
            if i != 0 {
                print!(",");
            }
            print!("{}", pack_aarrggbb(v.dark_color));
        }
        print!("],");

        print!("\"indices\":[");
        for (i, idx) in indices.iter().enumerate() {
            if i != 0 {
                print!(",");
            }
            print!("{}", idx - min_v);
        }
        print!("]}}");
    }

    println!("]}}");
}
