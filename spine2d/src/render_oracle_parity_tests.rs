use crate::runtime::{AnimationState, AnimationStateData, TrackEntryHandle};
use crate::{Atlas, Physics, Skeleton, SkeletonData};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        "Upstream Spine examples not found. Run `python3 ./scripts/prepare_spine_runtimes_web_assets.py --scope tests` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn example_path(relative: &str) -> PathBuf {
    upstream_examples_root().join(relative)
}

fn golden_render_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/render_oracle_scenarios")
        .join(name)
}

#[cfg(feature = "binary")]
fn golden_render_skel_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/render_oracle_scenarios_skel")
        .join(name)
}

#[derive(Clone, Debug)]
struct RenderCase {
    name: &'static str,
    atlas: &'static str,
    skeleton: &'static str,
    anim: &'static str,
    time: f32,
    time_name: &'static str,
    looped: bool,
    skin: Option<&'static str>,
    physics: Physics,
}

#[derive(Clone, Debug)]
enum RenderScenarioCommand {
    Mix {
        from: &'static str,
        to: &'static str,
        duration: f32,
    },
    Set {
        track: usize,
        animation: &'static str,
        looped: bool,
    },
    EntryAlpha(f32),
    EntryAdditive(bool),
    Step(f32),
}

#[derive(Clone, Debug)]
struct RenderScenarioCase {
    name: &'static str,
    atlas: &'static str,
    skeleton: &'static str,
    commands: Vec<RenderScenarioCommand>,
    physics: Physics,
}

fn render_cases_json() -> Vec<RenderCase> {
    vec![
        RenderCase {
            name: "coin",
            atlas: "coin/export/coin-pma.atlas",
            skeleton: "coin/export/coin-pro.json",
            anim: "animation",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "coin_nonpma",
            atlas: "coin/export/coin.atlas",
            skeleton: "coin/export/coin-pro.json",
            anim: "animation",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "spineboy",
            atlas: "spineboy/export/spineboy-pma.atlas",
            skeleton: "spineboy/export/spineboy-pro.json",
            anim: "run",
            time: 0.2,
            time_name: "0_2",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "spineboy_nonpma",
            atlas: "spineboy/export/spineboy.atlas",
            skeleton: "spineboy/export/spineboy-pro.json",
            anim: "run",
            time: 0.2,
            time_name: "0_2",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "alien",
            atlas: "alien/export/alien-pma.atlas",
            skeleton: "alien/export/alien-pro.json",
            anim: "run",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "dragon",
            atlas: "dragon/export/dragon-pma.atlas",
            skeleton: "dragon/export/dragon-ess.json",
            anim: "flying",
            time: 0.25,
            time_name: "0_25",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "goblins",
            atlas: "goblins/export/goblins-pma.atlas",
            skeleton: "goblins/export/goblins-pro.json",
            anim: "walk",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "hero",
            atlas: "hero/export/hero-pma.atlas",
            skeleton: "hero/export/hero-pro.json",
            anim: "idle",
            time: 0.55,
            time_name: "0_55",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "hero_nonpma",
            atlas: "hero/export/hero.atlas",
            skeleton: "hero/export/hero-pro.json",
            anim: "idle",
            time: 0.55,
            time_name: "0_55",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_boy_pma",
            atlas: "mix-and-match/export/mix-and-match-pma.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.json",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/boy"),
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_girl_pma",
            atlas: "mix-and-match/export/mix-and-match-pma.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.json",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/girl"),
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_boy_nonpma",
            atlas: "mix-and-match/export/mix-and-match.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.json",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/boy"),
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_girl_nonpma",
            atlas: "mix-and-match/export/mix-and-match.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.json",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/girl"),
            physics: Physics::None,
        },
        RenderCase {
            name: "vine",
            atlas: "vine/export/vine-pma.atlas",
            skeleton: "vine/export/vine-pro.json",
            anim: "grow",
            time: 0.5,
            time_name: "0_5",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "tank",
            atlas: "tank/export/tank-pma.atlas",
            skeleton: "tank/export/tank-pro.json",
            anim: "shoot",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "chibi",
            atlas: "chibi-stickers/export/chibi-stickers-pma.atlas",
            skeleton: "chibi-stickers/export/chibi-stickers.json",
            anim: "movement/idle-front",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "chibi_davide_pma",
            atlas: "chibi-stickers/export/chibi-stickers-pma.atlas",
            skeleton: "chibi-stickers/export/chibi-stickers.json",
            anim: "movement/idle-front",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: Some("davide"),
            physics: Physics::None,
        },
        RenderCase {
            name: "chibi_davide_nonpma",
            atlas: "chibi-stickers/export/chibi-stickers.atlas",
            skeleton: "chibi-stickers/export/chibi-stickers.json",
            anim: "movement/idle-front",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: Some("davide"),
            physics: Physics::None,
        },
    ]
}

