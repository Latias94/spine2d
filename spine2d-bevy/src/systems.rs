mod render;

use bevy::asset::AssetEvent;
use bevy::ecs::lifecycle::RemovedComponents;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use spine2d::{AnimationState, AnimationStateData, Skeleton, build_draw_list_with_atlas};
use std::collections::HashSet;

use render::despawn_mesh_children;
pub use render::render_spines;

use crate::{
    Spine, SpineAnimation, SpineAnimationCommand, SpineAnimationCommandKind, SpineAnimationEvent,
    SpineAtlasAsset, SpineDrawSignatureCache, SpineInstance, SpineInstanceKey, SpineInstanceParts,
    SpineLifecycleEvent, SpineLifecycleEventKind, SpineMeshChild, SpineReady, SpineReleaseReason,
    SpineSkeletonAsset, SpineSkin, SpineWorld,
};

type SpawnSpineQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Spine,
        Option<&'static SpineAnimation>,
        Option<&'static SpineSkin>,
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
    ),
>;

type SpineEntityQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Spine, Option<&'static SpineInstanceKey>)>;

type SpineKeyQuery<'w, 's> = Query<'w, 's, &'static SpineInstanceKey>;

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
) {
    for (entity, spine, animation, skin, draw_signature_cache) in &query {
        let Some(skeleton_asset) = skeletons.get(&spine.skeleton) else {
            continue;
        };
        let Some(atlas_asset) = atlases.get(&spine.atlas) else {
            continue;
        };

        let animation_component = animation.cloned().unwrap_or_else(|| SpineAnimation {
            name: spine.animation.clone(),
            loop_animation: spine.loop_animation,
            time_scale: spine.time_scale,
        });
        let skin_component = skin.cloned().unwrap_or_else(|| SpineSkin {
            name: spine.skin.clone(),
        });

        let skeleton_data = skeleton_asset.data.clone();
        let mut skeleton = Skeleton::new(skeleton_data.clone());
        if let Err(err) = skeleton.set_skin(skin_component.name.as_deref()) {
            warn!("Failed to set Spine skin for {entity:?}: {err}");
        }
        skeleton.update_world_transform();

        let mut instance = SpineInstance::new(SpineInstanceParts {
            skeleton,
            animation_state: AnimationState::new(AnimationStateData::new(skeleton_data.clone())),
            atlas: atlas_asset.atlas.clone(),
            atlas_directory: atlas_asset.directory.clone(),
            animation_name: animation_component.name.clone(),
            loop_animation: animation_component.loop_animation,
            time_scale: animation_component.time_scale,
            skin_name: skin_component.name.clone(),
        });
        instance.attach_event_listener();
        if let Some(animation_name) = animation_component.name.as_deref() {
            if let Err(err) = instance.animation_state.set_animation(
                0,
                animation_name,
                animation_component.loop_animation,
            ) {
                warn!("Failed to set Spine animation for {entity:?}: {err}");
            }
        }
        rebuild_pose(&mut instance, 0.0);
        let _ = instance.drain_events();

        let id = spine_world.insert(entity, instance);
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(SpineInstanceKey(id));
        entity_commands.insert(SpineReady);
        if draw_signature_cache.is_none() {
            entity_commands.insert(SpineDrawSignatureCache::default());
        }
        lifecycle_events.write(SpineLifecycleEvent {
            entity,
            kind: SpineLifecycleEventKind::Ready,
        });
    }
}

pub fn cleanup_spine_instances(
    mut commands: Commands,
    mut spine_world: NonSendMut<SpineWorld>,
    mut lifecycle_events: MessageWriter<SpineLifecycleEvent>,
    mut removed_spines: RemovedComponents<Spine>,
    mut removed_instances: RemovedComponents<SpineInstanceKey>,
    mesh_children: SpineMeshChildrenParam,
) {
    let mut removed = HashSet::new();
    removed.extend(removed_spines.read());
    removed.extend(removed_instances.read());

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
        if !changed_skeletons.contains(&spine.skeleton.id())
            && !changed_atlases.contains(&spine.atlas.id())
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
                loop_animation,
            } => {
                if let Err(err) =
                    instance
                        .animation_state
                        .set_animation(*track_index, animation, *loop_animation)
                {
                    warn!(
                        "Failed to set Spine animation for {:?}: {err}",
                        message.entity
                    );
                    continue;
                }
                if *track_index == 0 {
                    instance.animation_name = Some(animation.clone());
                    instance.loop_animation = *loop_animation;
                }
            }
            SpineAnimationCommandKind::Add {
                track_index,
                animation,
                loop_animation,
                delay,
            } => {
                if let Err(err) = instance.animation_state.add_animation(
                    *track_index,
                    animation,
                    *loop_animation,
                    *delay,
                ) {
                    warn!(
                        "Failed to queue Spine animation for {:?}: {err}",
                        message.entity
                    );
                }
            }
            SpineAnimationCommandKind::SetEmpty {
                track_index,
                mix_duration,
            } => {
                if let Err(err) = instance
                    .animation_state
                    .set_empty_animation(*track_index, *mix_duration)
                {
                    warn!(
                        "Failed to set empty Spine animation for {:?}: {err}",
                        message.entity
                    );
                    continue;
                }
                if *track_index == 0 {
                    instance.animation_name = None;
                    instance.loop_animation = false;
                }
            }
            SpineAnimationCommandKind::AddEmpty {
                track_index,
                mix_duration,
                delay,
            } => {
                if let Err(err) = instance.animation_state.add_empty_animation(
                    *track_index,
                    *mix_duration,
                    *delay,
                ) {
                    warn!(
                        "Failed to queue empty Spine animation for {:?}: {err}",
                        message.entity
                    );
                }
            }
            SpineAnimationCommandKind::ClearTrack { track_index } => {
                instance.animation_state.clear_track(*track_index);
                if *track_index == 0 {
                    instance.animation_name = None;
                    instance.loop_animation = false;
                }
            }
            SpineAnimationCommandKind::ClearTracks => {
                instance.animation_state.clear_tracks();
                instance.animation_name = None;
                instance.loop_animation = false;
            }
        }
    }
}

