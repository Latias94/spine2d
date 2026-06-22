use spine2d::{AnimationState, AnimationStateData, Atlas, Physics, Skeleton, SkeletonData};
use std::{collections::HashMap, env, fs, path::Path, sync::Arc};

fn usage() -> ! {
    eprintln!(
        "Usage:\n  render_dump <atlas.atlas> <skeleton.(json|skel)> --anim <name> [--time <seconds>] [--loop 0|1]\n           [--skin <name|none>] [--physics none|reset|update|pose]\n"
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

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 4 {
        usage();
    }

    let atlas_path = Path::new(&args[0]).to_path_buf();
    let skeleton_path = Path::new(&args[1]).to_path_buf();
    args.drain(0..2);

    let mut skin: Option<String> = None;
    let mut anim: Option<String> = None;
    let mut time: f32 = 0.0;
    let mut looped: bool = true;
    let mut physics = Physics::None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--skin" if i + 1 < args.len() => {
                let v = args[i + 1].as_str();
                skin = if v == "none" {
                    None
                } else {
                    Some(v.to_string())
                };
                i += 2;
            }
            "--anim" if i + 1 < args.len() => {
                anim = Some(args[i + 1].to_string());
                i += 2;
            }
            "--time" if i + 1 < args.len() => {
                time = args[i + 1].parse::<f32>().unwrap_or(0.0);
                i += 2;
            }
            "--loop" if i + 1 < args.len() => {
                looped = args[i + 1].parse::<i32>().unwrap_or(1) != 0;
                i += 2;
            }
            "--physics" if i + 1 < args.len() => {
                physics = parse_physics(args[i + 1].as_str()).unwrap_or(Physics::None);
                i += 2;
            }
            _ => usage(),
        }
    }

    let Some(anim) = anim else {
        eprintln!("missing required --anim <name>");
        usage();
    };

    let atlas_text = read_to_string(&atlas_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });
    let atlas = Atlas::from_str(&atlas_text).unwrap_or_else(|e| {
        eprintln!("failed to parse {}: {e}", atlas_path.display());
        std::process::exit(2);
    });

    let data = load_skeleton_data(&skeleton_path).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2);
    });

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    if let Some(skin_name) = skin.as_deref() {
        skeleton.set_skin(Some(skin_name)).unwrap_or_else(|e| {
            eprintln!("failed to set skin {skin_name}: {e}");
            std::process::exit(2);
        });
        skeleton.setup_pose();
        skeleton.update_cache();
    }

    let mut state = AnimationState::new(AnimationStateData::new(data));
    state.set_animation(0, &anim, looped).unwrap_or_else(|e| {
        eprintln!("failed to set animation {anim}: {e}");
        std::process::exit(2);
    });
    state.update(time);
    state.apply(&mut skeleton);
    skeleton.update(time);
    skeleton.update_world_transform_with_physics(physics);

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
    let anim_json = json_escape(&anim);
    println!(
        "{{\"physics\":\"{}\",\"skin\":{},\"anim\":\"{}\",\"time\":{time},\"draws\":[",
        physics_name(physics),
        skin_json,
        anim_json
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
