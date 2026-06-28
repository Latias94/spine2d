mod render;

use bevy::asset::AssetEvent;
use bevy::ecs::lifecycle::RemovedComponents;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use spine2d::{AnimationState, AnimationStateData, Skeleton};
use std::collections::HashSet;

use render::despawn_mesh_children;
pub use render::render_spines;

use crate::{
    Spine, SpineAnimation, SpineAnimationCommand, SpineAnimationCommandKind, SpineAnimationEvent,
    SpineAnimationStateConfig, SpineAtlasAsset, SpineBounds, SpineDrawSignatureCache, SpineFlipY,
    SpineInstance, SpineInstanceKey, SpineInstanceParts, SpineLifecycleEvent,
    SpineLifecycleEventKind, SpineMeshChild, SpineReady, SpineReleaseReason, SpineRuntimeState,
    SpineSkeletonAsset, SpineSkeletonCommand, SpineSkeletonCommandKind, SpineSkeletonControl,
    SpineSkin, SpineTrackState, SpineTrackStateParts, SpineWorld,
};

type SpawnSpineQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Spine,
        Option<&'static SpineInstanceKey>,
        Option<&'static SpineAnimation>,
        Option<&'static SpineAnimationStateConfig>,
        Option<&'static SpineSkeletonControl>,
        Option<&'static SpineSkin>,
        Option<&'static SpineFlipY>,
        Option<&'static SpineDrawSignatureCache>,
    ),
    Or<(Without<SpineInstanceKey>, Changed<Spine>)>,
>;

type UpdateSpineQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SpineInstanceKey,
        Option<Ref<'static, SpineAnimation>>,
        Option<Ref<'static, SpineSkin>>,
        Option<&'static SpineFlipY>,
        Option<&'static mut SpineBounds>,
        Option<&'static mut SpineRuntimeState>,
    ),
>;

type SpineEntityQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Spine, Option<&'static SpineInstanceKey>)>;

type SpineKeyQuery<'w, 's> = Query<'w, 's, &'static SpineInstanceKey>;

type AnimationStateConfigQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SpineInstanceKey,
        Ref<'static, SpineAnimationStateConfig>,
    ),
>;

type SkeletonControlQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SpineInstanceKey,
        Ref<'static, SpineSkeletonControl>,
    ),
>;

#[derive(SystemParam)]
pub(crate) struct SpineMeshChildrenParam<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    mesh_children: Query<'w, 's, &'static SpineMeshChild>,
}

impl SpineMeshChildrenParam<'_, '_> {
    fn despawn(&self, commands: &mut Commands, entity: Entity) {
        despawn_mesh_children(commands, &self.children, &self.mesh_children, entity);
    }
}

pub fn spawn_spine_instances(
    mut commands: Commands,
    mut spine_world: NonSendMut<SpineWorld>,
    mut lifecycle_events: MessageWriter<SpineLifecycleEvent>,
    skeletons: Res<Assets<SpineSkeletonAsset>>,
    atlases: Res<Assets<SpineAtlasAsset>>,
    query: SpawnSpineQuery,
    mesh_children: SpineMeshChildrenParam,
) {
    for (
        entity,
        spine,
        existing_key,
        animation,
        animation_state_config,
        skeleton_control,
        skin,
        flip_y,
        draw_signature_cache,
    ) in &query
    {
        if existing_key.is_some() {
            if spine_world.remove_by_owner(entity).is_some() {
                lifecycle_events.write(SpineLifecycleEvent {
                    entity,
                    kind: SpineLifecycleEventKind::Released(SpineReleaseReason::ComponentChanged),
                });
            }
            mesh_children.despawn(&mut commands, entity);
            commands
                .entity(entity)
                .remove::<SpineInstanceKey>()
                .remove::<SpineReady>()
                .remove::<SpineBounds>()
                .remove::<SpineRuntimeState>()
                .remove::<SpineDrawSignatureCache>();
        }

        let Some(skeleton_asset) = skeletons.get(spine.get_skeleton()) else {
            continue;
        };
        let Some(atlas_asset) = atlases.get(spine.get_atlas()) else {
            continue;
        };

        let animation_component = animation.cloned().unwrap_or_else(|| {
            let mut component = SpineAnimation::default();
            component.set_name(spine.get_animation_name().map(str::to_owned));
            component.set_loop(spine.get_loop());
            component.set_time_scale(spine.get_time_scale());
            component
        });
        let skin_component = skin.cloned().unwrap_or_else(|| {
            let mut component = SpineSkin::default();
            component.set_name(spine.get_skin_name().map(str::to_owned));
            component
        });
        let skeleton_control = skeleton_control.copied().unwrap_or_default();
        let flip_y = flip_y.map(SpineFlipY::get_flip_y).unwrap_or(false);

        let skeleton_data = skeleton_asset.get_data().clone();
        let mut state_data = AnimationStateData::new(skeleton_data.clone());
        if let Some(animation_state_config) = animation_state_config {
            apply_animation_state_config(&mut state_data, animation_state_config, entity);
        }

        let mut skeleton = Skeleton::new(skeleton_data.clone());
        skeleton.set_skin(skin_component.get_name());
        apply_skeleton_control_to_skeleton(&mut skeleton, skeleton_control);
        skeleton.update_world_transform_with_physics(skeleton_control.get_physics());

        let mut instance = SpineInstance::new(
            SpineInstanceParts::new(
                skeleton,
                AnimationState::new(state_data),
                atlas_asset.get_atlas().clone(),
                atlas_asset.get_directory().to_owned(),
            )
            .with_animation_name(animation_component.get_name().map(str::to_owned))
            .with_loop(animation_component.get_loop())
            .with_time_scale(animation_component.get_time_scale())
            .with_skin_name(skin_component.get_name().map(str::to_owned))
            .with_flip_y(flip_y)
            .with_skeleton_control(skeleton_control),
        );
        instance.attach_event_listener();
        if let Some(animation_name) = animation_component.get_name() {
            instance.get_animation_state_mut().set_animation(
                0,
                animation_name,
                animation_component.get_loop(),
            );
        }
        rebuild_pose(&mut instance, 0.0);
        let _ = instance.drain_events();

        let new_bounds = draw_list_bounds(instance.get_draw_list(), instance.get_flip_y());
        let runtime_state = runtime_state_from_instance(&instance, new_bounds);
        let id = spine_world.insert(entity, instance);
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(SpineInstanceKey(id));
        entity_commands.insert(SpineReady);
        entity_commands.insert(new_bounds);
        entity_commands.insert(runtime_state);
        if existing_key.is_some() || draw_signature_cache.is_none() {
            entity_commands.insert(SpineDrawSignatureCache::default());
        }
        lifecycle_events.write(SpineLifecycleEvent {
            entity,
            kind: SpineLifecycleEventKind::Ready,
        });
    }
}

pub fn apply_spine_animation_state_config(
    mut spine_world: NonSendMut<SpineWorld>,
    query: AnimationStateConfigQuery,
) {
    for (entity, key, config) in &query {
        if !config.is_changed() {
            continue;
        }
        let Some(instance) = spine_world.get_mut(key.0) else {
            warn!("Missing Spine runtime instance for {entity:?}");
            continue;
        };

        apply_animation_state_config(
            instance.get_animation_state_mut().get_data(),
            &config,
            entity,
        );
    }
}

