use bevy::{
    asset::AssetPlugin,
    camera::{OrthographicProjection, Projection},
    prelude::*,
    window::WindowPlugin,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use serde::Deserialize;
use spine2d_bevy::{
    Spine, Spine2dPlugin, SpineAnimationCommand, SpineAnimationStateConfig, SpineRuntimeState,
    SpineTrackEntrySettings,
};
use std::{fs, path::Path};

const ASSET_ROOT: &str = "..";
const MANIFEST_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/spine-runtimes/web_manifest.json"
);
const MANIFEST_BASE: &str = "assets/spine-runtimes";
const PREFERRED_EXAMPLE: &str = "spineboy";
const DEMO_SKELETON: &str = "spine2d-web/assets/demo.json";
const DEMO_ATLAS: &str = "spine2d-web/assets/demo.atlas";
const FIT_MARGIN: f32 = 1.2;
const MIN_CAMERA_SCALE: f32 = 0.1;

#[derive(Component)]
struct MixingSpine;

#[derive(Component)]
struct MixingCamera;

#[derive(Resource)]
struct MixingAssets {
    first: String,
    second: String,
}

#[derive(Resource)]
struct MixingControls {
    default_mix: f32,
    pair_mix: f32,
    entry_mix: f32,
    queued_mix: f32,
    empty_mix: f32,
    alpha: f32,
    additive: bool,
    smooth_mix: bool,
    reverse: bool,
}

impl Default for MixingControls {
    fn default() -> Self {
        Self {
            default_mix: 0.15,
            pair_mix: 0.35,
            entry_mix: 0.35,
            queued_mix: 0.25,
            empty_mix: 0.3,
            alpha: 1.0,
            additive: false,
            smooth_mix: false,
            reverse: false,
        }
    }
}

#[derive(Deserialize)]
struct WebManifest {
    base: Option<String>,
    examples: Vec<WebManifestEntry>,
}

#[derive(Deserialize)]
struct WebManifestEntry {
    name: String,
    skeleton: String,
    atlas: String,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: ASSET_ROOT.to_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "spine2d-bevy mixing inspector".into(),
                        resolution: (1000, 700).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .add_plugins(EguiPlugin::default())
        .init_resource::<MixingControls>()
        .add_systems(Startup, setup)
        .add_systems(Update, fit_camera_to_spine)
        .add_systems(EguiPrimaryContextPass, inspector_panel)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, controls: Res<MixingControls>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        MixingCamera,
    ));

    let assets = load_mixing_assets();
    commands.insert_resource(MixingAssets {
        first: assets.first_animation.clone(),
        second: assets.second_animation.clone(),
    });
    commands.spawn((
        Spine::new(
            asset_server.load(assets.skeleton),
            asset_server.load(assets.atlas),
        )
        .with_animation(assets.first_animation.clone(), true),
        SpineAnimationStateConfig::new()
            .with_default_mix(controls.default_mix)
            .with_mix(
                assets.first_animation.clone(),
                assets.second_animation.clone(),
                controls.pair_mix,
            )
            .with_mix(
                assets.second_animation.clone(),
                assets.first_animation.clone(),
                controls.pair_mix,
            ),
        Transform::default(),
        MixingSpine,
    ));
}

fn inspector_panel(
    mut contexts: EguiContexts,
    assets: Res<MixingAssets>,
    mut controls: ResMut<MixingControls>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    mut query: Query<
        (
            Entity,
            &mut SpineAnimationStateConfig,
            Option<&SpineRuntimeState>,
        ),
        With<MixingSpine>,
    >,
) {
    let Ok(context) = contexts.ctx_mut() else {
        return;
    };
    let Ok((entity, mut config, runtime_state)) = query.single_mut() else {
        return;
    };

    egui::Window::new("Mixing Inspector")
        .default_width(340.0)
        .show(context, |ui| {
            ui.label(format!("animation A: {}", assets.first));
            ui.label(format!("animation B: {}", assets.second));
            ui.separator();

            let mut config_changed = false;
            config_changed |= ui
                .add(egui::Slider::new(&mut controls.default_mix, 0.0..=1.0).text("default mix"))
                .changed();
            config_changed |= ui
                .add(egui::Slider::new(&mut controls.pair_mix, 0.0..=1.0).text("pair mix"))
                .changed();
            if config_changed {
                *config = SpineAnimationStateConfig::new()
                    .with_default_mix(controls.default_mix)
                    .with_mix(
                        assets.first.clone(),
                        assets.second.clone(),
                        controls.pair_mix,
                    )
                    .with_mix(
                        assets.second.clone(),
                        assets.first.clone(),
                        controls.pair_mix,
                    );
            }

            ui.separator();
            ui.add(egui::Slider::new(&mut controls.entry_mix, 0.0..=1.0).text("entry mix"));
            ui.add(egui::Slider::new(&mut controls.queued_mix, 0.0..=1.0).text("queued mix"));
            ui.add(egui::Slider::new(&mut controls.empty_mix, 0.0..=1.0).text("empty mix"));
            ui.add(egui::Slider::new(&mut controls.alpha, 0.0..=1.0).text("alpha"));
            ui.checkbox(&mut controls.additive, "additive");
            ui.checkbox(&mut controls.smooth_mix, "smooth mix");
            ui.checkbox(&mut controls.reverse, "reverse");

            ui.horizontal(|ui| {
                if ui.button("Blend to A").clicked() {
                    write_set(&mut animation_commands, entity, &assets.first, &controls);
                }
                if ui.button("Blend to B").clicked() {
                    write_set(&mut animation_commands, entity, &assets.second, &controls);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Queue A").clicked() {
                    animation_commands.write(
                        SpineAnimationCommand::add(entity, 0, assets.first.clone(), false, 0.0)
                            .with_entry_settings(entry_settings(&controls, controls.queued_mix)),
                    );
                }
                if ui.button("Fade to setup").clicked() {
                    animation_commands.write(SpineAnimationCommand::set_empty(
                        entity,
                        0,
                        controls.empty_mix,
                    ));
                }
            });

            ui.separator();
            if let Some(state) = runtime_state {
                let current = state
                    .get_tracks()
                    .first()
                    .map(|track| track.get_animation_name())
                    .unwrap_or("<none>");
                ui.label(format!("current: {current}"));
                ui.label(format!("tracks: {}", state.get_tracks().len()));
                if let Some(track) = state.get_tracks().first() {
                    ui.label(format!(
                        "track time: {:.2}, mix: {:.2}/{:.2}, alpha: {:.2}",
                        track.get_track_time(),
                        track.get_mix_time(),
                        track.get_mix_duration(),
                        track.get_alpha()
                    ));
                }
            }
        });
}

