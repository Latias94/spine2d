use bevy::{
    asset::AssetPlugin,
    camera::{OrthographicProjection, Projection},
    prelude::*,
    window::WindowPlugin,
};
use serde::Deserialize;
use spine2d_bevy::{
    Spine, Spine2dPlugin, SpineAnimation, SpineAnimationCommand, SpineBounds, SpineReady,
    SpineSkeletonAsset, SpineSkin, SpineSystemSet,
};
use std::fs;

const ASSET_ROOT: &str = "..";
const MANIFEST_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/spine-runtimes/web_manifest.json"
);
const DEMO_SKELETON: &str = "spine2d-web/assets/demo.json";
const DEMO_ATLAS: &str = "spine2d-web/assets/demo.atlas";
const FIT_MARGIN: f32 = 1.25;
const MIN_CAMERA_SCALE: f32 = 0.1;

#[derive(Component)]
struct ViewerSpine;

#[derive(Component)]
struct ViewerCamera;

#[derive(Component)]
struct StatusText;

#[derive(Resource)]
struct ViewerState {
    examples: Vec<ExampleEntry>,
    example_index: usize,
    animation_index: usize,
    skin_index: usize,
    playing: bool,
    speed: f32,
    fit_pending: bool,
}

#[derive(Clone, Debug)]
struct ExampleEntry {
    name: String,
    skeleton: String,
    atlas: String,
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
                        title: "spine2d-bevy viewer".into(),
                        resolution: (1100, 760).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .insert_resource(ViewerState {
            examples: load_examples(),
            example_index: 0,
            animation_index: 0,
            skin_index: 0,
            playing: true,
            speed: 1.0,
            fit_pending: true,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (handle_keyboard, sync_viewer_selection)
                .chain()
                .before(SpineSystemSet::Spawn),
        )
        .add_systems(Update, fit_camera_to_spine.after(SpineSystemSet::Update))
        .add_systems(Update, update_status_text.after(sync_viewer_selection))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, state: Res<ViewerState>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        ViewerCamera,
    ));

    commands.spawn((
        Text::new("Loading Spine viewer..."),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::srgb(0.88, 0.9, 0.92)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(14),
            top: px(12),
            max_width: px(620),
            ..default()
        },
        StatusText,
    ));

    let example = state.current_example();
    commands.spawn((
        Spine::new(
            asset_server.load(example.skeleton.clone()),
            asset_server.load(example.atlas.clone()),
        ),
        SpineAnimation::default(),
        SpineSkin::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ViewerSpine,
    ));
}

fn load_examples() -> Vec<ExampleEntry> {
    let mut examples = manifest_examples().unwrap_or_else(|err| {
        info!("Using bundled demo assets; manifest unavailable: {err}");
        Vec::new()
    });

    examples.insert(
        0,
        ExampleEntry {
            name: "demo".to_owned(),
            skeleton: DEMO_SKELETON.to_owned(),
            atlas: DEMO_ATLAS.to_owned(),
        },
    );
    examples
}

fn manifest_examples() -> Result<Vec<ExampleEntry>, String> {
    let text =
        fs::read_to_string(MANIFEST_PATH).map_err(|err| format!("read {MANIFEST_PATH}: {err}"))?;
    let manifest = serde_json::from_str::<WebManifest>(&text)
        .map_err(|err| format!("parse {MANIFEST_PATH}: {err}"))?;
    let base = manifest
        .base
        .unwrap_or_else(|| "assets/spine-runtimes".to_owned());

    Ok(manifest
        .examples
        .into_iter()
        .map(|entry| ExampleEntry {
            name: entry.name,
            skeleton: format!("{base}/{}", entry.skeleton),
            atlas: format!("{base}/{}", entry.atlas),
        })
        .collect())
}

fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ViewerState>,
    skeleton_assets: Res<Assets<SpineSkeletonAsset>>,
    spine_query: Query<&Spine, With<ViewerSpine>>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    ready_query: Query<Entity, (With<ViewerSpine>, With<SpineReady>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        state.playing = !state.playing;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        state.fit_pending = true;
    }
    if keyboard.just_pressed(KeyCode::Equal) {
        state.speed = (state.speed + 0.1).min(3.0);
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        state.speed = (state.speed - 0.1).max(0.0);
    }

    if keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::ArrowRight) {
        state.example_index = wrap_next(state.example_index, state.examples.len());
        state.animation_index = 0;
        state.skin_index = 0;
        state.fit_pending = true;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        state.example_index = wrap_prev(state.example_index, state.examples.len());
        state.animation_index = 0;
        state.skin_index = 0;
        state.fit_pending = true;
    }

    let Some(asset) = current_skeleton_asset(&state, &spine_query, &skeleton_assets) else {
        return;
    };
    let animations = sorted_names(asset.animations());
    let skins = sorted_names(asset.skins());

    if keyboard.just_pressed(KeyCode::KeyA) {
        state.animation_index = wrap_next(state.animation_index, animations.len());
    }
    if keyboard.just_pressed(KeyCode::KeyZ) {
        state.animation_index = wrap_prev(state.animation_index, animations.len());
    }
    if keyboard.just_pressed(KeyCode::KeyS) {
        state.skin_index = wrap_next(state.skin_index, skins.len() + 1);
        state.fit_pending = true;
    }
    if keyboard.just_pressed(KeyCode::KeyX) {
        state.skin_index = wrap_prev(state.skin_index, skins.len() + 1);
        state.fit_pending = true;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        let Some(animation) = selected_animation(&state, &animations) else {
            return;
        };
        for entity in &ready_query {
            animation_commands.write(SpineAnimationCommand::set(entity, 0, animation, true));
        }
    }
}