pub fn apply_spine_skeleton_control(
    mut spine_world: NonSendMut<SpineWorld>,
    query: SkeletonControlQuery,
) {
    for (entity, key, control) in &query {
        if !control.is_changed() {
            continue;
        }
        let Some(instance) = spine_world.get_mut(key.0) else {
            warn!("Missing Spine runtime instance for {entity:?}");
            continue;
        };

        let skeleton_control = *control;
        instance.set_skeleton_control(skeleton_control);
        apply_skeleton_control_to_skeleton(instance.get_skeleton_mut(), skeleton_control);
    }
}

pub fn cleanup_spine_instances(
    mut commands: Commands,
    mut spine_world: NonSendMut<SpineWorld>,
    mut lifecycle_events: MessageWriter<SpineLifecycleEvent>,
    mut removed_spines: RemovedComponents<Spine>,
    mesh_children: SpineMeshChildrenParam,
) {
    let mut removed = HashSet::new();
    removed.extend(removed_spines.read());

    for entity in removed {
        if spine_world.remove_by_owner(entity).is_some() {
            let kind = if commands.get_entity(entity).is_ok() {
                SpineLifecycleEventKind::Released(SpineReleaseReason::ComponentRemoved)
            } else {
                SpineLifecycleEventKind::Released(SpineReleaseReason::EntityDespawned)
            };
            lifecycle_events.write(SpineLifecycleEvent { entity, kind });
        }
        mesh_children.despawn(&mut commands, entity);
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands
                .remove::<SpineInstanceKey>()
                .remove::<SpineReady>()
                .remove::<SpineBounds>()
                .remove::<SpineRuntimeState>()
                .remove::<SpineDrawSignatureCache>();
        }
    }
}

pub fn reload_modified_spine_assets(
    mut commands: Commands,
    mut spine_world: NonSendMut<SpineWorld>,
    mut lifecycle_events: MessageWriter<SpineLifecycleEvent>,
    mut skeleton_events: MessageReader<AssetEvent<SpineSkeletonAsset>>,
    mut atlas_events: MessageReader<AssetEvent<SpineAtlasAsset>>,
    spine_query: SpineEntityQuery,
    mesh_children: SpineMeshChildrenParam,
) {
    let changed_skeletons = changed_asset_ids(skeleton_events.read());
    let changed_atlases = changed_asset_ids(atlas_events.read());
    if changed_skeletons.is_empty() && changed_atlases.is_empty() {
        return;
    }

    for (entity, spine, key) in &spine_query {
        if key.is_none() {
            continue;
        }
        if !changed_skeletons.contains(&spine.get_skeleton().id())
            && !changed_atlases.contains(&spine.get_atlas().id())
        {
            continue;
        }

        if spine_world.remove_by_owner(entity).is_some() {
            lifecycle_events.write(SpineLifecycleEvent {
                entity,
                kind: SpineLifecycleEventKind::Released(SpineReleaseReason::AssetReload),
            });
        }
        mesh_children.despawn(&mut commands, entity);
        commands
            .entity(entity)
            .remove::<SpineInstanceKey>()
            .remove::<SpineReady>()
            .remove::<SpineBounds>()
            .remove::<SpineRuntimeState>()
            .remove::<SpineDrawSignatureCache>();
    }
}

pub fn apply_spine_animation_commands(
    mut spine_world: NonSendMut<SpineWorld>,
    mut commands: MessageReader<SpineAnimationCommand>,
    key_query: SpineKeyQuery,
) {
    for message in commands.read() {
        let Ok(key) = key_query.get(message.entity) else {
            continue;
        };
        let Some(instance) = spine_world.get_mut(key.0) else {
            warn!("Missing Spine runtime instance for {:?}", message.entity);
            continue;
        };

        match &message.command {
            SpineAnimationCommandKind::Set {
                track_index,
                animation,
                looped,
                settings,
            } => {
                let handle = instance.get_animation_state_mut().set_animation(
                    *track_index,
                    animation,
                    *looped,
                );
                settings.apply(instance.get_animation_state_mut(), handle);
                if *track_index == 0 {
                    instance.set_animation_name(Some(animation.clone()));
                    instance.set_loop(*looped);
                }
            }
            SpineAnimationCommandKind::Add {
                track_index,
                animation,
                looped,
                delay,
                settings,
            } => {
                let handle = instance.get_animation_state_mut().add_animation(
                    *track_index,
                    animation,
                    *looped,
                    *delay,
                );
                settings.apply(instance.get_animation_state_mut(), handle);
            }
            SpineAnimationCommandKind::SetEmpty {
                track_index,
                mix_duration,
                settings,
            } => {
                let handle = instance
                    .get_animation_state_mut()
                    .set_empty_animation(*track_index, *mix_duration);
                settings.apply(instance.get_animation_state_mut(), handle);
                if *track_index == 0 {
                    instance.set_animation_name(None);
                    instance.set_loop(false);
                }
            }
            SpineAnimationCommandKind::AddEmpty {
                track_index,
                mix_duration,
                delay,
                settings,
            } => {
                let handle = instance.get_animation_state_mut().add_empty_animation(
                    *track_index,
                    *mix_duration,
                    *delay,
                );
                settings.apply(instance.get_animation_state_mut(), handle);
            }
            SpineAnimationCommandKind::SetEmptyAnimations { mix_duration } => {
                instance
                    .get_animation_state_mut()
                    .set_empty_animations(*mix_duration);
                instance.set_animation_name(None);
                instance.set_loop(false);
            }
            SpineAnimationCommandKind::ClearTrack { track_index } => {
                instance.get_animation_state_mut().clear_track(*track_index);
                if *track_index == 0 {
                    instance.set_animation_name(None);
                    instance.set_loop(false);
                }
            }
            SpineAnimationCommandKind::ClearTracks => {
                instance.get_animation_state_mut().clear_tracks();
                instance.set_animation_name(None);
                instance.set_loop(false);
            }
            SpineAnimationCommandKind::SetDefaultMix { default_mix } => {
                apply_default_mix(instance.get_animation_state_mut().get_data(), *default_mix);
            }
            SpineAnimationCommandKind::SetMix { from, to, duration } => {
                apply_animation_mix(
                    instance.get_animation_state_mut().get_data(),
                    from,
                    to,
                    *duration,
                    message.entity,
                );
            }
            SpineAnimationCommandKind::ClearMixes => {
                instance.get_animation_state_mut().get_data().clear();
            }
        }
    }
}

pub fn apply_spine_skeleton_commands(
    mut spine_world: NonSendMut<SpineWorld>,
    mut commands: MessageReader<SpineSkeletonCommand>,
    key_query: SpineKeyQuery,
) {
    for message in commands.read() {
        let Ok(key) = key_query.get(message.entity) else {
            continue;
        };
        let Some(instance) = spine_world.get_mut(key.0) else {
            warn!("Missing Spine runtime instance for {:?}", message.entity);
            continue;
        };

        match message.command {
            SpineSkeletonCommandKind::SetControl(control) => {
                instance.set_skeleton_control(control);
                apply_skeleton_control_to_skeleton(instance.get_skeleton_mut(), control);
            }
            SpineSkeletonCommandKind::SetPhysics(physics) => {
                let mut skeleton_control = instance.get_skeleton_control();
                skeleton_control.set_physics(physics);
                instance.set_skeleton_control(skeleton_control);
            }
            SpineSkeletonCommandKind::SetWind(wind) => {
                let mut skeleton_control = instance.get_skeleton_control();
                skeleton_control.set_wind(wind);
                instance.set_skeleton_control(skeleton_control);
                instance.get_skeleton_mut().set_wind_x(wind.x);
                instance.get_skeleton_mut().set_wind_y(wind.y);
            }
            SpineSkeletonCommandKind::SetGravity(gravity) => {
                let mut skeleton_control = instance.get_skeleton_control();
                skeleton_control.set_gravity(gravity);
                instance.set_skeleton_control(skeleton_control);
                instance.get_skeleton_mut().set_gravity_x(gravity.x);
                instance.get_skeleton_mut().set_gravity_y(gravity.y);
            }
            SpineSkeletonCommandKind::SetTime(time) => {
                let mut skeleton_control = instance.get_skeleton_control();
                skeleton_control.set_time(time);
                instance.set_skeleton_control(skeleton_control);
                instance.get_skeleton_mut().set_time(time);
            }
            SpineSkeletonCommandKind::SetupPose => {
                instance.get_skeleton_mut().setup_pose();
            }
        }
    }
}

