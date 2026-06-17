use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};
use spine2d_bevy::{Spine, Spine2dPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "spine2d-web/assets".to_owned(),
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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Spine::new(
            asset_server.load("demo.json"),
            asset_server.load("demo.atlas"),
        )
        .with_animation("spin", true),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.5)),
    ));
}
