use bevy::{asset::AssetPlugin, prelude::*, window::WindowPlugin};
use spine2d_bevy::{Spine, Spine2dPlugin, SpineSkeletonAsset};

#[derive(Resource)]
struct DemoSkeleton(Handle<SpineSkeletonAsset>);

#[derive(Resource, Default)]
struct MetadataLogged(bool);

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
                        title: "spine2d-bevy metadata".into(),
                        resolution: (800, 600).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(Spine2dPlugin)
        .init_resource::<MetadataLogged>()
        .add_systems(Startup, setup)
        .add_systems(Update, log_loaded_metadata)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let skeleton = asset_server.load("demo.json");
    let atlas = asset_server.load("demo.atlas");

    commands.insert_resource(DemoSkeleton(skeleton.clone()));
    commands.spawn((
        Spine::new(skeleton, atlas).with_animation_name("spin", true),
        Transform::from_scale(Vec3::splat(1.5)),
    ));
}

fn log_loaded_metadata(
    skeleton: Res<DemoSkeleton>,
    skeleton_assets: Res<Assets<SpineSkeletonAsset>>,
    mut logged: ResMut<MetadataLogged>,
) {
    if logged.0 {
        return;
    }

    let Some(asset) = skeleton_assets.get(&skeleton.0) else {
        return;
    };

    let info = asset.info();
    info!("animations: {:?}", info.animations);
    info!("skins: {:?}", info.skins);
    info!("events: {:?}", info.events);
    info!("has spin animation: {}", asset.has_animation("spin"));
    info!("has default skin: {}", asset.has_skin("default"));
    logged.0 = true;
}
