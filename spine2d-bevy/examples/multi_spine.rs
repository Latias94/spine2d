use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};
use spine2d_bevy::{Spine, Spine2dPlugin, SpineAnimationCommand, SpineReady};

#[derive(Component)]
struct ToggleableSpine;

#[derive(Resource, Default)]
struct SpinPaused(bool);

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
                        title: "spine2d-bevy multi spine".into(),
                        resolution: (900, 600).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .init_resource::<SpinPaused>()
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_all_spines)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let skeleton = asset_server.load("demo.json");
    let atlas = asset_server.load("demo.atlas");

    for (x, scale) in [(-260.0, 0.8), (0.0, 1.2), (260.0, 1.6)] {
        commands.spawn((
            Spine::new(skeleton.clone(), atlas.clone()).with_animation("spin", true),
            Transform::from_xyz(x, 0.0, 0.0).with_scale(Vec3::splat(scale)),
            ToggleableSpine,
        ));
    }
}

fn toggle_all_spines(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<SpinPaused>,
    mut animation_commands: MessageWriter<SpineAnimationCommand>,
    query: Query<Entity, (With<ToggleableSpine>, With<SpineReady>)>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    paused.0 = !paused.0;
    for entity in &query {
        if paused.0 {
            animation_commands.write(SpineAnimationCommand::clear_track(entity, 0));
        } else {
            animation_commands.write(SpineAnimationCommand::set(entity, 0, "spin", true));
        }
    }
}
