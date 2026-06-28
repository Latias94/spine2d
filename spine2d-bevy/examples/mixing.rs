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
const FIT_MARGIN: f32 = 1.25;
const MIN_CAMERA_SCALE: f32 = 0.1;
const PRESET_KEYS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

#[derive(Component)]
struct MixingSpine;

#[derive(Component)]
struct MixingCamera;

#[derive(Resource)]
struct MixingAssets {
    idle: String,
    walk: String,
    run: String,
    action: String,
}

#[derive(Resource)]
struct PresetDemo {
    presets: Vec<MixingPreset>,
    selected: usize,
    auto_play: bool,
    timer: Timer,
}

impl Default for PresetDemo {
    fn default() -> Self {
        Self {
            presets: Vec::new(),
            selected: 0,
            auto_play: true,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

#[derive(Clone)]
struct MixingPreset {
    name: &'static str,
    note: &'static str,
    hold_seconds: f32,
    action: PresetAction,
}

#[derive(Clone)]
enum PresetAction {
    Idle,
    WalkFromIdle,
    RunFromWalk,
    SoftStop,
    HardStop,
    ActionThenRun,
    FadeToSetup { mix: f32 },
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
                        title: "spine2d-bevy mixing presets".into(),
                        resolution: (1000, 700).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .add_plugins(EguiPlugin::default())
        .init_resource::<PresetDemo>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (handle_keyboard, run_auto_demo, fit_camera_to_spine),
        )
        .add_systems(EguiPrimaryContextPass, preset_panel)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut demo: ResMut<PresetDemo>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        MixingCamera,
    ));

    let assets = load_mixing_assets();
    demo.presets = presets();
    if let Some(hold_seconds) = demo.presets.first().map(|preset| preset.hold_seconds) {
        demo.timer
            .set_duration(std::time::Duration::from_secs_f32(hold_seconds));
    }
    commands.insert_resource(MixingAssets {
        idle: assets.animations.idle.clone(),
        walk: assets.animations.walk.clone(),
        run: assets.animations.run.clone(),
        action: assets.animations.action.clone(),
    });

    commands.spawn((
        Spine::new(
            asset_server.load(assets.skeleton),
            asset_server.load(assets.atlas),
        )
        .with_animation_name(assets.animations.idle.clone(), true),
        SpineAnimationStateConfig::new()
            .with_default_mix(0.2)
            .with_mix(
                assets.animations.idle.clone(),
                assets.animations.walk.clone(),
                0.35,
            )
            .with_mix(
                assets.animations.walk.clone(),
                assets.animations.run.clone(),
                0.25,
            )
            .with_mix(
                assets.animations.run.clone(),
                assets.animations.walk.clone(),
                0.25,
            )
            .with_mix(
                assets.animations.run.clone(),
                assets.animations.idle.clone(),
                0.45,
            )
            .with_mix(
                assets.animations.action.clone(),
                assets.animations.run.clone(),
                0.25,
            ),
        Transform::default(),
        MixingSpine,
    ));
}

fn presets() -> Vec<MixingPreset> {
    vec![
        MixingPreset {
            name: "Idle baseline",
            note: "Hold a stable idle loop before moving into gameplay transitions.",
            hold_seconds: 1.8,
            action: PresetAction::Idle,
        },
        MixingPreset {
            name: "Start walking",
            note: "Blend from idle into a walk loop, like a character starting to move.",
            hold_seconds: 2.8,
            action: PresetAction::WalkFromIdle,
        },
        MixingPreset {
            name: "Accelerate",
            note: "Blend from walk into run, a typical locomotion-speed transition.",
            hold_seconds: 2.6,
            action: PresetAction::RunFromWalk,
        },
        MixingPreset {
            name: "Soft stop",
            note: "Ease from run back to idle with enough mix time to hide the pose change.",
            hold_seconds: 2.8,
            action: PresetAction::SoftStop,
        },
        MixingPreset {
            name: "Hard stop",
            note: "Use a tiny mix to show the same stop as a near-instant gameplay cut.",
            hold_seconds: 2.0,
            action: PresetAction::HardStop,
        },
        MixingPreset {
            name: "Action then run",
            note: "Interrupt locomotion with a one-shot action, then queue run again.",
            hold_seconds: 3.8,
            action: PresetAction::ActionThenRun,
        },
        MixingPreset {
            name: "Fade out",
            note: "Mix all active tracks into empty animations, useful for despawn or unequip flows.",
            hold_seconds: 2.2,
            action: PresetAction::FadeToSetup { mix: 0.35 },
        },
    ]
}

fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    assets: Res<MixingAssets>,
    mut demo: ResMut<PresetDemo>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    query: Query<Entity, With<MixingSpine>>,
) {
    let Some(entity) = query.iter().next() else {
        return;
    };

    for (index, key) in PRESET_KEYS.into_iter().enumerate() {
        if keyboard.just_pressed(key) && index < demo.presets.len() {
            demo.auto_play = false;
            select_preset(index, entity, &assets, &mut demo, &mut animation_commands);
        }
    }

    if keyboard.just_pressed(KeyCode::Space) {
        demo.auto_play = !demo.auto_play;
        demo.timer.reset();
    }
}

fn run_auto_demo(
    time: Res<Time>,
    assets: Res<MixingAssets>,
    mut demo: ResMut<PresetDemo>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    query: Query<Entity, With<MixingSpine>>,
) {
    if !demo.auto_play || demo.presets.is_empty() {
        return;
    }
    demo.timer.tick(time.delta());
    if !demo.timer.just_finished() {
        return;
    }

    let Some(entity) = query.iter().next() else {
        return;
    };
    demo.selected = (demo.selected + 1) % demo.presets.len();
    run_selected_preset(entity, &assets, &mut demo, &mut animation_commands);
}

fn preset_panel(
    mut contexts: EguiContexts,
    assets: Res<MixingAssets>,
    mut demo: ResMut<PresetDemo>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    query: Query<(Entity, Option<&SpineRuntimeState>), With<MixingSpine>>,
) {
    let Ok(context) = contexts.ctx_mut() else {
        return;
    };
    let Ok((entity, runtime_state)) = query.single() else {
        return;
    };

    egui::Window::new("Mixing Presets")
        .default_width(360.0)
        .show(context, |ui| {
            ui.heading("Click a preset");
            ui.label(format!("idle: {}", assets.idle));
            ui.label(format!("walk: {}", assets.walk));
            ui.label(format!("run: {}", assets.run));
            ui.label(format!("action: {}", assets.action));
            ui.checkbox(&mut demo.auto_play, "auto play presets");
            ui.separator();

            for index in 0..demo.presets.len() {
                let preset = demo.presets[index].clone();
                let selected = demo.selected == index;
                let label = format!("{}. {}", index + 1, preset.name);
                if ui.selectable_label(selected, label).clicked() {
                    demo.auto_play = false;
                    select_preset(index, entity, &assets, &mut demo, &mut animation_commands);
                }
                if selected {
                    ui.label(preset.note);
                }
            }

            ui.separator();
            if let Some(state) = runtime_state {
                let current = state
                    .get_tracks()
                    .first()
                    .map(|track| track.get_animation_name())
                    .unwrap_or("<none>");
                ui.label(format!("current: {current}"));
                ui.label(format!("tracks: {}", state.get_tracks().len()));
                for track in state.get_tracks() {
                    ui.label(format!(
                        "track {}: {}, time: {:.2}, mix: {:.2}/{:.2}, alpha: {:.2}",
                        track.get_track_index(),
                        track.get_animation_name(),
                        track.get_track_time(),
                        track.get_mix_time(),
                        track.get_mix_duration(),
                        track.get_alpha()
                    ));
                }
            }
        });
}

fn select_preset(
    index: usize,
    entity: Entity,
    assets: &MixingAssets,
    demo: &mut PresetDemo,
    animation_commands: &mut MessageWriter<SpineAnimationCommand>,
) {
    demo.selected = index;
    run_selected_preset(entity, assets, demo, animation_commands);
}

fn run_selected_preset(
    entity: Entity,
    assets: &MixingAssets,
    demo: &mut PresetDemo,
    animation_commands: &mut MessageWriter<SpineAnimationCommand>,
) {
    let Some(preset) = demo.presets.get(demo.selected) else {
        return;
    };
    run_preset(preset, entity, assets, animation_commands);
    demo.timer
        .set_duration(std::time::Duration::from_secs_f32(preset.hold_seconds));
    demo.timer.reset();
}