pub fn update_spine_animations(
    mut commands: Commands,
    mut spine_world: NonSendMut<SpineWorld>,
    mut animation_events: MessageWriter<SpineAnimationEvent>,
    mut query: UpdateSpineQuery,
    time: Res<Time>,
) {
    for (entity, key, animation_ref, skin_ref, flip_y, bounds, runtime_state) in &mut query {
        let Some(instance) = spine_world.get_mut(key.0) else {
            warn!("Missing Spine runtime instance for {entity:?}");
            continue;
        };

        if let Some(skin_ref) = skin_ref
            && skin_ref.is_changed()
            && instance.get_skin_name() != skin_ref.get_name()
        {
            instance.get_skeleton_mut().set_skin(skin_ref.get_name());
            instance.set_skin_name(skin_ref.get_name().map(str::to_owned));
        }

        if let Some(animation_ref) = animation_ref {
            instance.set_time_scale(animation_ref.get_time_scale());
            if animation_ref.is_changed()
                && (instance.get_animation_name() != animation_ref.get_name()
                    || instance.get_loop() != animation_ref.get_loop())
            {
                instance.get_animation_state_mut().clear_tracks();
                if let Some(animation_name) = animation_ref.get_name() {
                    instance.get_animation_state_mut().set_animation(
                        0,
                        animation_name,
                        animation_ref.get_loop(),
                    );
                }
                instance.set_animation_name(animation_ref.get_name().map(str::to_owned));
                instance.set_loop(animation_ref.get_loop());
            }
        }

        let new_flip_y = flip_y.map(SpineFlipY::get_flip_y).unwrap_or(false);
        if instance.get_flip_y() != new_flip_y {
            instance.set_flip_y(new_flip_y);
        }
        rebuild_pose(
            instance,
            time.delta().as_secs_f32() * instance.get_time_scale(),
        );
        let new_bounds = draw_list_bounds(instance.get_draw_list(), instance.get_flip_y());
        if let Some(mut bounds) = bounds {
            *bounds = new_bounds;
        }
        let new_runtime_state = runtime_state_from_instance(instance, new_bounds);
        if let Some(mut runtime_state) = runtime_state {
            *runtime_state = new_runtime_state;
        } else {
            commands.entity(entity).insert(new_runtime_state);
        }
        for event in instance.drain_events() {
            animation_events.write(SpineAnimationEvent {
                entity,
                track_index: event.track_index,
                animation_name: event.animation_name,
                track_time: event.track_time,
                kind: event.kind,
            });
        }
    }
}

fn changed_asset_ids<'a, A: Asset>(
    events: impl Iterator<Item = &'a AssetEvent<A>>,
) -> HashSet<AssetId<A>> {
    events
        .filter_map(|event| match event {
            AssetEvent::Modified { id } | AssetEvent::Removed { id } => Some(*id),
            AssetEvent::Added { .. }
            | AssetEvent::Unused { .. }
            | AssetEvent::LoadedWithDependencies { .. } => None,
        })
        .collect()
}

fn rebuild_pose(instance: &mut SpineInstance, delta: f32) {
    instance.rebuild_pose(delta);
}

fn apply_skeleton_control_to_skeleton(skeleton: &mut Skeleton, control: SpineSkeletonControl) {
    skeleton.set_wind_x(control.get_wind().x);
    skeleton.set_wind_y(control.get_wind().y);
    skeleton.set_gravity_x(control.get_gravity().x);
    skeleton.set_gravity_y(control.get_gravity().y);
    if let Some(time) = control.get_time() {
        skeleton.set_time(time);
    }
}

fn runtime_state_from_instance(instance: &SpineInstance, bounds: SpineBounds) -> SpineRuntimeState {
    let wind_x = instance.get_skeleton().get_wind_x();
    let wind_y = instance.get_skeleton().get_wind_y();
    let gravity_x = instance.get_skeleton().get_gravity_x();
    let gravity_y = instance.get_skeleton().get_gravity_y();
    let animation_state = instance.get_animation_state();
    SpineRuntimeState::new(
        true,
        animation_state
            .get_tracks()
            .into_iter()
            .filter_map(|track| {
                track?.entry(animation_state).map(|entry| {
                    SpineTrackState::new(SpineTrackStateParts {
                        track_index: entry.get_track_index(),
                        animation_name: entry.get_animation().get_name().to_string(),
                        track_time: entry.get_track_time(),
                        animation_time: entry.get_animation_time(),
                        looped: entry.get_loop(),
                        delay: entry.get_delay(),
                        mix_duration: entry.get_mix_duration(),
                        mix_time: entry.get_mix_time(),
                        alpha: entry.get_alpha(),
                        additive: entry.get_additive(),
                        mix_interpolation: entry.get_mix_interpolation(),
                        reverse: entry.get_reverse(),
                    })
                })
            })
            .collect(),
        instance.get_skeleton().get_time(),
        instance.get_skeleton_control().get_physics(),
        Vec2::new(wind_x, wind_y),
        Vec2::new(gravity_x, gravity_y),
        bounds,
    )
}

fn apply_animation_state_config(
    state_data: &mut AnimationStateData,
    config: &SpineAnimationStateConfig,
    entity: Entity,
) {
    apply_default_mix(state_data, config.get_default_mix());
    for mix in config.get_mixes() {
        apply_animation_mix(
            state_data,
            mix.get_from(),
            mix.get_to(),
            mix.get_duration(),
            entity,
        );
    }
}

fn apply_default_mix(state_data: &mut AnimationStateData, default_mix: f32) {
    state_data.set_default_mix(default_mix);
}

fn apply_animation_mix(
    state_data: &mut AnimationStateData,
    from: &str,
    to: &str,
    duration: f32,
    entity: Entity,
) {
    let _ = entity;
    state_data.set_mix(from, to, duration);
}

fn draw_list_bounds(draw_list: &spine2d::DrawList, flip_y: bool) -> SpineBounds {
    let Some(first) = draw_list.vertices.first() else {
        return SpineBounds::new(Vec2::ZERO, Vec2::ZERO);
    };
    let first_position = Vec2::new(first.position[0], first.position[1]);
    let mut min = Vec2::new(
        first_position.x,
        if flip_y {
            -first_position.y
        } else {
            first_position.y
        },
    );
    let mut max = min;

    for vertex in draw_list.vertices.iter().skip(1) {
        let position = Vec2::new(
            vertex.position[0],
            if flip_y {
                -vertex.position[1]
            } else {
                vertex.position[1]
            },
        );
        min = min.min(position);
        max = max.max(position);
    }

    SpineBounds::new(min, max)
}

