use crate::runtime::{AnimationState, AnimationStateData};
use crate::{MixBlend, Skeleton, SkeletonData, apply_animation, build_draw_list};
use std::path::PathBuf;
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
        "Upstream Spine examples not found. Run `./scripts/import_spine_runtimes_examples.zsh --mode json` \
or set SPINE2D_UPSTREAM_EXAMPLES_DIR to <spine-runtimes>/examples."
    );
}

fn example_json_path(relative: &str) -> PathBuf {
    let candidate = upstream_examples_root().join(relative);
    if candidate.is_file() {
        return candidate;
    }

    let cache = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.cache/spine-runtimes/examples")
        .join(relative);
    if cache.is_file() {
        return cache;
    }

    candidate
}

fn assert_skeleton_finite(skeleton: &Skeleton) {
    for bone in &skeleton.bones {
        assert!(bone.world_x.is_finite());
        assert!(bone.world_y.is_finite());
        assert!(bone.a.is_finite());
        assert!(bone.b.is_finite());
        assert!(bone.c.is_finite());
        assert!(bone.d.is_finite());
    }
}

fn smoke_example(relative: &str) {
    let path = example_json_path(relative);
    let json = std::fs::read_to_string(&path).expect("read example json");
    let data: Arc<SkeletonData> = SkeletonData::from_json_str(&json).expect("parse example json");

    let mut skeleton = Skeleton::new(data.clone());
    skeleton.set_to_setup_pose();
    skeleton.update_world_transform();
    assert_skeleton_finite(&skeleton);

    if let Some(anim) = data.animations.first() {
        apply_animation(
            anim,
            &mut skeleton,
            anim.duration * 0.5,
            true,
            1.0,
            MixBlend::Replace,
        );
        skeleton.update_world_transform();
        assert_skeleton_finite(&skeleton);
    }

    // Broad coverage for the MixBlend::Add path: try a 2-track overlay when the asset has at least
    // two animations. This is a smoke check (no oracle parity), but it exercises a large number of
    // timeline types in the Add blending path.
    if data.animations.len() >= 2 {
        let mut skeleton = Skeleton::new(data.clone());
        let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
        skeleton.set_to_setup_pose();

        let a0 = data.animations[0].name.as_str();
        let a1 = data.animations[1].name.as_str();
        state.set_animation(0, a0, true).expect("set track0");
        let entry = state.set_animation(1, a1, true).expect("set track1");
        entry.set_additive(&mut state, true);

        let dt = 0.2;
        state.update(dt);
        state.apply(&mut skeleton);
        skeleton.update_world_transform();
        assert_skeleton_finite(&skeleton);

        let draw_list = build_draw_list(&skeleton);
        for v in &draw_list.vertices {
            assert!(v.position[0].is_finite());
            assert!(v.position[1].is_finite());
            assert!(v.uv[0].is_finite());
            assert!(v.uv[1].is_finite());
            assert!(v.color[0].is_finite());
            assert!(v.color[1].is_finite());
            assert!(v.color[2].is_finite());
            assert!(v.color[3].is_finite());
            assert!(v.dark_color[0].is_finite());
            assert!(v.dark_color[1].is_finite());
            assert!(v.dark_color[2].is_finite());
            assert!(v.dark_color[3].is_finite());
        }
    }

    let draw_list = build_draw_list(&skeleton);
    for v in &draw_list.vertices {
        assert!(v.position[0].is_finite());
        assert!(v.position[1].is_finite());
        assert!(v.uv[0].is_finite());
        assert!(v.uv[1].is_finite());
        assert!(v.color[0].is_finite());
        assert!(v.color[1].is_finite());
        assert!(v.color[2].is_finite());
        assert!(v.color[3].is_finite());
        assert!(v.dark_color[0].is_finite());
        assert!(v.dark_color[1].is_finite());
        assert!(v.dark_color[2].is_finite());
        assert!(v.dark_color[3].is_finite());
    }
}

#[test]
fn example_alien_ess_smoke() {
    smoke_example("alien/export/alien-ess.json");
}

#[test]
fn example_alien_pro_smoke() {
    smoke_example("alien/export/alien-pro.json");
}

#[test]
fn example_dragon_ess_smoke() {
    smoke_example("dragon/export/dragon-ess.json");
}

#[test]
fn example_diamond_pro_smoke() {
    smoke_example("diamond/export/diamond-pro.json");
}

#[test]
fn example_hero_ess_smoke() {
    smoke_example("hero/export/hero-ess.json");
}

#[test]
fn example_hero_pro_smoke() {
    smoke_example("hero/export/hero-pro.json");
}

#[test]
fn example_owl_pro_smoke() {
    smoke_example("owl/export/owl-pro.json");
}

#[test]
fn example_raptor_pro_smoke() {
    smoke_example("raptor/export/raptor-pro.json");
}

#[test]
fn example_spinosaurus_ess_smoke() {
    smoke_example("spinosaurus/export/spinosaurus-ess.json");
}

#[test]
fn example_stretchyman_pro_smoke() {
    smoke_example("stretchyman/export/stretchyman-pro.json");
}

#[test]
fn example_speedy_ess_smoke() {
    smoke_example("speedy/export/speedy-ess.json");
}

#[test]
fn example_windmill_ess_smoke() {
    smoke_example("windmill/export/windmill-ess.json");
}

#[test]
fn example_celestial_circus_pro_smoke() {
    smoke_example("celestial-circus/export/celestial-circus-pro.json");
}

#[test]
fn example_chibi_stickers_smoke() {
    smoke_example("chibi-stickers/export/chibi-stickers.json");
}

#[test]
fn example_cloud_pot_smoke() {
    smoke_example("cloud-pot/export/cloud-pot.json");
}

#[test]
fn example_coin_pro_smoke() {
    smoke_example("coin/export/coin-pro.json");
}

#[test]
fn example_6_arcs_pro_smoke() {
    smoke_example("6-arcs/export/6-arcs-pro.json");
}

#[test]
fn example_8_follow_through_pro_ball_smoke() {
    smoke_example("8-follow-through/export/8-follow-through-pro-ball.json");
}

#[test]
fn example_food_app_search_smoke() {
    smoke_example("food-app/export/food-app-search-ess.json");
}

#[test]
fn example_goblins_pro_smoke() {
    smoke_example("goblins/export/goblins-pro.json");
}

#[test]
fn example_powerup_pro_smoke() {
    smoke_example("powerup/export/powerup-pro.json");
}

#[test]
fn example_snowglobe_pro_smoke() {
    smoke_example("snowglobe/export/snowglobe-pro.json");
}

#[test]
fn example_mix_and_match_pro_smoke() {
    smoke_example("mix-and-match/export/mix-and-match-pro.json");
}

#[test]
fn example_spineboy_ess_smoke() {
    smoke_example("spineboy/export/spineboy-ess.json");
}

#[test]
fn example_spineboy_pro_smoke() {
    smoke_example("spineboy/export/spineboy-pro.json");
}

#[test]
fn example_tank_pro_smoke() {
    smoke_example("tank/export/tank-pro.json");
}