fn run_preset(
    preset: &MixingPreset,
    entity: Entity,
    assets: &MixingAssets,
    animation_commands: &mut MessageWriter<SpineAnimationCommand>,
) {
    match preset.action {
        PresetAction::Idle => {
            clear_secondary_track(entity, animation_commands);
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, assets.idle.clone(), true)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.2)),
            );
        }
        PresetAction::WalkFromIdle => {
            clear_secondary_track(entity, animation_commands);
            set_loop_immediate(entity, &assets.idle, animation_commands);
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, assets.walk.clone(), true)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.35)),
            );
        }
        PresetAction::RunFromWalk => {
            clear_secondary_track(entity, animation_commands);
            set_loop_immediate(entity, &assets.walk, animation_commands);
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, assets.run.clone(), true)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.25)),
            );
        }
        PresetAction::SoftStop => {
            clear_secondary_track(entity, animation_commands);
            set_loop_immediate(entity, &assets.run, animation_commands);
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, assets.idle.clone(), true)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.45)),
            );
        }
        PresetAction::HardStop => {
            clear_secondary_track(entity, animation_commands);
            set_loop_immediate(entity, &assets.run, animation_commands);
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, assets.idle.clone(), true)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.03)),
            );
        }
        PresetAction::ActionThenRun => {
            clear_secondary_track(entity, animation_commands);
            set_loop_immediate(entity, &assets.run, animation_commands);
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, assets.action.clone(), false)
                    .with_entry_settings(
                        SpineTrackEntrySettings::new()
                            .with_track_end(1.0)
                            .with_mix_duration(0.12),
                    ),
            );
            animation_commands.write(
                SpineAnimationCommand::add(entity, 0, assets.run.clone(), true, 0.0)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.25)),
            );
        }
        PresetAction::FadeToSetup { mix } => {
            clear_secondary_track(entity, animation_commands);
            set_loop_immediate(entity, &assets.run, animation_commands);
            animation_commands.write(SpineAnimationCommand::set_empty_animations(entity, mix));
        }
    }
}

fn set_loop_immediate(
    entity: Entity,
    animation: &str,
    animation_commands: &mut MessageWriter<SpineAnimationCommand>,
) {
    animation_commands.write(
        SpineAnimationCommand::set(entity, 0, animation.to_owned(), true)
            .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.0)),
    );
}

fn clear_secondary_track(
    entity: Entity,
    animation_commands: &mut MessageWriter<SpineAnimationCommand>,
) {
    animation_commands.write(SpineAnimationCommand::clear_track(entity, 1));
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
    animations: MixingAnimationSet,
}

#[derive(Clone)]
struct MixingAnimationSet {
    idle: String,
    walk: String,
    run: String,
    action: String,
}

fn load_mixing_assets() -> MixingAssetSet {
    manifest_mixing_assets().unwrap_or_else(|err| {
        info!("Using bundled demo assets for mixing example: {err}");
        MixingAssetSet {
            skeleton: DEMO_SKELETON.to_owned(),
            atlas: DEMO_ATLAS.to_owned(),
            animations: MixingAnimationSet::single("spin"),
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
    let animation_set = real_world_animation_set(animations)
        .or_else(|| fallback_animation_set(animations))
        .ok_or_else(|| format!("{} has no usable animations", skeleton_path.display()))?;

    Ok(MixingAssetSet {
        skeleton: format!("{base}/{}", entry.skeleton),
        atlas: format!("{base}/{}", entry.atlas),
        animations: animation_set,
    })
}

impl MixingAnimationSet {
    fn single(animation: &str) -> Self {
        Self {
            idle: animation.to_owned(),
            walk: animation.to_owned(),
            run: animation.to_owned(),
            action: animation.to_owned(),
        }
    }
}

fn real_world_animation_set(
    animations: &serde_json::Map<String, serde_json::Value>,
) -> Option<MixingAnimationSet> {
    Some(MixingAnimationSet {
        idle: animation_named(animations, &["idle", "stand"])?.to_owned(),
        walk: animation_named(animations, &["walk"])?.to_owned(),
        run: animation_named(animations, &["run"])?.to_owned(),
        action: animation_named(animations, &["jump", "attack", "hit"])?.to_owned(),
    })
}

fn fallback_animation_set(
    animations: &serde_json::Map<String, serde_json::Value>,
) -> Option<MixingAnimationSet> {
    let (first, second) = preferred_animation_pair(animations)?;
    Some(MixingAnimationSet {
        idle: first.to_owned(),
        walk: first.to_owned(),
        run: second.to_owned(),
        action: second.to_owned(),
    })
}

fn animation_named<'a>(
    animations: &serde_json::Map<String, serde_json::Value>,
    candidates: &'a [&'a str],
) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|name| animations.contains_key(*name))
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
        ("run", "portal"),
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