fn sync_viewer_selection(
    state: Res<ViewerState>,
    asset_server: Res<AssetServer>,
    skeleton_assets: Res<Assets<SpineSkeletonAsset>>,
    mut query: Query<(&mut Spine, &mut SpineAnimation, &mut SpineSkin), With<ViewerSpine>>,
) {
    let Ok((mut spine, mut animation, mut skin)) = query.single_mut() else {
        return;
    };

    let example = state.current_example();
    if state.is_changed() {
        let skeleton = asset_server.load(example.skeleton.clone());
        let atlas = asset_server.load(example.atlas.clone());
        if spine.skeleton != skeleton || spine.atlas != atlas {
            spine.skeleton = skeleton;
            spine.atlas = atlas;
            spine.set_changed();
        }
    }

    let Some(asset) = skeleton_assets.get(&spine.skeleton) else {
        animation.time_scale = 0.0;
        return;
    };

    let animations = sorted_names(asset.animations());
    let skins = sorted_names(asset.skins());
    let animation_name = selected_animation(&state, &animations).map(str::to_owned);
    let skin_name = selected_skin(&state, &skins).map(str::to_owned);
    let speed = if state.playing { state.speed } else { 0.0 };

    if animation.name != animation_name
        || !animation.loop_animation
        || (animation.time_scale - speed).abs() > f32::EPSILON
    {
        animation.name = animation_name;
        animation.loop_animation = true;
        animation.time_scale = speed;
        animation.set_changed();
    }

    if skin.name != skin_name {
        skin.name = skin_name;
        skin.set_changed();
    }
}

fn fit_camera_to_spine(
    mut state: ResMut<ViewerState>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<ViewerCamera>>,
    spine_query: Query<(&GlobalTransform, &SpineBounds), With<ViewerSpine>>,
    window_query: Query<&Window>,
) {
    if !state.fit_pending {
        return;
    }

    let Ok((spine_transform, bounds)) = spine_query.single() else {
        return;
    };
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };

    let bounds_size = bounds.size();
    if bounds_size.x <= 0.0 || bounds_size.y <= 0.0 {
        return;
    }

    let scale = spine_transform.to_scale_rotation_translation().0.truncate();
    let center = spine_transform.translation().truncate() + bounds.center() * scale;
    camera_transform.translation.x = center.x;
    camera_transform.translation.y = center.y;

    let viewport = Vec2::new(window.width().max(1.0), window.height().max(1.0));
    let scaled_size = bounds_size * scale.abs();
    let fit_scale = (scaled_size.x / viewport.x)
        .max(scaled_size.y / viewport.y)
        .max(MIN_CAMERA_SCALE)
        * FIT_MARGIN;

    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scale = fit_scale;
    }
    state.fit_pending = false;
}

fn update_status_text(
    state: Res<ViewerState>,
    skeleton_assets: Res<Assets<SpineSkeletonAsset>>,
    spine_query: Query<&Spine, With<ViewerSpine>>,
    mut text_query: Query<&mut Text, With<StatusText>>,
) {
    if !state.is_changed() && !skeleton_assets.is_changed() {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let example = state.current_example();
    let Some(asset) = current_skeleton_asset(&state, &spine_query, &skeleton_assets) else {
        **text = format!(
            "Example: {}\nLoading {}\n\nQ/E or Left/Right: example    A/Z: animation    S/X: skin\nSpace: play/pause    R: restart    -/=: speed    F: fit",
            example.name, example.skeleton
        );
        return;
    };

    let animations = sorted_names(asset.animations());
    let skins = sorted_names(asset.skins());
    let animation = selected_animation(&state, &animations).unwrap_or("<none>");
    let skin = selected_skin(&state, &skins).unwrap_or("<setup/default>");
    let playback = if state.playing { "playing" } else { "paused" };

    **text = format!(
        "Example: {} ({}/{})\nAnimation: {} ({}/{})    Skin: {} ({}/{})\nPlayback: {}    Speed: {:.2}x\n\nQ/E or Left/Right: example    A/Z: animation    S/X: skin\nSpace: play/pause    R: restart    -/=: speed    F: fit",
        example.name,
        state.example_index + 1,
        state.examples.len(),
        animation,
        selected_display_index(state.animation_index, animations.len()),
        animations.len(),
        skin,
        selected_display_index(state.skin_index, skins.len() + 1),
        skins.len() + 1,
        playback,
        state.speed,
    );
}

fn current_skeleton_asset<'a>(
    state: &ViewerState,
    spine_query: &Query<&Spine, With<ViewerSpine>>,
    skeleton_assets: &'a Assets<SpineSkeletonAsset>,
) -> Option<&'a SpineSkeletonAsset> {
    let spine = spine_query.single().ok()?;
    let asset = skeleton_assets.get(&spine.skeleton)?;
    let expected = state.current_example();
    let path = spine.skeleton.path()?.path().to_string_lossy();
    (path == expected.skeleton).then_some(asset)
}

fn selected_animation<'a>(state: &ViewerState, animations: &'a [&'a str]) -> Option<&'a str> {
    animations.get(state.animation_index).copied()
}

fn selected_skin<'a>(state: &ViewerState, skins: &'a [&'a str]) -> Option<&'a str> {
    if state.skin_index == 0 {
        None
    } else {
        skins.get(state.skin_index - 1).copied()
    }
}

fn sorted_names<'a>(names: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut names = names.collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn wrap_next(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (index + 1) % len }
}

fn wrap_prev(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (index + len - 1) % len }
}

fn selected_display_index(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { index.min(len - 1) + 1 }
}

impl ViewerState {
    fn current_example(&self) -> &ExampleEntry {
        &self.examples[self.example_index.min(self.examples.len() - 1)]
    }
}
