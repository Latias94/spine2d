use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};
use spine2d_bevy::{
    Spine, Spine2dPlugin, SpineAnimationCommand, SpineAnimationEvent, SpineLifecycleEvent,
    SpineReady, SpineTrackEntrySettings,
};

#[derive(Component)]
struct PlayerSpine;

#[derive(Resource, Default)]
struct PlaybackPaused(bool);

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
                        title: "spine2d-bevy basic".into(),
                        resolution: (800, 600).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .init_resource::<PlaybackPaused>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, log_spine_messages))
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
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.5)),
        PlayerSpine,
    ));
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<PlaybackPaused>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    spine_query: Query<Entity, (With<PlayerSpine>, With<SpineReady>)>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for entity in &spine_query {
        paused.0 = !paused.0;
        if paused.0 {
            animation_commands.write(SpineAnimationCommand::clear_track(entity, 0));
        } else {
            animation_commands.write(
                SpineAnimationCommand::set(entity, 0, "spin", true)
                    .with_entry_settings(SpineTrackEntrySettings::new().with_mix_duration(0.2)),
            );
        }
    }
}

fn log_spine_messages(
    mut lifecycle_events: MessageReader<SpineLifecycleEvent>,
    mut animation_events: MessageReader<SpineAnimationEvent>,
) {
    for event in lifecycle_events.read() {
        info!("Spine lifecycle: {:?}", event);
    }

    for event in animation_events.read() {
        info!("Spine animation event: {:?}", event);
    }
}