fn render_scenario_cases_json() -> Vec<RenderScenarioCase> {
    vec![
        RenderScenarioCase {
            name: "tank_scn_drive_to_shoot_midmix",
            atlas: "tank/export/tank-pma.atlas",
            skeleton: "tank/export/tank-pro.json",
            physics: Physics::None,
            commands: vec![
                RenderScenarioCommand::Mix {
                    from: "drive",
                    to: "shoot",
                    duration: 0.2,
                },
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "drive",
                    looped: true,
                },
                RenderScenarioCommand::Step(0.1),
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "shoot",
                    looped: false,
                },
                RenderScenarioCommand::Step(0.1),
            ],
        },
        RenderScenarioCase {
            name: "spineboy_scn_idle_to_shoot_midmix",
            atlas: "spineboy/export/spineboy-pma.atlas",
            skeleton: "spineboy/export/spineboy-pro.json",
            physics: Physics::None,
            commands: vec![
                RenderScenarioCommand::Mix {
                    from: "idle",
                    to: "shoot",
                    duration: 0.2,
                },
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "idle",
                    looped: true,
                },
                RenderScenarioCommand::Step(0.1),
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "shoot",
                    looped: false,
                },
                RenderScenarioCommand::Step(0.1),
            ],
        },
        RenderScenarioCase {
            name: "tank_scn_drive_plus_shoot_add_alpha0_5_t0_4",
            atlas: "tank/export/tank-pma.atlas",
            skeleton: "tank/export/tank-pro.json",
            physics: Physics::None,
            commands: vec![
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "drive",
                    looped: true,
                },
                RenderScenarioCommand::Step(0.1),
                RenderScenarioCommand::Set {
                    track: 1,
                    animation: "shoot",
                    looped: false,
                },
                RenderScenarioCommand::EntryAdditive(true),
                RenderScenarioCommand::EntryAlpha(0.5),
                RenderScenarioCommand::Step(0.3),
            ],
        },
    ]
}

#[cfg(feature = "binary")]
fn render_cases_skel() -> Vec<RenderCase> {
    vec![
        RenderCase {
            name: "coin",
            atlas: "coin/export/coin-pma.atlas",
            skeleton: "coin/export/coin-pro.skel",
            anim: "animation",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "coin_nonpma",
            atlas: "coin/export/coin.atlas",
            skeleton: "coin/export/coin-pro.skel",
            anim: "animation",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "spineboy",
            atlas: "spineboy/export/spineboy-pma.atlas",
            skeleton: "spineboy/export/spineboy-pro.skel",
            anim: "run",
            time: 0.2,
            time_name: "0_2",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "spineboy_nonpma",
            atlas: "spineboy/export/spineboy.atlas",
            skeleton: "spineboy/export/spineboy-pro.skel",
            anim: "run",
            time: 0.2,
            time_name: "0_2",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "alien",
            atlas: "alien/export/alien-pma.atlas",
            skeleton: "alien/export/alien-pro.skel",
            anim: "run",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "dragon",
            atlas: "dragon/export/dragon-pma.atlas",
            skeleton: "dragon/export/dragon-ess.skel",
            anim: "flying",
            time: 0.25,
            time_name: "0_25",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "goblins",
            atlas: "goblins/export/goblins-pma.atlas",
            skeleton: "goblins/export/goblins-pro.skel",
            anim: "walk",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "hero",
            atlas: "hero/export/hero-pma.atlas",
            skeleton: "hero/export/hero-pro.skel",
            anim: "idle",
            time: 0.55,
            time_name: "0_55",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "hero_nonpma",
            atlas: "hero/export/hero.atlas",
            skeleton: "hero/export/hero-pro.skel",
            anim: "idle",
            time: 0.55,
            time_name: "0_55",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_boy_pma",
            atlas: "mix-and-match/export/mix-and-match-pma.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.skel",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/boy"),
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_girl_pma",
            atlas: "mix-and-match/export/mix-and-match-pma.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.skel",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/girl"),
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_boy_nonpma",
            atlas: "mix-and-match/export/mix-and-match.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.skel",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/boy"),
            physics: Physics::None,
        },
        RenderCase {
            name: "mix_and_match_girl_nonpma",
            atlas: "mix-and-match/export/mix-and-match.atlas",
            skeleton: "mix-and-match/export/mix-and-match-pro.skel",
            anim: "walk",
            time: 0.1667,
            time_name: "0_1667",
            looped: true,
            skin: Some("full-skins/girl"),
            physics: Physics::None,
        },
        RenderCase {
            name: "vine",
            atlas: "vine/export/vine-pma.atlas",
            skeleton: "vine/export/vine-pro.skel",
            anim: "grow",
            time: 0.5,
            time_name: "0_5",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "tank",
            atlas: "tank/export/tank-pma.atlas",
            skeleton: "tank/export/tank-pro.skel",
            anim: "shoot",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "chibi",
            atlas: "chibi-stickers/export/chibi-stickers-pma.atlas",
            skeleton: "chibi-stickers/export/chibi-stickers.skel",
            anim: "movement/idle-front",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: None,
            physics: Physics::None,
        },
        RenderCase {
            name: "chibi_davide_pma",
            atlas: "chibi-stickers/export/chibi-stickers-pma.atlas",
            skeleton: "chibi-stickers/export/chibi-stickers.skel",
            anim: "movement/idle-front",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: Some("davide"),
            physics: Physics::None,
        },
        RenderCase {
            name: "chibi_davide_nonpma",
            atlas: "chibi-stickers/export/chibi-stickers.atlas",
            skeleton: "chibi-stickers/export/chibi-stickers.skel",
            anim: "movement/idle-front",
            time: 0.3,
            time_name: "0_3",
            looped: true,
            skin: Some("davide"),
            physics: Physics::None,
        },
    ]
}

