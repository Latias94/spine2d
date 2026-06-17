use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use spine2d::{AnimationState, AnimationStateData, BlendMode, Skeleton, build_draw_list_with_atlas};

use crate::{
    Spine, SpineAnimationPlayer, SpineAtlasAsset, SpineHandle, SpineInstance,
    SpineSkeletonAsset, SpineSkin, SpineWorld,
    materials::{
        DARK_COLOR_ATTRIBUTE,
        SpineNormalMaterial, SpineAdditiveMaterial, SpineMultiplyMaterial, SpineScreenMaterial,
        SpineNormalPmaMaterial, SpineAdditivePmaMaterial, SpineMultiplyPmaMaterial, SpineScreenPmaMaterial,
        insert_spine_material, SpineMaterialHandle,
    },
};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Tracks how many mesh children the parent Spine entity currently has.
/// Used to detect when the draw list count changes so we know whether to
/// update in place or fall back to a full respawn.
#[derive(Component, Default)]
pub struct SpineDrawCount(pub usize);

/// Marks a child entity as a Spine mesh and stores the mesh asset handle so
/// render_spines can write updated vertex data directly into it each frame.
#[derive(Component)]
pub struct SpineMeshChild {
    pub mesh: Handle<Mesh>,
}

// ---------------------------------------------------------------------------
// Helper: build vertex data from a draw and write it into an existing Mesh.
// ---------------------------------------------------------------------------

fn write_mesh_data(
    mesh: &mut Mesh,
    draw_list: &spine2d::DrawList,
    draw: &spine2d::Draw,
) {
    let raw_indices = &draw_list.indices[draw.first_index..draw.first_index + draw.index_count];

    let min_vertex = *raw_indices.iter().min().unwrap() as usize;
    let max_vertex = *raw_indices.iter().max().unwrap() as usize;

    let indices: Vec<u32> = raw_indices.iter()
        .map(|&i| (i as usize - min_vertex) as u32)
        .collect();

    let vertex_slice = &draw_list.vertices[min_vertex..=max_vertex];

    let positions: Vec<[f32; 3]> = vertex_slice.iter()
        .map(|v| [v.position[0], -v.position[1], 0.0])
        .collect();
    let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; vertex_slice.len()];
    let uvs: Vec<[f32; 2]> = vertex_slice.iter().map(|v| v.uv).collect();
    let colors: Vec<[f32; 4]> = vertex_slice.iter().map(|v| v.color).collect();
    let dark_colors: Vec<[f32; 4]> = vertex_slice.iter().map(|v| v.dark_color).collect();

    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(DARK_COLOR_ATTRIBUTE, dark_colors);
}

// ---------------------------------------------------------------------------
// Helper: spawn all mesh children for a Spine entity from scratch.
// ---------------------------------------------------------------------------

