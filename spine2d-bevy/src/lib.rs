mod asset_loader;
mod components;
mod spine_world;
mod systems;
mod materials;

use bevy::prelude::*;

pub use asset_loader::*;
pub use components::*;
pub use spine_world::*;

pub struct Spine2dPlugin;

impl Plugin for Spine2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpineSkeletonAsset>()
            .init_asset::<SpineAtlasAsset>()
            .init_asset_loader::<asset_loader::JsonSkeletonLoader>()
            .init_asset_loader::<asset_loader::BinarySkeletonLoader>()
            .init_asset_loader::<asset_loader::AtlasLoader>();

        app.insert_non_send_resource(SpineWorld::new());

        app.add_systems(Update, (
            systems::spawn_spine_instances,
            systems::update_spine_animations,
            systems::render_spines,
        ).chain());

        app.add_plugins(materials::SpineMaterialPlugin);
    }
}