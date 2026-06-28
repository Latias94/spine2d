use bevy::{asset::AssetPlugin, math::Isometry2d, prelude::*, window::WindowPlugin};
use spine2d_bevy::{Spine, Spine2dPlugin, SpineBounds, SpineReady};

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
                        title: "spine2d-bevy bounds".into(),
                        resolution: (800, 600).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_bounds)
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
        Transform::from_scale(Vec3::splat(1.5)),
    ));
}

fn draw_bounds(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &SpineBounds), With<SpineReady>>,
) {
    for (transform, bounds) in &query {
        let scale = transform.to_scale_rotation_translation().0.truncate();
        let translation = transform.translation().truncate();
        let center = translation + bounds.center() * scale;
        let size = bounds.size() * scale;

        gizmos.rect_2d(
            Isometry2d::from_translation(center),
            size,
            Color::srgb(0.1, 0.9, 0.6),
        );
    }
}