fn write_set(
    animation_commands: &mut MessageWriter<SpineAnimationCommand>,
    entity: Entity,
    animation: &str,
    controls: &MixingControls,
) {
    animation_commands.write(
        SpineAnimationCommand::set(entity, 0, animation.to_owned(), true)
            .with_entry_settings(entry_settings(controls, controls.entry_mix)),
    );
}

fn entry_settings(controls: &MixingControls, mix_duration: f32) -> SpineTrackEntrySettings {
    let settings = SpineTrackEntrySettings::new()
        .with_mix_duration(mix_duration)
        .with_alpha(controls.alpha)
        .with_additive(controls.additive)
        .with_reverse(controls.reverse);

    if controls.smooth_mix {
        settings.with_mix_interpolation(spine2d::MixInterpolation::Smooth)
    } else {
        settings
    }
}

fn fit_camera_to_spine(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<MixingCamera>>,
    spine_query: Query<&SpineRuntimeState, With<MixingSpine>>,
    window_query: Query<&Window>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let Ok(state) = spine_query.single() else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };

    let size = state.get_bounds().size();
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }

    camera_transform.translation.x = state.get_bounds().center().x;
    camera_transform.translation.y = state.get_bounds().center().y;
    let viewport = Vec2::new(window.width().max(1.0), window.height().max(1.0));
    let fit_scale = (size.x / viewport.x)
        .max(size.y / viewport.y)
        .max(MIN_CAMERA_SCALE)
        * FIT_MARGIN;
    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scale = fit_scale;
    }
}

struct MixingAssetSet {
    skeleton: String,
    atlas: String,
    first_animation: String,
    second_animation: String,
}

fn load_mixing_assets() -> MixingAssetSet {
    manifest_mixing_assets().unwrap_or_else(|err| {
        info!("Using bundled demo assets for mixing inspector: {err}");
        MixingAssetSet {
            skeleton: DEMO_SKELETON.to_owned(),
            atlas: DEMO_ATLAS.to_owned(),
            first_animation: "spin".to_owned(),
            second_animation: "spin".to_owned(),
        }
    })
}

fn manifest_mixing_assets() -> Result<MixingAssetSet, String> {
    let text =
        fs::read_to_string(MANIFEST_PATH).map_err(|err| format!("read {MANIFEST_PATH}: {err}"))?;
    let manifest = serde_json::from_str::<WebManifest>(&text)
        .map_err(|err| format!("parse {MANIFEST_PATH}: {err}"))?;
    let base = manifest.base.unwrap_or_else(|| MANIFEST_BASE.to_owned());
    let entry = manifest
        .examples
        .iter()
        .find(|entry| entry.name == PREFERRED_EXAMPLE)
        .or_else(|| manifest.examples.first())
        .ok_or_else(|| "manifest has no examples".to_owned())?;
    let skeleton_path = Path::new(&base).join(&entry.skeleton);
    let text = fs::read_to_string(&skeleton_path)
        .map_err(|err| format!("read {}: {err}", skeleton_path.display()))?;
    let skeleton_json = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|err| format!("parse {}: {err}", skeleton_path.display()))?;
    let animations = skeleton_json
        .get("animations")
        .and_then(|value| value.as_object())
        .ok_or_else(|| format!("{} has no animations object", skeleton_path.display()))?;
    let (first, second) = preferred_animation_pair(animations)
        .ok_or_else(|| format!("{} has fewer than two animations", skeleton_path.display()))?;

    Ok(MixingAssetSet {
        skeleton: format!("{base}/{}", entry.skeleton),
        atlas: format!("{base}/{}", entry.atlas),
        first_animation: first.to_owned(),
        second_animation: second.to_owned(),
    })
}

fn preferred_animation_pair(
    animations: &serde_json::Map<String, serde_json::Value>,
) -> Option<(&str, &str)> {
    [
        ("walk", "run"),
        ("idle", "walk"),
        ("idle", "run"),
        ("jump", "run"),
        ("run", "walk"),
        ("run", "shoot"),
        ("run", "portal"),
        ("drive", "shoot"),
        ("walk", "hello"),
    ]
    .into_iter()
    .find(|(first, second)| animations.contains_key(*first) && animations.contains_key(*second))
    .or_else(|| {
        let mut names = animations.keys().map(String::as_str).collect::<Vec<_>>();
        names.sort_unstable();
        (names.len() >= 2).then(|| (names[0], names[1]))
    })
}
