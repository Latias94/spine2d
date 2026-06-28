use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};
use spine2d_bevy::{
    Spine, Spine2dPlugin, SpineAnimationCommand, SpineAnimationStateConfig, SpineRuntimeState,
    SpineSkeletonCommand, SpineSkeletonControl, SpineTrackEntrySettings,
};

#[derive(Component)]
struct ControlledSpine;

#[derive(Resource, Default)]
struct ControlsLogged(bool);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../spine2d-web/assets".to_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "spine2d-bevy runtime controls".into(),
                        resolution: (800, 600).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .init_resource::<ControlsLogged>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, log_runtime_state_once))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        Spine::new(
            asset_server.load("demo.json"),
            asset_server.load("demo.atlas"),
        )
        .with_animation_name("spin", true),
        SpineAnimationStateConfig::new().with_default_mix(0.2),
        SpineSkeletonControl::new()
            .with_physics(spine2d::Physics::None)
            .with_wind(Vec2::new(1.0, 0.0))
            .with_gravity(Vec2::new(0.0, 1.0)),
        Transform::from_scale(Vec3::splat(1.5)),
        ControlledSpine,
    ));
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    mut skeleton_commands: MessageWriter<SpineSkeletonCommand>,
    query: Query<Entity, With<ControlledSpine>>,
) {
    for entity in &query {
        if keyboard.just_pressed(KeyCode::Digit1) {
            animation_commands.write(
                SpineAnimationCommand::set_animation(entity, 0, "spin", true).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_mix_duration(0.2)
                        .with_alpha(1.0),
                ),
            );
        }
        if keyboard.just_pressed(KeyCode::Digit2) {
            animation_commands.write(SpineAnimationCommand::set_empty_animation(entity, 0, 0.25));
        }
        if keyboard.just_pressed(KeyCode::Digit3) {
            skeleton_commands.write(SpineSkeletonCommand::set_physics(
                entity,
                spine2d::Physics::Update,
            ));
        }
        if keyboard.just_pressed(KeyCode::Digit4) {
            skeleton_commands.write(SpineSkeletonCommand::setup_pose(entity));
        }
    }
}

fn log_runtime_state_once(
    query: Query<&SpineRuntimeState, With<ControlledSpine>>,
    mut logged: ResMut<ControlsLogged>,
) {
    if logged.0 {
        return;
    }

    let Ok(state) = query.single() else {
        return;
    };

    info!(
        "runtime state: tracks={}, physics={:?}, bounds={:?}",
        state.get_tracks().len(),
        state.get_physics(),
        state.get_bounds()
    );
    logged.0 = true;
}