#[cfg(test)]
mod tests {
    use super::render::texture_asset_path;
    use super::*;
    use crate::{
        SpineAnimationEventKind, SpineDrawSignature, SpineRenderSignature, SpineTrackEntrySettings,
        materials::{
            SpineAdditiveMaterial, SpineAdditivePmaMaterial, SpineMaterialCache,
            SpineMultiplyMaterial, SpineMultiplyPmaMaterial, SpineNormalMaterial,
            SpineNormalPmaMaterial, SpineScreenMaterial, SpineScreenPmaMaterial,
        },
    };
    use bevy::app::TaskPoolPlugin;
    use bevy::asset::{AssetEventSystems, AssetMetaCheck, AssetPlugin, UnapprovedPathMode};
    use bevy::camera::visibility::RenderLayers;
    use bevy::ecs::message::Messages;
    use spine2d::{Atlas, BlendMode, SkeletonData};
    use std::time::Duration;

    fn app_with_lifecycle_systems() -> App {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            },
        ))
        .init_asset::<SpineSkeletonAsset>()
        .init_asset::<SpineAtlasAsset>()
        .add_message::<SpineLifecycleEvent>()
        .add_message::<SpineAnimationEvent>()
        .add_message::<SpineAnimationCommand>()
        .add_message::<SpineSkeletonCommand>()
        .init_resource::<Time>()
        .insert_non_send(SpineWorld::new())
        .add_systems(
            Update,
            (
                cleanup_spine_instances,
                spawn_spine_instances,
                apply_spine_animation_state_config,
                apply_spine_skeleton_control,
                apply_spine_skeleton_commands,
                apply_spine_animation_commands,
                update_spine_animations,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            reload_modified_spine_assets.after(AssetEventSystems),
        );
        app
    }

    fn app_with_render_systems() -> App {
        let mut app = app_with_lifecycle_systems();
        app.init_asset::<Image>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<SpineNormalMaterial>>()
            .init_resource::<Assets<SpineAdditiveMaterial>>()
            .init_resource::<Assets<SpineMultiplyMaterial>>()
            .init_resource::<Assets<SpineScreenMaterial>>()
            .init_resource::<Assets<SpineNormalPmaMaterial>>()
            .init_resource::<Assets<SpineAdditivePmaMaterial>>()
            .init_resource::<Assets<SpineMultiplyPmaMaterial>>()
            .init_resource::<Assets<SpineScreenPmaMaterial>>()
            .init_resource::<SpineMaterialCache>()
            .add_systems(Update, render_spines);
        app
    }

    fn demo_handles(app: &mut App) -> (Handle<SpineSkeletonAsset>, Handle<SpineAtlasAsset>) {
        let skeleton_data =
            SkeletonData::from_json_str(include_str!("../../spine2d-web/assets/demo.json"))
                .expect("parse demo skeleton");
        let atlas = (include_str!("../../spine2d-web/assets/demo.atlas"))
            .parse::<Atlas>()
            .expect("parse demo atlas");

        let skeleton = app
            .world_mut()
            .resource_mut::<Assets<SpineSkeletonAsset>>()
            .add(SpineSkeletonAsset::new(skeleton_data));
        let atlas = app
            .world_mut()
            .resource_mut::<Assets<SpineAtlasAsset>>()
            .add(SpineAtlasAsset::new(atlas, "spine2d-web/assets".to_owned()));

        (skeleton, atlas)
    }

    fn event_handles(app: &mut App) -> (Handle<SpineSkeletonAsset>, Handle<SpineAtlasAsset>) {
        let skeleton_data = SkeletonData::from_json_str(
            r#"
            {
              "skeleton": { "spine": "4.3.00" },
              "bones": [ { "name": "root" } ],
              "slots": [ { "name": "slot0", "bone": "root", "attachment": "mesh0" } ],
              "skins": {
                "default": {
                  "slot0": {
                    "mesh0": {
                      "type": "mesh",
                      "path": "mesh0",
                      "uvs": [0,0, 1,0, 1,1, 0,1],
                      "vertices": [-1,-1, 1,-1, 1,1, -1,1],
                      "triangles": [0,1,2, 2,3,0]
                    }
                  }
                }
              },
              "events": { "hit": { "int": 7, "float": 1.5, "string": "default" } },
              "animations": {
                "first": {
                  "events": [
                    { "time": 0.1, "name": "hit", "string": "impact" }
                  ]
                },
                "second": {}
              }
            }
            "#,
        )
        .expect("parse event skeleton");
        let atlas = (include_str!("../../spine2d-web/assets/demo.atlas"))
            .parse::<Atlas>()
            .expect("parse demo atlas");

        let skeleton = app
            .world_mut()
            .resource_mut::<Assets<SpineSkeletonAsset>>()
            .add(SpineSkeletonAsset::new(skeleton_data));
        let atlas = app
            .world_mut()
            .resource_mut::<Assets<SpineAtlasAsset>>()
            .add(SpineAtlasAsset::new(atlas, "spine2d-web/assets".to_owned()));

        (skeleton, atlas)
    }

    fn animation_component(name: Option<&str>, looped: bool, time_scale: f32) -> SpineAnimation {
        let mut component = SpineAnimation::default();
        component.set_name(name);
        component.set_loop(looped);
        component.set_time_scale(time_scale);
        component
    }

    fn skin_component(name: Option<&str>) -> SpineSkin {
        let mut component = SpineSkin::default();
        component.set_name(name);
        component
    }

    fn drain_animation_events(app: &mut App) -> Vec<SpineAnimationEvent> {
        app.world_mut()
            .resource_mut::<Messages<SpineAnimationEvent>>()
            .drain()
            .collect()
    }

    fn drain_lifecycle_events(app: &mut App) -> Vec<SpineLifecycleEvent> {
        app.world_mut()
            .resource_mut::<Messages<SpineLifecycleEvent>>()
            .drain()
            .collect()
    }

    fn mesh_child_entities(app: &App, entity: Entity) -> Vec<Entity> {
        app.world()
            .get::<Children>(entity)
            .map(|children| {
                children
                    .iter()
                    .filter(|child| app.world().get::<SpineMeshChild>(*child).is_some())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn mesh_child_handles(app: &App, entity: Entity) -> Vec<Handle<Mesh>> {
        mesh_child_entities(app, entity)
            .into_iter()
            .filter_map(|child| {
                app.world()
                    .get::<SpineMeshChild>(child)
                    .map(|mesh_child| mesh_child.get_mesh().clone())
            })
            .collect()
    }

    fn current_track_mix_duration(app: &App, entity: Entity) -> f32 {
        current_track_entry(app, entity, 0, |entry| entry.get_mix_duration())
    }

    fn current_track_entry<F: FnOnce(&spine2d::TrackEntry) -> R, R>(
        app: &App,
        entity: Entity,
        track_index: usize,
        f: F,
    ) -> R {
        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let state = spine_world.get(key.0).unwrap().get_animation_state();
        state
            .get_track(track_index)
            .and_then(|handle| handle.entry(state).map(f))
            .unwrap()
    }

    fn queued_track_entry<F: FnOnce(&spine2d::TrackEntry) -> R, R>(
        app: &App,
        entity: Entity,
        track_index: usize,
        queue_index: usize,
        f: F,
    ) -> R {
        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let state = spine_world.get(key.0).unwrap().get_animation_state();
        let mut handle = state
            .get_track(track_index)
            .unwrap()
            .get_next(state)
            .unwrap();
        for _ in 0..queue_index {
            handle = handle.get_next(state).unwrap();
        }
        handle.entry(state).map(f).unwrap()
    }

    fn spine_instance<F: FnOnce(&SpineInstance) -> R, R>(app: &App, entity: Entity, f: F) -> R {
        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        f(spine_world.get(key.0).unwrap())
    }

    fn animation_state_default_mix(app: &mut App, entity: Entity) -> f32 {
        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        app.world_mut()
            .non_send_mut::<SpineWorld>()
            .get_mut(key.0)
            .unwrap()
            .get_animation_state_mut()
            .get_data()
            .get_default_mix()
    }

    #[test]
    fn spawn_adds_only_internal_runtime_components_after_assets_are_ready() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("spin", true))
            .id();

        app.update();

        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);
        assert!(app.world().get::<SpineInstanceKey>(entity).is_some());
        assert!(app.world().get::<SpineReady>(entity).is_some());
        assert!(app.world().get::<SpineDrawSignatureCache>(entity).is_some());
        assert_eq!(
            *app.world().get::<SpineBounds>(entity).unwrap(),
            SpineBounds::new(Vec2::new(-128.0, -128.0), Vec2::new(128.0, 128.0))
        );
        assert!(app.world().get::<SpineAnimation>(entity).is_none());
        assert!(app.world().get::<SpineSkin>(entity).is_none());

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.get_animation_name(), Some("spin"));
        assert!(instance.get_loop());
        assert_eq!(instance.get_time_scale(), 1.0);

        assert_eq!(
            drain_lifecycle_events(&mut app),
            vec![SpineLifecycleEvent {
                entity,
                kind: SpineLifecycleEventKind::Ready,
            }]
        );
    }

    #[test]
    fn spawn_preserves_user_animation_and_skin_components() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas)
                    .with_animation("spin", true)
                    .with_skin("default"),
                animation_component(None, false, 2.0),
                skin_component(None),
            ))
            .id();

        app.update();

        let animation = app.world().get::<SpineAnimation>(entity).unwrap();
        assert_eq!(animation.get_name(), None);
        assert!(!animation.get_loop());
        assert_eq!(animation.get_time_scale(), 2.0);
        assert_eq!(
            app.world().get::<SpineSkin>(entity).unwrap().get_name(),
            None
        );

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.get_animation_name(), None);
        assert!(!instance.get_loop());
        assert_eq!(instance.get_skin_name(), None);
    }

    #[test]
    fn spawn_preserves_existing_draw_signature_cache() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);
        let signature = SpineDrawSignature::from_draw(
            &spine2d::Draw {
                texture_path: "old.png".to_owned(),
                blend: BlendMode::Normal,
                premultiplied_alpha: false,
                first_index: 0,
                index_count: 0,
            },
            "old.png".to_owned(),
        );

        let entity = app
            .world_mut()
            .spawn((Spine::new(skeleton, atlas), {
                let mut cache = SpineDrawSignatureCache::default();
                cache.set_signature(SpineRenderSignature::new(vec![signature.clone()], None));
                cache
            }))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .get_signature()
                .get_draws(),
            vec![signature]
        );
    }

    #[test]
    fn texture_asset_path_includes_atlas_directory() {
        assert_eq!(texture_asset_path("", "page.png"), "page.png");
        assert_eq!(
            texture_asset_path("spineboy/export", "page.png"),
            "spineboy/export/page.png"
        );
    }

    #[test]
    fn draw_list_bounds_preserve_spine_y_axis() {
        let draw_list = spine2d::DrawList {
            vertices: vec![
                spine2d::Vertex {
                    position: [1.0, 2.0],
                    uv: [0.0, 0.0],
                    color: [1.0; 4],
                    dark_color: [0.0; 4],
                },
                spine2d::Vertex {
                    position: [4.0, 5.0],
                    uv: [1.0, 1.0],
                    color: [1.0; 4],
                    dark_color: [0.0; 4],
                },
            ],
            indices: vec![0, 1],
            draws: Vec::new(),
        };

        assert_eq!(
            draw_list_bounds(&draw_list, false),
            SpineBounds::new(Vec2::new(1.0, 2.0), Vec2::new(4.0, 5.0))
        );
    }

    #[test]
    fn draw_list_bounds_flip_y_when_requested() {
        let draw_list = spine2d::DrawList {
            vertices: vec![
                spine2d::Vertex {
                    position: [1.0, 2.0],
                    uv: [0.0, 0.0],
                    color: [1.0; 4],
                    dark_color: [0.0; 4],
                },
                spine2d::Vertex {
                    position: [4.0, 5.0],
                    uv: [1.0, 1.0],
                    color: [1.0; 4],
                    dark_color: [0.0; 4],
                },
            ],
            indices: vec![0, 1],
            draws: Vec::new(),
        };

        assert_eq!(
            draw_list_bounds(&draw_list, true),
            SpineBounds::new(Vec2::new(1.0, -5.0), Vec2::new(4.0, -2.0))
        );
    }

    #[test]
    fn spawn_copies_flip_y_into_runtime_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((Spine::new(skeleton, atlas), SpineFlipY::flipped()))
            .id();

        app.update();

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert!(instance.get_flip_y());
    }

    #[test]
    fn render_signature_tracks_parent_render_layers() {
        let mut app = app_with_render_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("spin", true),
                RenderLayers::layer(2),
            ))
            .id();

        app.update();
        app.update();

        assert_eq!(
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .get_signature()
                .get_render_layers(),
            Some(&RenderLayers::layer(2))
        );
        let mesh_handles_before = mesh_child_handles(&app, entity);
        assert!(!mesh_handles_before.is_empty());
        for child in mesh_child_entities(&app, entity) {
            assert_eq!(
                app.world().get::<RenderLayers>(child),
                Some(&RenderLayers::layer(2))
            );
        }

        app.world_mut()
            .entity_mut(entity)
            .insert(RenderLayers::layer(5));
        app.update();

        assert_eq!(
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .get_signature()
                .get_render_layers(),
            Some(&RenderLayers::layer(5))
        );
        assert_eq!(mesh_child_handles(&app, entity), mesh_handles_before);
        for child in mesh_child_entities(&app, entity) {
            assert_eq!(
                app.world().get::<RenderLayers>(child),
                Some(&RenderLayers::layer(5))
            );
        }

        app.world_mut().entity_mut(entity).remove::<RenderLayers>();
        app.update();

        assert_eq!(
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .get_signature()
                .get_render_layers(),
            None
        );
        assert_eq!(mesh_child_handles(&app, entity), mesh_handles_before);
        for child in mesh_child_entities(&app, entity) {
            assert!(app.world().get::<RenderLayers>(child).is_none());
        }
    }

    #[test]
    fn render_rebuilds_when_mesh_children_are_missing() {
        let mut app = app_with_render_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("spin", true))
            .id();

        app.update();
        app.update();

        let child = {
            let children = app.world().get::<Children>(entity).unwrap();
            children
                .iter()
                .find(|child| app.world().get::<SpineMeshChild>(*child).is_some())
                .expect("spine mesh child exists")
        };
        app.world_mut().entity_mut(child).despawn();
        app.update();

        let mesh_child_count = app
            .world()
            .get::<Children>(entity)
            .unwrap()
            .iter()
            .filter(|child| app.world().get::<SpineMeshChild>(*child).is_some())
            .count();

        assert_eq!(
            mesh_child_count,
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .get_signature()
                .get_draws()
                .len()
        );
    }

    #[test]
    fn render_reuses_materials_when_mesh_children_are_rebuilt() {
        let mut app = app_with_render_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("spin", true))
            .id();

        app.update();
        app.update();

        let cache_len = app.world().resource::<SpineMaterialCache>().len();
        let normal_material_len = app.world().resource::<Assets<SpineNormalMaterial>>().len();
        let child = {
            let children = app.world().get::<Children>(entity).unwrap();
            children
                .iter()
                .find(|child| app.world().get::<SpineMeshChild>(*child).is_some())
                .expect("spine mesh child exists")
        };

        app.world_mut().entity_mut(child).despawn();
        app.update();

        assert_eq!(
            app.world().resource::<SpineMaterialCache>().len(),
            cache_len
        );
        assert_eq!(
            app.world().resource::<Assets<SpineNormalMaterial>>().len(),
            normal_material_len
        );
    }

    #[test]
    fn changing_spine_component_rebuilds_runtime_instance_and_mesh_children() {
        let mut app = app_with_render_systems();
        let (first_skeleton, first_atlas) = demo_handles(&mut app);
        let (second_skeleton, second_atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(first_skeleton, first_atlas).with_animation("spin", true))
            .id();

        app.update();
        app.update();
        drain_lifecycle_events(&mut app);

        let old_key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let old_mesh_children = mesh_child_entities(&app, entity);
        assert!(!old_mesh_children.is_empty());
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);

        app.world_mut().entity_mut(entity).insert(
            Spine::new(second_skeleton, second_atlas)
                .with_animation("spin", true)
                .with_skin("default"),
        );
        app.update();
        app.update();

        let new_key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let new_mesh_children = mesh_child_entities(&app, entity);
        assert_ne!(old_key, new_key);
        assert!(!new_mesh_children.is_empty());
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);
        for child in old_mesh_children {
            assert!(app.world().get_entity(child).is_err());
        }

        let lifecycle_events = drain_lifecycle_events(&mut app);
        assert_eq!(
            lifecycle_events,
            vec![
                SpineLifecycleEvent {
                    entity,
                    kind: SpineLifecycleEventKind::Released(SpineReleaseReason::ComponentChanged),
                },
                SpineLifecycleEvent {
                    entity,
                    kind: SpineLifecycleEventKind::Ready,
                },
            ]
        );
    }

    #[test]
    fn changing_spine_component_to_unloaded_assets_releases_old_runtime() {
        let mut app = app_with_render_systems();
        let (first_skeleton, first_atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(first_skeleton, first_atlas).with_animation("spin", true))
            .id();

        app.update();
        app.update();
        drain_lifecycle_events(&mut app);

        let old_mesh_children = mesh_child_entities(&app, entity);
        assert!(!old_mesh_children.is_empty());
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);

        app.world_mut().entity_mut(entity).insert(
            Spine::new(
                Handle::<SpineSkeletonAsset>::default(),
                Handle::<SpineAtlasAsset>::default(),
            )
            .with_animation("missing", true),
        );
        app.update();

        assert!(app.world().get::<SpineInstanceKey>(entity).is_none());
        assert!(app.world().get::<SpineReady>(entity).is_none());
        assert!(app.world().get::<SpineBounds>(entity).is_none());
        assert!(app.world().get::<SpineDrawSignatureCache>(entity).is_none());
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 0);
        for child in old_mesh_children {
            assert!(app.world().get_entity(child).is_err());
        }
        assert_eq!(
            drain_lifecycle_events(&mut app),
            vec![SpineLifecycleEvent {
                entity,
                kind: SpineLifecycleEventKind::Released(SpineReleaseReason::ComponentChanged),
            }]
        );
    }

    #[test]
    fn update_writes_spine_animation_event_messages() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", false))
            .id();

        app.update();
        assert!(drain_animation_events(&mut app).is_empty());

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(100));
        app.update();

        let events = drain_animation_events(&mut app);
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.animation_name == "first"
                && matches!(
                    &event.kind,
                    SpineAnimationEventKind::Event(spine_event)
                        if spine_event.get_data().get_name() == "hit"
                            && spine_event.get_int() == 7
                            && spine_event.get_string() == "impact"
                )
        }));
    }

    #[test]
    fn update_respects_negative_animation_time_scale() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();

        app.update();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(100));
        app.update();

        let forward_time = current_track_entry(&app, entity, 0, |entry| entry.get_track_time());
        assert!(forward_time > 0.0);

        app.world_mut()
            .entity_mut(entity)
            .insert(animation_component(Some("first"), true, -1.0));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(100));
        app.update();

        let reversed_time = current_track_entry(&app, entity, 0, |entry| entry.get_track_time());
        assert!(reversed_time < forward_time);
    }

    #[test]
    fn animation_component_change_writes_control_messages() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();

        app.update();
        drain_animation_events(&mut app);

        app.world_mut()
            .entity_mut(entity)
            .insert(animation_component(Some("second"), false, 1.0));
        app.update();

        let events = drain_animation_events(&mut app);
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.animation_name == "first"
                && matches!(
                    event.kind,
                    SpineAnimationEventKind::End | SpineAnimationEventKind::Dispose
                )
        }));
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.animation_name == "second"
                && matches!(event.kind, SpineAnimationEventKind::Start)
        }));
    }

    #[test]
    fn animation_command_sets_animation_without_public_runtime_handle() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app.world_mut().spawn(Spine::new(skeleton, atlas)).id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::set(entity, 0, "first", true));
        app.update();

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.get_animation_name(), Some("first"));
        assert!(instance.get_loop());

        let events = drain_animation_events(&mut app);
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.animation_name == "first"
                && matches!(event.kind, SpineAnimationEventKind::Start)
        }));
    }

    #[test]
    fn animation_state_config_applies_spawn_mix_settings() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("first", true),
                SpineAnimationStateConfig::new()
                    .with_default_mix(0.35)
                    .with_mix("first", "second", 0.2),
            ))
            .id();
        app.update();

        assert_eq!(animation_state_default_mix(&mut app, entity), 0.35);

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::set(entity, 0, "second", true));
        app.update();

        assert_eq!(current_track_mix_duration(&app, entity), 0.2);
    }

    #[test]
    fn changed_animation_state_config_updates_existing_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("first", true),
                SpineAnimationStateConfig::new().with_default_mix(0.1),
            ))
            .id();
        app.update();
        assert_eq!(animation_state_default_mix(&mut app, entity), 0.1);

        app.world_mut()
            .entity_mut(entity)
            .insert(SpineAnimationStateConfig::new().with_default_mix(0.45));
        app.update();

        assert_eq!(animation_state_default_mix(&mut app, entity), 0.45);
        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::set(entity, 0, "second", true));
        app.update();

        assert_eq!(current_track_mix_duration(&app, entity), 0.45);
    }

    #[test]
    fn animation_state_commands_override_changed_config_in_same_frame() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("first", true),
                SpineAnimationStateConfig::new().with_default_mix(0.1),
            ))
            .id();
        app.update();

        app.world_mut()
            .entity_mut(entity)
            .insert(SpineAnimationStateConfig::new().with_default_mix(0.2));
        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::set_default_mix(entity, 0.5));
        app.update();

        assert_eq!(animation_state_default_mix(&mut app, entity), 0.5);
    }

    #[test]
    fn animation_state_config_commands_configure_existing_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        {
            let mut messages = app
                .world_mut()
                .resource_mut::<Messages<SpineAnimationCommand>>();
            messages.write(SpineAnimationCommand::set_default_mix(entity, 0.35));
            messages.write(SpineAnimationCommand::set_mix(
                entity, "first", "second", 0.2,
            ));
            messages.write(SpineAnimationCommand::set(entity, 0, "second", true));
        }
        app.update();

        assert_eq!(animation_state_default_mix(&mut app, entity), 0.35);
        assert_eq!(current_track_mix_duration(&app, entity), 0.2);
    }

    #[test]
    fn set_animation_command_applies_track_entry_settings_to_current_entry() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app.world_mut().spawn(Spine::new(skeleton, atlas)).id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(
                SpineAnimationCommand::set(entity, 0, "first", true).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_alpha(0.5)
                        .with_loop(false)
                        .with_additive(true)
                        .with_mix_interpolation(spine2d::MixInterpolation::Smooth)
                        .with_reverse(true)
                        .with_shortest_rotation(true)
                        .with_mix_duration(0.25)
                        .with_event_threshold(0.75),
                ),
            );
        app.update();

        current_track_entry(&app, entity, 0, |entry| {
            assert_eq!(entry.get_animation().get_name(), "first");
            assert_eq!(entry.get_alpha(), 0.5);
            assert!(!entry.get_loop());
            assert!(entry.get_additive());
            assert_eq!(
                entry.get_mix_interpolation(),
                spine2d::MixInterpolation::Smooth
            );
            assert!(entry.get_reverse());
            assert!(entry.get_shortest_rotation());
            assert_eq!(entry.get_mix_duration(), 0.25);
            assert_eq!(entry.get_event_threshold(), 0.75);
        });
    }

    #[test]
    fn add_animation_command_applies_track_entry_settings_to_queued_entry() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(
                SpineAnimationCommand::add(entity, 0, "second", false, 0.0).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_delay(0.3)
                        .with_track_end(0.8)
                        .with_mix_duration(0.4)
                        .with_animation_start(0.1)
                        .with_animation_end(0.9)
                        .with_animation_last(0.2),
                ),
            );
        app.update();

        queued_track_entry(&app, entity, 0, 0, |entry| {
            assert_eq!(entry.get_animation().get_name(), "second");
            assert_eq!(entry.get_delay(), 0.3);
            assert_eq!(entry.get_track_end(), 0.8);
            assert_eq!(entry.get_mix_duration(), 0.4);
            assert_eq!(entry.get_animation_start(), 0.1);
            assert_eq!(entry.get_animation_end(), 0.9);
            assert_eq!(entry.get_animation_last(), 0.2);
        });
    }

    #[test]
    fn add_animation_settings_adjust_queued_delay_with_mix_duration() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        let previous_duration = current_track_entry(&app, entity, 0, |entry| {
            entry.get_animation().get_duration()
        });
        let mix_duration = previous_duration * 0.25;
        let expected_delay = previous_duration - mix_duration;

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(
                SpineAnimationCommand::add(entity, 0, "second", false, 0.0).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_delay(0.0)
                        .with_mix_duration(mix_duration),
                ),
            );
        app.update();

        queued_track_entry(&app, entity, 0, 0, |entry| {
            assert!((entry.get_delay() - expected_delay).abs() <= 0.0001);
            assert!((entry.get_mix_duration() - mix_duration).abs() <= 0.0001);
        });
    }

    #[test]
    fn empty_animation_command_accepts_track_entry_settings() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(
                SpineAnimationCommand::set_empty(entity, 0, 0.5).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_track_end(0.7)
                        .with_alpha_attachment_threshold(0.2)
                        .with_mix_attachment_threshold(0.3)
                        .with_mix_draw_order_threshold(0.4),
                ),
            );
        app.update();

        current_track_entry(&app, entity, 0, |entry| {
            assert_eq!(entry.get_animation().get_name(), "<empty>");
            assert_eq!(entry.get_mix_duration(), 0.5);
            assert_eq!(entry.get_track_end(), 0.7);
            assert_eq!(entry.get_alpha_attachment_threshold(), 0.2);
            assert_eq!(entry.get_mix_attachment_threshold(), 0.3);
            assert_eq!(entry.get_mix_draw_order_threshold(), 0.4);
        });
    }

    #[test]
    fn missing_animation_command_panics_without_applying_settings_to_prior_entry() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(
                SpineAnimationCommand::set(entity, 0, "missing", true).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_alpha(0.25)
                        .with_additive(true),
                ),
            );
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.update())).is_err());

        current_track_entry(&app, entity, 0, |entry| {
            assert_eq!(entry.get_animation().get_name(), "first");
            assert_eq!(entry.get_alpha(), 1.0);
            assert!(!entry.get_additive());
        });
    }

    #[test]
    fn mix_set_and_clear_commands_update_state_data() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        {
            let mut messages = app
                .world_mut()
                .resource_mut::<Messages<SpineAnimationCommand>>();
            messages.write(SpineAnimationCommand::set_default_mix(entity, 0.3));
            messages.write(SpineAnimationCommand::set_mix(
                entity, "first", "second", 0.1,
            ));
        }
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::set(entity, 0, "second", true));
        app.update();
        assert_eq!(current_track_mix_duration(&app, entity), 0.1);

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::clear_mixes(entity));
        app.update();
        assert_eq!(animation_state_default_mix(&mut app, entity), 0.0);
    }

    #[test]
    fn skeleton_control_component_applies_spawn_settings() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas),
                SpineSkeletonControl::new()
                    .with_physics(spine2d::Physics::Update)
                    .with_wind(Vec2::new(2.0, 3.0))
                    .with_gravity(Vec2::new(4.0, 5.0))
                    .with_time(1.25),
            ))
            .id();
        app.update();

        spine_instance(&app, entity, |instance| {
            assert_eq!(
                instance.get_skeleton_control().get_physics(),
                spine2d::Physics::Update
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_wind_x(),
                    instance.get_skeleton().get_wind_y()
                ),
                (2.0, 3.0)
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_gravity_x(),
                    instance.get_skeleton().get_gravity_y()
                ),
                (4.0, 5.0)
            );
            assert_eq!(instance.get_skeleton().get_time(), 1.25);
        });
    }

    #[test]
    fn changed_skeleton_control_component_updates_existing_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app.world_mut().spawn(Spine::new(skeleton, atlas)).id();
        app.update();

        app.world_mut().entity_mut(entity).insert(
            SpineSkeletonControl::new()
                .with_physics(spine2d::Physics::Pose)
                .with_wind(Vec2::new(6.0, 7.0)),
        );
        app.update();

        spine_instance(&app, entity, |instance| {
            assert_eq!(
                instance.get_skeleton_control().get_physics(),
                spine2d::Physics::Pose
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_wind_x(),
                    instance.get_skeleton().get_wind_y()
                ),
                (6.0, 7.0)
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_gravity_x(),
                    instance.get_skeleton().get_gravity_y()
                ),
                (0.0, 1.0)
            );
        });
    }

    #[test]
    fn skeleton_commands_update_existing_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app.world_mut().spawn(Spine::new(skeleton, atlas)).id();
        app.update();

        {
            let mut messages = app
                .world_mut()
                .resource_mut::<Messages<SpineSkeletonCommand>>();
            messages.write(SpineSkeletonCommand::set_physics(
                entity,
                spine2d::Physics::Reset,
            ));
            messages.write(SpineSkeletonCommand::set_wind(entity, Vec2::new(8.0, 9.0)));
            messages.write(SpineSkeletonCommand::set_gravity(
                entity,
                Vec2::new(10.0, 11.0),
            ));
            messages.write(SpineSkeletonCommand::set_time(entity, 2.5));
        }
        app.update();

        spine_instance(&app, entity, |instance| {
            assert_eq!(
                instance.get_skeleton_control().get_physics(),
                spine2d::Physics::Reset
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_wind_x(),
                    instance.get_skeleton().get_wind_y()
                ),
                (8.0, 9.0)
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_gravity_x(),
                    instance.get_skeleton().get_gravity_y()
                ),
                (10.0, 11.0)
            );
            assert_eq!(instance.get_skeleton().get_time(), 2.5);
        });
    }

    #[test]
    fn skeleton_command_overrides_changed_control_component_in_same_frame() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas),
                SpineSkeletonControl::new().with_physics(spine2d::Physics::None),
            ))
            .id();
        app.update();

        app.world_mut().entity_mut(entity).insert(
            SpineSkeletonControl::new()
                .with_physics(spine2d::Physics::Pose)
                .with_wind(Vec2::new(1.0, 1.0)),
        );
        app.world_mut()
            .resource_mut::<Messages<SpineSkeletonCommand>>()
            .write(SpineSkeletonCommand::set_control(
                entity,
                SpineSkeletonControl::new()
                    .with_physics(spine2d::Physics::Update)
                    .with_wind(Vec2::new(2.0, 2.0)),
            ));
        app.update();

        spine_instance(&app, entity, |instance| {
            assert_eq!(
                instance.get_skeleton_control().get_physics(),
                spine2d::Physics::Update
            );
            assert_eq!(
                (
                    instance.get_skeleton().get_wind_x(),
                    instance.get_skeleton().get_wind_y()
                ),
                (2.0, 2.0)
            );
        });
    }

    #[test]
    fn runtime_state_snapshot_is_inserted_after_spawn() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("first", true),
                SpineSkeletonControl::new()
                    .with_physics(spine2d::Physics::Update)
                    .with_wind(Vec2::new(3.0, 4.0)),
            ))
            .id();
        app.update();

        let state = app.world().get::<SpineRuntimeState>(entity).unwrap();
        assert!(state.is_ready());
        assert_eq!(state.get_physics(), spine2d::Physics::Update);
        assert_eq!(state.get_wind(), Vec2::new(3.0, 4.0));
        assert_eq!(state.get_tracks().len(), 1);
        assert_eq!(state.get_tracks()[0].get_track_index(), 0);
        assert_eq!(state.get_tracks()[0].get_animation_name(), "first");
        assert!(state.get_tracks()[0].get_loop());
    }

    #[test]
    fn runtime_state_snapshot_reflects_same_frame_commands() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(
                SpineAnimationCommand::set(entity, 0, "second", false).with_entry_settings(
                    SpineTrackEntrySettings::new()
                        .with_alpha(0.5)
                        .with_mix_duration(0.25),
                ),
            );
        app.update();

        let state = app.world().get::<SpineRuntimeState>(entity).unwrap();
        assert_eq!(state.get_tracks().len(), 1);
        assert_eq!(state.get_tracks()[0].get_animation_name(), "second");
        assert!(!state.get_tracks()[0].get_loop());
        assert_eq!(state.get_tracks()[0].get_alpha(), 0.5);
        assert_eq!(state.get_tracks()[0].get_mix_duration(), 0.25);
    }

    #[test]
    fn runtime_state_snapshot_tracks_clear_commands() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();

        app.world_mut()
            .resource_mut::<Messages<SpineAnimationCommand>>()
            .write(SpineAnimationCommand::clear_track(entity, 0));
        app.update();

        let state = app.world().get::<SpineRuntimeState>(entity).unwrap();
        assert!(state.get_tracks().is_empty());
    }

    #[test]
    fn runtime_state_snapshot_is_removed_when_instance_releases() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();
        assert!(app.world().get::<SpineRuntimeState>(entity).is_some());

        app.world_mut().entity_mut(entity).remove::<Spine>();
        app.update();

        assert!(app.world().get::<SpineRuntimeState>(entity).is_none());
    }

    #[test]
    fn animation_commands_can_queue_and_clear_tracks() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = event_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("first", true))
            .id();
        app.update();
        drain_animation_events(&mut app);

        {
            let mut messages = app
                .world_mut()
                .resource_mut::<Messages<SpineAnimationCommand>>();
            messages.write(SpineAnimationCommand::add(entity, 1, "second", false, 0.0));
            messages.write(SpineAnimationCommand::clear_track(entity, 0));
        }
        app.update();

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.get_animation_name(), None);

        let events = drain_animation_events(&mut app);
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.track_index == 1
                && event.animation_name == "second"
                && matches!(event.kind, SpineAnimationEventKind::Start)
        }));
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.track_index == 0
                && event.animation_name == "first"
                && matches!(
                    event.kind,
                    SpineAnimationEventKind::End | SpineAnimationEventKind::Dispose
                )
        }));
    }

    #[test]
    fn modified_spine_assets_rebuild_runtime_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton.clone(), atlas).with_animation("spin", true))
            .id();
        app.update();

        let old_key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let skeleton_data =
            SkeletonData::from_json_str(include_str!("../../spine2d-web/assets/demo.json"))
                .expect("parse demo skeleton");
        app.world_mut()
            .resource_mut::<Assets<SpineSkeletonAsset>>()
            .insert(skeleton.id(), SpineSkeletonAsset::new(skeleton_data))
            .expect("replace skeleton asset");

        app.update();
        assert!(app.world().get::<SpineInstanceKey>(entity).is_none());
        assert!(app.world().get::<SpineReady>(entity).is_none());
        assert!(app.world().get::<SpineBounds>(entity).is_none());
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 0);
        assert!(drain_lifecycle_events(&mut app).iter().any(|event| {
            *event
                == SpineLifecycleEvent {
                    entity,
                    kind: SpineLifecycleEventKind::Released(SpineReleaseReason::AssetReload),
                }
        }));

        app.update();
        let new_key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        assert!(app.world().get::<SpineReady>(entity).is_some());
        assert!(app.world().get::<SpineBounds>(entity).is_some());
        assert_ne!(old_key, new_key);
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);
    }

    #[test]
    fn removing_spine_component_releases_runtime_instance_without_touching_user_controls() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("spin", true),
                animation_component(None, false, 2.0),
                skin_component(None),
            ))
            .id();
        app.update();
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);
        drain_lifecycle_events(&mut app);

        app.world_mut().entity_mut(entity).remove::<Spine>();
        app.update();

        assert_eq!(app.world().non_send::<SpineWorld>().len(), 0);
        assert!(app.world().get::<SpineInstanceKey>(entity).is_none());
        assert!(app.world().get::<SpineReady>(entity).is_none());
        assert!(app.world().get::<SpineBounds>(entity).is_none());
        assert!(app.world().get::<SpineDrawSignatureCache>(entity).is_none());
        assert!(app.world().get::<SpineAnimation>(entity).is_some());
        assert!(app.world().get::<SpineSkin>(entity).is_some());
        assert_eq!(
            drain_lifecycle_events(&mut app),
            vec![SpineLifecycleEvent {
                entity,
                kind: SpineLifecycleEventKind::Released(SpineReleaseReason::ComponentRemoved),
            }]
        );
    }

    #[test]
    fn despawning_spine_entity_releases_runtime_instance() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("spin", true))
            .id();
        app.update();
        assert_eq!(app.world().non_send::<SpineWorld>().len(), 1);
        drain_lifecycle_events(&mut app);

        app.world_mut().entity_mut(entity).despawn();
        app.update();

        assert_eq!(app.world().non_send::<SpineWorld>().len(), 0);
        assert_eq!(
            drain_lifecycle_events(&mut app),
            vec![SpineLifecycleEvent {
                entity,
                kind: SpineLifecycleEventKind::Released(SpineReleaseReason::EntityDespawned),
            }]
        );
    }
}
