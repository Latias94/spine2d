mod asset_loader;
mod components;
mod materials;
mod spine_world;
mod systems;

use bevy::asset::AssetEventSystems;
use bevy::prelude::*;

pub use asset_loader::*;
pub use components::*;
pub(crate) use spine_world::*;

pub struct Spine2dPlugin;

impl Plugin for Spine2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpineSkeletonAsset>()
            .init_asset::<SpineAtlasAsset>()
            .init_asset_loader::<asset_loader::JsonSkeletonLoader>()
            .init_asset_loader::<asset_loader::BinarySkeletonLoader>()
            .init_asset_loader::<asset_loader::AtlasLoader>();
        app.add_message::<SpineLifecycleEvent>();
        app.add_message::<SpineAnimationEvent>();
        app.add_message::<SpineAnimationCommand>();

        app.insert_non_send_resource(SpineWorld::new());

        app.configure_sets(
            Update,
            (
                SpineSystemSet::Cleanup,
                SpineSystemSet::Spawn,
                SpineSystemSet::Commands,
                SpineSystemSet::Update,
                SpineSystemSet::Render,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                systems::cleanup_spine_instances.in_set(SpineSystemSet::Cleanup),
                systems::spawn_spine_instances.in_set(SpineSystemSet::Spawn),
                systems::apply_spine_animation_commands.in_set(SpineSystemSet::Commands),
                systems::update_spine_animations.in_set(SpineSystemSet::Update),
                systems::render_spines.in_set(SpineSystemSet::Render),
            ),
        );
        app.add_systems(
            PostUpdate,
            systems::reload_modified_spine_assets.after(AssetEventSystems),
        );

        app.add_plugins(materials::SpineMaterialPlugin);
    }
}