#[cfg(feature = "binary")]
fn render_scenario_cases_skel() -> Vec<RenderScenarioCase> {
    vec![
        RenderScenarioCase {
            name: "tank_scn_drive_to_shoot_midmix",
            atlas: "tank/export/tank-pma.atlas",
            skeleton: "tank/export/tank-pro.skel",
            physics: Physics::None,
            commands: vec![
                RenderScenarioCommand::Mix {
                    from: "drive",
                    to: "shoot",
                    duration: 0.2,
                },
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "drive",
                    looped: true,
                },
                RenderScenarioCommand::Step(0.1),
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "shoot",
                    looped: false,
                },
                RenderScenarioCommand::Step(0.1),
            ],
        },
        RenderScenarioCase {
            name: "spineboy_scn_idle_to_shoot_midmix",
            atlas: "spineboy/export/spineboy-pma.atlas",
            skeleton: "spineboy/export/spineboy-pro.skel",
            physics: Physics::None,
            commands: vec![
                RenderScenarioCommand::Mix {
                    from: "idle",
                    to: "shoot",
                    duration: 0.2,
                },
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "idle",
                    looped: true,
                },
                RenderScenarioCommand::Step(0.1),
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "shoot",
                    looped: false,
                },
                RenderScenarioCommand::Step(0.1),
            ],
        },
        RenderScenarioCase {
            name: "tank_scn_drive_plus_shoot_add_alpha0_5_t0_4",
            atlas: "tank/export/tank-pma.atlas",
            skeleton: "tank/export/tank-pro.skel",
            physics: Physics::None,
            commands: vec![
                RenderScenarioCommand::Set {
                    track: 0,
                    animation: "drive",
                    looped: true,
                },
                RenderScenarioCommand::Step(0.1),
                RenderScenarioCommand::Set {
                    track: 1,
                    animation: "shoot",
                    looped: false,
                },
                RenderScenarioCommand::EntryAdditive(true),
                RenderScenarioCommand::EntryAlpha(0.5),
                RenderScenarioCommand::Step(0.3),
            ],
        },
    ]
}

fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

#[cfg(feature = "binary")]
fn read_bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn load_skeleton_data(path: &Path) -> Arc<SkeletonData> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("skel") {
        #[cfg(feature = "binary")]
        {
            let bytes = read_bytes(path);
            return SkeletonData::from_skel_bytes(&bytes)
                .unwrap_or_else(|e| panic!("parse skel {path:?}: {e}"));
        }
        #[cfg(not(feature = "binary"))]
        {
            panic!("loading .skel requires `--features binary`");
        }
    }

    let json = read_to_string(path);
    SkeletonData::from_json_str(&json).unwrap_or_else(|e| panic!("parse json {path:?}: {e}"))
}