pub fn update_spine_animations(
    mut spine_world: NonSendMut<SpineWorld>,
    mut animation_events: MessageWriter<SpineAnimationEvent>,
    query: UpdateSpineQuery,
    time: Res<Time>,
) {
    for (entity, key, animation_ref, skin_ref) in &query {
        let Some(instance) = spine_world.get_mut(key.0) else {
            warn!("Missing Spine runtime instance for {entity:?}");
            continue;
        };

        if let Some(skin_ref) = skin_ref
            && skin_ref.is_changed()
            && instance.skin_name != skin_ref.name
        {
            match instance.skeleton.set_skin(skin_ref.name.as_deref()) {
                Ok(()) => {
                    instance.skin_name = skin_ref.name.clone();
                }
                Err(err) => warn!("Failed to set Spine skin for {entity:?}: {err}"),
            }
        }

        if let Some(animation_ref) = animation_ref {
            instance.time_scale = animation_ref.time_scale;
            if animation_ref.is_changed()
                && (instance.animation_name != animation_ref.name
                    || instance.loop_animation != animation_ref.loop_animation)
            {
                instance.animation_state.clear_tracks();
                if let Some(animation_name) = animation_ref.name.as_deref()
                    && let Err(err) = instance.animation_state.set_animation(
                        0,
                        animation_name,
                        animation_ref.loop_animation,
                    )
                {
                    warn!("Failed to set Spine animation for {entity:?}: {err}");
                }
                instance.animation_name = animation_ref.name.clone();
                instance.loop_animation = animation_ref.loop_animation;
            }
        }

        rebuild_pose(instance, time.delta().as_secs_f32() * instance.time_scale);
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
    instance.animation_state.update(delta.max(0.0));
    instance.animation_state.apply(&mut instance.skeleton);
    instance.skeleton.update_world_transform();
    instance.draw_list = build_draw_list_with_atlas(&instance.skeleton, &instance.atlas);
}

#[cfg(test)]
mod tests {
    use super::render::texture_asset_path;
    use super::*;
    use crate::{
        SpineAnimationEventKind, SpineDrawSignature, SpineRenderSignature,
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
        .init_resource::<Time>()
        .insert_non_send_resource(SpineWorld::new())
        .add_systems(
            Update,
            (
                cleanup_spine_instances,
                spawn_spine_instances,
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
        let atlas = Atlas::from_str(include_str!("../../spine2d-web/assets/demo.atlas"))
            .expect("parse demo atlas");

        let skeleton = app
            .world_mut()
            .resource_mut::<Assets<SpineSkeletonAsset>>()
            .add(SpineSkeletonAsset {
                data: skeleton_data,
            });
        let atlas = app
            .world_mut()
            .resource_mut::<Assets<SpineAtlasAsset>>()
            .add(SpineAtlasAsset {
                atlas,
                directory: "spine2d-web/assets".to_owned(),
            });

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
        let atlas = Atlas::from_str(include_str!("../../spine2d-web/assets/demo.atlas"))
            .expect("parse demo atlas");

        let skeleton = app
            .world_mut()
            .resource_mut::<Assets<SpineSkeletonAsset>>()
            .add(SpineSkeletonAsset {
                data: skeleton_data,
            });
        let atlas = app
            .world_mut()
            .resource_mut::<Assets<SpineAtlasAsset>>()
            .add(SpineAtlasAsset {
                atlas,
                directory: "spine2d-web/assets".to_owned(),
            });

        (skeleton, atlas)
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

    #[test]
    fn spawn_adds_only_internal_runtime_components_after_assets_are_ready() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn(Spine::new(skeleton, atlas).with_animation("spin", true))
            .id();

        app.update();

        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 1);
        assert!(app.world().get::<SpineInstanceKey>(entity).is_some());
        assert!(app.world().get::<SpineReady>(entity).is_some());
        assert!(app.world().get::<SpineDrawSignatureCache>(entity).is_some());
        assert!(app.world().get::<SpineAnimation>(entity).is_none());
        assert!(app.world().get::<SpineSkin>(entity).is_none());

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send_resource::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.animation_name, Some("spin".to_owned()));
        assert!(instance.loop_animation);
        assert_eq!(instance.time_scale, 1.0);

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
                SpineAnimation {
                    name: None,
                    loop_animation: false,
                    time_scale: 2.0,
                },
                SpineSkin { name: None },
            ))
            .id();

        app.update();

        let animation = app.world().get::<SpineAnimation>(entity).unwrap();
        assert_eq!(animation.name, None);
        assert!(!animation.loop_animation);
        assert_eq!(animation.time_scale, 2.0);
        assert_eq!(app.world().get::<SpineSkin>(entity).unwrap().name, None);

        let key = *app.world().get::<SpineInstanceKey>(entity).unwrap();
        let spine_world = app.world().non_send_resource::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.animation_name, None);
        assert!(!instance.loop_animation);
        assert_eq!(instance.skin_name, None);
    }

    #[test]
    fn spawn_preserves_existing_draw_signature_cache() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);
        let signature = SpineDrawSignature {
            texture_path: "old.png".to_owned(),
            blend: BlendMode::Normal,
            premultiplied_alpha: false,
        };

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas),
                SpineDrawSignatureCache {
                    signature: SpineRenderSignature {
                        draws: vec![signature.clone()],
                        render_layers: None,
                    },
                },
            ))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .signature
                .draws,
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
                .signature
                .render_layers,
            Some(RenderLayers::layer(2))
        );

        app.world_mut()
            .entity_mut(entity)
            .insert(RenderLayers::layer(5));
        app.update();

        assert_eq!(
            app.world()
                .get::<SpineDrawSignatureCache>(entity)
                .unwrap()
                .signature
                .render_layers,
            Some(RenderLayers::layer(5))
        );
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
                .signature
                .draws
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
                        if spine_event.name == "hit"
                            && spine_event.int_value == 7
                            && spine_event.string == "impact"
                )
        }));
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

        app.world_mut().entity_mut(entity).insert(SpineAnimation {
            name: Some("second".to_owned()),
            loop_animation: false,
            time_scale: 1.0,
        });
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
        let spine_world = app.world().non_send_resource::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.animation_name, Some("first".to_owned()));
        assert!(instance.loop_animation);

        let events = drain_animation_events(&mut app);
        assert!(events.iter().any(|event| {
            event.entity == entity
                && event.animation_name == "first"
                && matches!(event.kind, SpineAnimationEventKind::Start)
        }));
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
        let spine_world = app.world().non_send_resource::<SpineWorld>();
        let instance = spine_world.get(key.0).unwrap();
        assert_eq!(instance.animation_name, None);

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
            .insert(
                skeleton.id(),
                SpineSkeletonAsset {
                    data: skeleton_data,
                },
            )
            .expect("replace skeleton asset");

        app.update();
        assert!(app.world().get::<SpineInstanceKey>(entity).is_none());
        assert!(app.world().get::<SpineReady>(entity).is_none());
        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 0);
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
        assert_ne!(old_key, new_key);
        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 1);
    }

    #[test]
    fn removing_spine_component_releases_runtime_instance_without_touching_user_controls() {
        let mut app = app_with_lifecycle_systems();
        let (skeleton, atlas) = demo_handles(&mut app);

        let entity = app
            .world_mut()
            .spawn((
                Spine::new(skeleton, atlas).with_animation("spin", true),
                SpineAnimation {
                    name: None,
                    loop_animation: false,
                    time_scale: 2.0,
                },
                SpineSkin { name: None },
            ))
            .id();
        app.update();
        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 1);
        drain_lifecycle_events(&mut app);

        app.world_mut().entity_mut(entity).remove::<Spine>();
        app.update();

        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 0);
        assert!(app.world().get::<SpineInstanceKey>(entity).is_none());
        assert!(app.world().get::<SpineReady>(entity).is_none());
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
        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 1);
        drain_lifecycle_events(&mut app);

        app.world_mut().entity_mut(entity).despawn();
        app.update();

        assert_eq!(app.world().non_send_resource::<SpineWorld>().len(), 0);
        assert_eq!(
            drain_lifecycle_events(&mut app),
            vec![SpineLifecycleEvent {
                entity,
                kind: SpineLifecycleEventKind::Released(SpineReleaseReason::EntityDespawned),
            }]
        );
    }
}