fn spawn_mesh_children(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    normal_mats:       &mut Assets<SpineNormalMaterial>,
    additive_mats:     &mut Assets<SpineAdditiveMaterial>,
    multiply_mats:     &mut Assets<SpineMultiplyMaterial>,
    screen_mats:       &mut Assets<SpineScreenMaterial>,
    normal_pma_mats:   &mut Assets<SpineNormalPmaMaterial>,
    additive_pma_mats: &mut Assets<SpineAdditivePmaMaterial>,
    multiply_pma_mats: &mut Assets<SpineMultiplyPmaMaterial>,
    screen_pma_mats:   &mut Assets<SpineScreenPmaMaterial>,
    asset_server:      &AssetServer,
    atlas_dir:         &str,
    spine_entity:      Entity,
    draw_list:         &spine2d::DrawList,
    render_layers:     Option<&RenderLayers>,
) {
    commands.entity(spine_entity).with_children(|parent| {
        for (draw_index, draw) in draw_list.draws.iter().enumerate() {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            write_mesh_data(&mut mesh, draw_list, draw);

            let mesh_handle = meshes.add(mesh);

            let texture_path = if atlas_dir.is_empty() {
                draw.texture_path.clone()
            } else {
                format!("{}/{}", atlas_dir, draw.texture_path)
            };
            let texture: Handle<Image> = asset_server.load(texture_path);

            let material_handle = match (draw.blend, draw.premultiplied_alpha) {
                (BlendMode::Normal,   false) => SpineMaterialHandle::Normal(normal_mats.add(SpineNormalMaterial { texture })),
                (BlendMode::Additive, false) => SpineMaterialHandle::Additive(additive_mats.add(SpineAdditiveMaterial { texture })),
                (BlendMode::Multiply, false) => SpineMaterialHandle::Multiply(multiply_mats.add(SpineMultiplyMaterial { texture })),
                (BlendMode::Screen,   false) => SpineMaterialHandle::Screen(screen_mats.add(SpineScreenMaterial { texture })),
                (BlendMode::Normal,   true)  => SpineMaterialHandle::NormalPma(normal_pma_mats.add(SpineNormalPmaMaterial { texture })),
                (BlendMode::Additive, true)  => SpineMaterialHandle::AdditivePma(additive_pma_mats.add(SpineAdditivePmaMaterial { texture })),
                (BlendMode::Multiply, true)  => SpineMaterialHandle::MultiplyPma(multiply_pma_mats.add(SpineMultiplyPmaMaterial { texture })),
                (BlendMode::Screen,   true)  => SpineMaterialHandle::ScreenPma(screen_pma_mats.add(SpineScreenPmaMaterial { texture })),
            };

            let z = draw_index as f32 * 0.001;

            let mut child = parent.spawn((
                SpineMeshChild { mesh: mesh_handle.clone() },
                Mesh2d(mesh_handle),
                Transform::from_xyz(0.0, 0.0, z),
            ));

            insert_spine_material(&mut child, material_handle);

            if let Some(layers) = render_layers {
                child.insert(layers.clone());
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

pub fn spawn_spine_instances(
    mut commands: Commands,
    mut spine_world: NonSendMut<SpineWorld>,
    skeletons: Res<Assets<SpineSkeletonAsset>>,
    atlases: Res<Assets<SpineAtlasAsset>>,
    query: Query<(Entity, &Spine), Without<SpineHandle>>,
) {
    for (entity, spine) in query.iter() {
        let Some(skeleton_asset) = skeletons.get(&spine.skeleton) else { continue };
        let Some(atlas_asset) = atlases.get(&spine.atlas) else { continue };

        info!("Spawning SpineInstance for entity {:?}", entity);

        let skeleton_data = skeleton_asset.data.clone();
        let mut skeleton = Skeleton::new(skeleton_data.clone());
        let state_data = AnimationStateData::new(skeleton_data.clone());
        let mut animation_state = AnimationState::new(state_data);

        if !spine.animation.is_empty() {
            let _ = animation_state.set_animation(0, &spine.animation, spine.loop_animation);
        }

        skeleton.update_world_transform();

        let instance = SpineInstance::new(
            skeleton,
            animation_state,
            atlas_asset.atlas.clone(),
            skeleton_data,
        );

        let handle = spine_world.insert(instance);

        commands.entity(entity)
            .insert(SpineHandle(handle.0))
            .insert(SpineDrawCount::default())
            .insert(SpineAnimationPlayer {
                animation_name: spine.animation.clone(),
                loop_animation: spine.loop_animation,
                time_scale: spine.time_scale,
            })
            .insert(SpineSkin {
                skin_name: spine.skin.clone().unwrap_or_default(),
            });
    }
}

pub fn update_spine_animations(
    mut spine_world: NonSendMut<SpineWorld>,
    query: Query<(&SpineHandle, &SpineAnimationPlayer, &SpineSkin)>,
    time: Res<Time>,
) {
    for (handle, player, skin) in query.iter() {
        let instance = spine_world.get_mut(*handle);

        if !skin.skin_name.is_empty() {
            let _ = instance.skeleton.set_skin(Some(&skin.skin_name));
        }

        instance.animation_state.update(time.delta().as_secs_f32() * player.time_scale);
        instance.animation_state.apply(&mut instance.skeleton);
        instance.skeleton.update_world_transform();

        instance.draw_list = build_draw_list_with_atlas(&instance.skeleton, &instance.atlas);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_spines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut normal_mats:       ResMut<Assets<SpineNormalMaterial>>,
    mut additive_mats:     ResMut<Assets<SpineAdditiveMaterial>>,
    mut multiply_mats:     ResMut<Assets<SpineMultiplyMaterial>>,
    mut screen_mats:       ResMut<Assets<SpineScreenMaterial>>,
    mut normal_pma_mats:   ResMut<Assets<SpineNormalPmaMaterial>>,
    mut additive_pma_mats: ResMut<Assets<SpineAdditivePmaMaterial>>,
    mut multiply_pma_mats: ResMut<Assets<SpineMultiplyPmaMaterial>>,
    mut screen_pma_mats:   ResMut<Assets<SpineScreenPmaMaterial>>,
    asset_server: Res<AssetServer>,
    atlases: Res<Assets<SpineAtlasAsset>>,
    spine_world: NonSend<SpineWorld>,
    mut spine_query: Query<(Entity, &SpineHandle, &Spine, &mut SpineDrawCount, Option<&RenderLayers>)>,
    children_query: Query<&Children>,
    mesh_child_query: Query<&SpineMeshChild>,
) {
    for (spine_entity, handle, spine_component, mut draw_count, render_layers) in spine_query.iter_mut() {
        let instance = spine_world.get(*handle);
        let draw_list = &instance.draw_list;

        let atlas_dir = atlases
            .get(&spine_component.atlas)
            .map(|a| a.directory.as_str())
            .unwrap_or("");

        let new_count = draw_list.draws.len();
        let old_count = draw_count.0;

        if new_count == 0 {
            // Nothing to render; if we had children before, despawn them.
            if old_count > 0 {
                despawn_mesh_children(&mut commands, &children_query, &mesh_child_query, spine_entity);
                draw_count.0 = 0;
            }
            continue;
        }

        if new_count != old_count {
            // Draw count changed — despawn stale children and respawn everything.
            if old_count > 0 {
                despawn_mesh_children(&mut commands, &children_query, &mesh_child_query, spine_entity);
            }
            spawn_mesh_children(
                &mut commands, &mut meshes,
                &mut normal_mats, &mut additive_mats, &mut multiply_mats, &mut screen_mats,
                &mut normal_pma_mats, &mut additive_pma_mats, &mut multiply_pma_mats, &mut screen_pma_mats,
                &asset_server, atlas_dir, spine_entity, draw_list, render_layers,
            );
            draw_count.0 = new_count;
        } else {
            // Draw count unchanged — update vertex data in existing mesh assets in place.
            let Ok(children) = children_query.get(spine_entity) else { continue };

            let mesh_children: Vec<Handle<Mesh>> = children
                .iter()
                .filter_map(|child| mesh_child_query.get(child).ok())
                .map(|c| c.mesh.clone())
                .collect();

            for (draw, mesh_handle) in draw_list.draws.iter().zip(mesh_children.iter()) {
                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    write_mesh_data(mesh, draw_list, draw);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: despawn all SpineMeshChild entities under a parent.
// ---------------------------------------------------------------------------

fn despawn_mesh_children(
    commands: &mut Commands,
    children_query: &Query<&Children>,
    mesh_child_query: &Query<&SpineMeshChild>,
    spine_entity: Entity,
) {
    if let Ok(children) = children_query.get(spine_entity) {
        for child in children.iter() {
            if mesh_child_query.get(child).is_ok() {
                commands.entity(child).despawn();
            }
        }
    }
}