#[derive(Clone, Debug, Deserialize)]
struct RenderDoc {
    draws: Vec<RenderDrawDoc>,
}

#[derive(Clone, Debug, Deserialize)]
struct RenderDrawDoc {
    page: i32,
    blend: String,
    #[serde(rename = "num_vertices")]
    num_vertices: usize,
    positions: Vec<f32>,
    uvs: Vec<f32>,
    colors: Vec<u32>,
    #[serde(rename = "dark_colors")]
    dark_colors: Vec<u32>,
    indices: Vec<u32>,
}

#[derive(Copy, Clone, Debug)]
struct TriRef {
    draw_index: usize,
    tri_index: usize,
}

#[derive(Clone, Debug)]
struct Triangle {
    page: i32,
    blend: String,
    v: [(f32, f32, f32, f32); 3],
    c: [u32; 3],
    dc: [u32; 3],
    reference: TriRef,
}

fn triangles_from_doc(doc: &RenderDoc) -> Vec<Triangle> {
    let mut out = Vec::new();
    for (draw_i, draw) in doc.draws.iter().enumerate() {
        assert_eq!(
            draw.positions.len(),
            draw.num_vertices * 2,
            "draw[{draw_i}]: invalid positions length"
        );
        assert_eq!(
            draw.uvs.len(),
            draw.num_vertices * 2,
            "draw[{draw_i}]: invalid uvs length"
        );
        assert_eq!(
            draw.colors.len(),
            draw.num_vertices,
            "draw[{draw_i}]: invalid colors length"
        );
        assert_eq!(
            draw.dark_colors.len(),
            draw.num_vertices,
            "draw[{draw_i}]: invalid dark_colors length"
        );
        assert_eq!(
            draw.indices.len() % 3,
            0,
            "draw[{draw_i}]: indices length not divisible by 3"
        );

        let vertex = |idx: usize| -> ((f32, f32, f32, f32), u32, u32) {
            assert!(
                idx < draw.num_vertices,
                "draw[{draw_i}]: index out of range"
            );
            let x = draw.positions[idx * 2];
            let y = draw.positions[idx * 2 + 1];
            let u = draw.uvs[idx * 2];
            let v = draw.uvs[idx * 2 + 1];
            ((x, y, u, v), draw.colors[idx], draw.dark_colors[idx])
        };

        for tri_i in 0..(draw.indices.len() / 3) {
            let i0 = draw.indices[tri_i * 3] as usize;
            let i1 = draw.indices[tri_i * 3 + 1] as usize;
            let i2 = draw.indices[tri_i * 3 + 2] as usize;
            let (v0, c0, d0) = vertex(i0);
            let (v1, c1, d1) = vertex(i1);
            let (v2, c2, d2) = vertex(i2);
            out.push(Triangle {
                page: draw.page,
                blend: draw.blend.clone(),
                v: [v0, v1, v2],
                c: [c0, c1, c2],
                dc: [d0, d1, d2],
                reference: TriRef {
                    draw_index: draw_i,
                    tri_index: tri_i,
                },
            });
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

fn max_color_channel_diff(a: u32, b: u32) -> u32 {
    let aa = (a >> 24) & 0xff;
    let ar = (a >> 16) & 0xff;
    let ag = (a >> 8) & 0xff;
    let ab = a & 0xff;

    let ba = (b >> 24) & 0xff;
    let br = (b >> 16) & 0xff;
    let bg = (b >> 8) & 0xff;
    let bb = b & 0xff;

    (aa.abs_diff(ba))
        .max(ar.abs_diff(br))
        .max(ag.abs_diff(bg))
        .max(ab.abs_diff(bb))
}

fn triangles_from_rust(skeleton: &Skeleton, atlas: &Atlas) -> Vec<Triangle> {
    let draw_list = crate::build_draw_list_with_atlas(skeleton, atlas);

    let mut page_index_by_name: HashMap<&str, i32> = HashMap::new();
    for (i, page) in atlas.get_pages().iter().enumerate() {
        page_index_by_name.insert(page.name.as_str(), i as i32);
    }

    let mut out = Vec::new();
    for (draw_i, draw) in draw_list.draws.iter().enumerate() {
        let page = page_index_by_name
            .get(draw.texture_path.as_str())
            .copied()
            .unwrap_or(-1);
        let blend = match draw.blend {
            crate::BlendMode::Normal => "normal",
            crate::BlendMode::Additive => "additive",
            crate::BlendMode::Multiply => "multiply",
            crate::BlendMode::Screen => "screen",
        }
        .to_string();

        let indices = &draw_list.indices[draw.first_index..(draw.first_index + draw.index_count)];
        assert_eq!(
            indices.len() % 3,
            0,
            "draw[{draw_i}]: indices length not divisible by 3"
        );

        for tri_i in 0..(indices.len() / 3) {
            let i0 = indices[tri_i * 3] as usize;
            let i1 = indices[tri_i * 3 + 1] as usize;
            let i2 = indices[tri_i * 3 + 2] as usize;

            let v0 = draw_list.vertices[i0];
            let v1 = draw_list.vertices[i1];
            let v2 = draw_list.vertices[i2];

            out.push(Triangle {
                page,
                blend: blend.clone(),
                v: [
                    (v0.position[0], v0.position[1], v0.uv[0], v0.uv[1]),
                    (v1.position[0], v1.position[1], v1.uv[0], v1.uv[1]),
                    (v2.position[0], v2.position[1], v2.uv[0], v2.uv[1]),
                ],
                c: [
                    pack_aarrggbb(v0.color),
                    pack_aarrggbb(v1.color),
                    pack_aarrggbb(v2.color),
                ],
                dc: [
                    pack_aarrggbb(v0.dark_color),
                    pack_aarrggbb(v1.dark_color),
                    pack_aarrggbb(v2.dark_color),
                ],
                reference: TriRef {
                    draw_index: draw_i,
                    tri_index: tri_i,
                },
            });
        }
    }

    out
}

fn assert_render_parity(case: &RenderCase, golden_path: &Path) {
    let atlas_path = example_path(case.atlas);
    let skeleton_path = example_path(case.skeleton);
    let atlas_text = read_to_string(&atlas_path);
    let atlas = atlas_text
        .parse::<Atlas>()
        .unwrap_or_else(|e| panic!("parse atlas {atlas_path:?}: {e}"));

    let data = load_skeleton_data(&skeleton_path);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    if let Some(skin_name) = case.skin {
        skeleton.set_skin(Some(skin_name));
        skeleton.setup_pose();
        skeleton.update_cache();
    }

    let mut state = AnimationState::new(AnimationStateData::new(data));
    state.set_animation(0, case.anim, case.looped);
    state.update(case.time);
    state.apply(&mut skeleton);
    skeleton.update(case.time);
    skeleton.update_world_transform_with_physics(case.physics);

    let rust_tris = triangles_from_rust(&skeleton, &atlas);

    let golden_json = std::fs::read_to_string(golden_path)
        .unwrap_or_else(|e| panic!("read golden {golden_path:?}: {e}"));
    let golden: RenderDoc = serde_json::from_str(&golden_json)
        .unwrap_or_else(|e| panic!("parse golden {golden_path:?}: {e}"));
    let golden_tris = triangles_from_doc(&golden);

    let eps_pos = 1e-3_f32;
    let eps_uv = 1e-5_f32;
    let eps_color = 2_u32;

    assert_eq!(
        golden_tris.len(),
        rust_tris.len(),
        "{}: triangle count mismatch: {} != {}",
        case.name,
        golden_tris.len(),
        rust_tris.len()
    );

    for (i, (a, b)) in golden_tris.iter().zip(rust_tris.iter()).enumerate() {
        assert_eq!(
            a.page,
            b.page,
            "{}: triangle #{i}: page mismatch: {} != {} (A draw={}, tri={}, B draw={}, tri={})",
            case.name,
            a.page,
            b.page,
            a.reference.draw_index,
            a.reference.tri_index,
            b.reference.draw_index,
            b.reference.tri_index
        );
        assert_eq!(
            a.blend,
            b.blend,
            "{}: triangle #{i}: blend mismatch: {} != {} (A draw={}, tri={}, B draw={}, tri={})",
            case.name,
            a.blend,
            b.blend,
            a.reference.draw_index,
            a.reference.tri_index,
            b.reference.draw_index,
            b.reference.tri_index
        );

        for vi in 0..3 {
            let (ax, ay, au, av) = a.v[vi];
            let (bx, by, bu, bv) = b.v[vi];
            assert!(
                ax.is_finite() && ay.is_finite() && au.is_finite() && av.is_finite(),
                "{}: triangle #{i} vertex {vi}: non-finite A",
                case.name
            );
            assert!(
                bx.is_finite() && by.is_finite() && bu.is_finite() && bv.is_finite(),
                "{}: triangle #{i} vertex {vi}: non-finite B",
                case.name
            );

            let dx = (ax - bx).abs();
            let dy = (ay - by).abs();
            let du = (au - bu).abs();
            let dv = (av - bv).abs();

            assert!(
                dx <= eps_pos && dy <= eps_pos,
                "{}: triangle #{i} vertex {vi}: pos mismatch: dx={dx} dy={dy} (eps={eps_pos})",
                case.name
            );
            assert!(
                du <= eps_uv && dv <= eps_uv,
                "{}: triangle #{i} vertex {vi}: uv mismatch: du={du} dv={dv} (eps={eps_uv})",
                case.name
            );

            let dc = max_color_channel_diff(a.c[vi], b.c[vi]);
            assert!(
                dc <= eps_color,
                "{}: triangle #{i} vertex {vi}: color mismatch: diff={dc} A={:#x} B={:#x}",
                case.name,
                a.c[vi],
                b.c[vi]
            );
            let ddc = max_color_channel_diff(a.dc[vi], b.dc[vi]);
            assert!(
                ddc <= eps_color,
                "{}: triangle #{i} vertex {vi}: dark color mismatch: diff={ddc} A={:#x} B={:#x}",
                case.name,
                a.dc[vi],
                b.dc[vi]
            );
        }
    }
}

fn step_animation(state: &mut AnimationState, skeleton: &mut Skeleton, dt: f32, physics: Physics) {
    state.update(dt);
    state.apply(skeleton);
    skeleton.update(dt);
    skeleton.update_world_transform_with_physics(physics);
}

fn apply_entry_command(
    state: &mut AnimationState,
    last_entry: &TrackEntryHandle,
    cmd: &RenderScenarioCommand,
) {
    match *cmd {
        RenderScenarioCommand::EntryAlpha(alpha) => last_entry.set_alpha(state, alpha),
        RenderScenarioCommand::EntryAdditive(v) => last_entry.set_additive(state, v),
        _ => unreachable!("non-entry command passed to apply_entry_command"),
    }
}

fn assert_render_scenario_parity(case: &RenderScenarioCase, golden_path: &Path) {
    let atlas_path = example_path(case.atlas);
    let skeleton_path = example_path(case.skeleton);
    let atlas_text = read_to_string(&atlas_path);
    let atlas = atlas_text
        .parse::<Atlas>()
        .unwrap_or_else(|e| panic!("parse atlas {atlas_path:?}: {e}"));

    let data = load_skeleton_data(&skeleton_path);
    let mut skeleton = Skeleton::new(data.clone());
    skeleton.setup_pose();

    let mut state = AnimationState::new(AnimationStateData::new(data));

    let physics = case.physics;
    let mut last_entry: Option<TrackEntryHandle> = None;
    for cmd in &case.commands {
        match *cmd {
            RenderScenarioCommand::Mix { from, to, duration } => {
                state.get_data().set_mix(from, to, duration);
            }
            RenderScenarioCommand::Set {
                track,
                animation,
                looped,
            } => {
                last_entry = Some(state.set_animation(track, animation, looped));
            }
            RenderScenarioCommand::Step(dt) => {
                step_animation(&mut state, &mut skeleton, dt, physics)
            }
            RenderScenarioCommand::EntryAlpha(_) | RenderScenarioCommand::EntryAdditive(_) => {
                let Some(entry) = last_entry.as_ref() else {
                    panic!(
                        "{:?}: entry command requires a preceding set/add",
                        case.name
                    );
                };
                apply_entry_command(&mut state, entry, cmd);
            }
        }
    }

    let rust_tris = triangles_from_rust(&skeleton, &atlas);

    let golden_json = std::fs::read_to_string(golden_path)
        .unwrap_or_else(|e| panic!("read golden {golden_path:?}: {e}"));
    let golden: RenderDoc = serde_json::from_str(&golden_json)
        .unwrap_or_else(|e| panic!("parse golden {golden_path:?}: {e}"));
    let golden_tris = triangles_from_doc(&golden);

    // Scenario mode exercises mixing chains (including additive overlays),
    // which tends to magnify small floating-point differences across implementations.
    // Keep the tolerance tight enough to catch semantic mismatches but avoid flakiness.
    let eps_pos = 3e-3_f32;
    let eps_uv = 1e-5_f32;
    let eps_color = 2_u32;

    assert_eq!(
        golden_tris.len(),
        rust_tris.len(),
        "{}: triangle count mismatch: {} != {}",
        case.name,
        golden_tris.len(),
        rust_tris.len()
    );

    for (i, (a, b)) in golden_tris.iter().zip(rust_tris.iter()).enumerate() {
        assert_eq!(
            a.page,
            b.page,
            "{}: triangle #{i}: page mismatch: {} != {} (A draw={}, tri={}, B draw={}, tri={})",
            case.name,
            a.page,
            b.page,
            a.reference.draw_index,
            a.reference.tri_index,
            b.reference.draw_index,
            b.reference.tri_index
        );
        assert_eq!(
            a.blend,
            b.blend,
            "{}: triangle #{i}: blend mismatch: {} != {} (A draw={}, tri={}, B draw={}, tri={})",
            case.name,
            a.blend,
            b.blend,
            a.reference.draw_index,
            a.reference.tri_index,
            b.reference.draw_index,
            b.reference.tri_index
        );

        for vi in 0..3 {
            let (ax, ay, au, av) = a.v[vi];
            let (bx, by, bu, bv) = b.v[vi];
            assert!(
                ax.is_finite() && ay.is_finite() && au.is_finite() && av.is_finite(),
                "{}: triangle #{i} vertex {vi}: non-finite A",
                case.name
            );
            assert!(
                bx.is_finite() && by.is_finite() && bu.is_finite() && bv.is_finite(),
                "{}: triangle #{i} vertex {vi}: non-finite B",
                case.name
            );

            let dx = (ax - bx).abs();
            let dy = (ay - by).abs();
            let du = (au - bu).abs();
            let dv = (av - bv).abs();

            assert!(
                dx <= eps_pos && dy <= eps_pos,
                "{}: triangle #{i} vertex {vi}: pos mismatch: dx={dx} dy={dy} (eps={eps_pos})",
                case.name
            );
            assert!(
                du <= eps_uv && dv <= eps_uv,
                "{}: triangle #{i} vertex {vi}: uv mismatch: du={du} dv={dv} (eps={eps_uv})",
                case.name
            );

            let dc = max_color_channel_diff(a.c[vi], b.c[vi]);
            assert!(
                dc <= eps_color,
                "{}: triangle #{i} vertex {vi}: color mismatch: diff={dc} A={:#x} B={:#x}",
                case.name,
                a.c[vi],
                b.c[vi]
            );
            let ddc = max_color_channel_diff(a.dc[vi], b.dc[vi]);
            assert!(
                ddc <= eps_color,
                "{}: triangle #{i} vertex {vi}: dark color mismatch: diff={ddc} A={:#x} B={:#x}",
                case.name,
                a.dc[vi],
                b.dc[vi]
            );
        }
    }
}

#[test]
fn render_oracle_parity_json_cases_match_cpp() {
    for case in render_cases_json() {
        let golden_name = format!(
            "{}_{}_t{}.json",
            case.name,
            case.anim.replace('/', "__"),
            case.time_name
        );
        let golden = golden_render_path(&golden_name);
        assert_render_parity(&case, &golden);
    }
}

#[test]
fn render_oracle_parity_json_scenarios_match_cpp() {
    for case in render_scenario_cases_json() {
        let golden = golden_render_path(&format!("{}.json", case.name));
        assert_render_scenario_parity(&case, &golden);
    }
}

#[test]
#[cfg(feature = "binary")]
fn render_oracle_parity_skel_cases_match_cpp() {
    for case in render_cases_skel() {
        let golden_name = format!(
            "{}_{}_t{}.json",
            case.name,
            case.anim.replace('/', "__"),
            case.time_name
        );
        let golden = golden_render_skel_path(&golden_name);
        assert_render_parity(&case, &golden);
    }
}

#[test]
#[cfg(feature = "binary")]
fn render_oracle_parity_skel_scenarios_match_cpp() {
    for case in render_scenario_cases_skel() {
        let golden = golden_render_skel_path(&format!("{}.json", case.name));
        assert_render_scenario_parity(&case, &golden);
    }
}
