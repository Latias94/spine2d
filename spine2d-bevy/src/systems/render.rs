use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::system::SystemParam;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::{
    SpineDrawSignature, SpineDrawSignatureCache, SpineInstanceKey, SpineMeshChild,
    SpineRenderSignature, SpineWorld,
    materials::{
        DARK_COLOR_ATTRIBUTE, SpineAdditiveMaterial, SpineAdditivePmaMaterial, SpineMaterialCache,
        SpineMaterialKey, SpineMultiplyMaterial, SpineMultiplyPmaMaterial, SpineNormalMaterial,
        SpineNormalPmaMaterial, SpineScreenMaterial, SpineScreenPmaMaterial, insert_spine_material,
    },
};

#[derive(SystemParam)]
pub(crate) struct SpineRenderAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    normal_mats: ResMut<'w, Assets<SpineNormalMaterial>>,
    additive_mats: ResMut<'w, Assets<SpineAdditiveMaterial>>,
    multiply_mats: ResMut<'w, Assets<SpineMultiplyMaterial>>,
    screen_mats: ResMut<'w, Assets<SpineScreenMaterial>>,
    normal_pma_mats: ResMut<'w, Assets<SpineNormalPmaMaterial>>,
    additive_pma_mats: ResMut<'w, Assets<SpineAdditivePmaMaterial>>,
    multiply_pma_mats: ResMut<'w, Assets<SpineMultiplyPmaMaterial>>,
    screen_pma_mats: ResMut<'w, Assets<SpineScreenPmaMaterial>>,
    material_cache: ResMut<'w, SpineMaterialCache>,
    asset_server: Res<'w, AssetServer>,
}

#[derive(SystemParam)]
pub(crate) struct SpineMeshChildren<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    mesh_children: Query<'w, 's, &'static SpineMeshChild>,
}

pub fn render_spines(
    mut commands: Commands,
    mut render_assets: SpineRenderAssets,
    spine_world: NonSend<SpineWorld>,
    mut spine_query: Query<(
        Entity,
        &SpineInstanceKey,
        &mut SpineDrawSignatureCache,
        Option<&RenderLayers>,
    )>,
    mesh_children: SpineMeshChildren,
) {
    for (spine_entity, key, mut signature_cache, render_layers) in &mut spine_query {
        let Some(instance) = spine_world.get(key.0) else {
            continue;
        };
        let draw_list = &instance.draw_list;
        let new_render_layers = render_layers.cloned();
        let new_draws = draw_list
            .draws
            .iter()
            .map(|draw| {
                SpineDrawSignature::from_draw(
                    draw,
                    texture_asset_path(&instance.atlas_directory, &draw.texture_path),
                )
            })
            .collect::<Vec<_>>();
        let geometry_changed = new_draws != signature_cache.signature.draws;
        let render_layers_changed = new_render_layers != signature_cache.signature.render_layers;

        if new_draws.is_empty() {
            if !signature_cache.signature.draws.is_empty() {
                despawn_mesh_children(
                    &mut commands,
                    &mesh_children.children,
                    &mesh_children.mesh_children,
                    spine_entity,
                );
                signature_cache.signature = SpineRenderSignature::default();
            }
            continue;
        }

        if geometry_changed {
            despawn_mesh_children(
                &mut commands,
                &mesh_children.children,
                &mesh_children.mesh_children,
                spine_entity,
            );
            spawn_mesh_children(
                &mut commands,
                &mut render_assets,
                &instance.atlas_directory,
                spine_entity,
                draw_list,
                render_layers,
            );
            signature_cache.signature = SpineRenderSignature {
                draws: new_draws,
                render_layers: new_render_layers,
            };
            continue;
        }

        let Some(mesh_child_handles) = collect_mesh_children(
            &mesh_children.children,
            &mesh_children.mesh_children,
            spine_entity,
        ) else {
            spawn_mesh_children(
                &mut commands,
                &mut render_assets,
                &instance.atlas_directory,
                spine_entity,
                draw_list,
                render_layers,
            );
            signature_cache.signature = SpineRenderSignature {
                draws: new_draws,
                render_layers: new_render_layers,
            };
            continue;
        };

        let stale_meshes = mesh_child_handles.len() != draw_list.draws.len()
            || mesh_child_handles
                .iter()
                .any(|mesh_handle| !render_assets.meshes.contains(mesh_handle.id()));
        if stale_meshes {
            despawn_mesh_children(
                &mut commands,
                &mesh_children.children,
                &mesh_children.mesh_children,
                spine_entity,
            );
            spawn_mesh_children(
                &mut commands,
                &mut render_assets,
                &instance.atlas_directory,
                spine_entity,
                draw_list,
                render_layers,
            );
            signature_cache.signature = SpineRenderSignature {
                draws: new_draws,
                render_layers: new_render_layers,
            };
            continue;
        }

        if render_layers_changed {
            sync_mesh_child_render_layers(
                &mut commands,
                &mesh_children.children,
                &mesh_children.mesh_children,
                spine_entity,
                render_layers,
            );
            signature_cache.signature.render_layers = new_render_layers;
        }

        for (draw, mesh_handle) in draw_list.draws.iter().zip(mesh_child_handles.iter()) {
            if let Some(mesh) = render_assets.meshes.get_mut(mesh_handle) {
                write_mesh_data(mesh, draw_list, draw);
            }
        }
    }
}

fn collect_mesh_children(
    children_query: &Query<&Children>,
    mesh_child_query: &Query<&SpineMeshChild>,
    spine_entity: Entity,
) -> Option<Vec<Handle<Mesh>>> {
    children_query.get(spine_entity).ok().map(|children| {
        children
            .iter()
            .filter_map(|child| mesh_child_query.get(child).ok())
            .map(|child| child.mesh.clone())
            .collect::<Vec<_>>()
    })
}

fn write_mesh_data(mesh: &mut Mesh, draw_list: &spine2d::DrawList, draw: &spine2d::Draw) {
    let raw_indices = &draw_list.indices[draw.first_index..draw.first_index + draw.index_count];
    let Some(min_vertex) = raw_indices.iter().min().map(|i| *i as usize) else {
        mesh.insert_indices(Indices::U32(Vec::new()));
        return;
    };
    let max_vertex = raw_indices
        .iter()
        .max()
        .map(|i| *i as usize)
        .unwrap_or(min_vertex);

    let indices = raw_indices
        .iter()
        .map(|&i| (i as usize - min_vertex) as u32)
        .collect::<Vec<_>>();
    let vertex_slice = &draw_list.vertices[min_vertex..=max_vertex];

    let positions = vertex_slice
        .iter()
        .map(|v| [v.position[0], v.position[1], 0.0])
        .collect::<Vec<_>>();
    let normals = vec![[0.0, 0.0, 1.0]; vertex_slice.len()];
    let uvs = vertex_slice.iter().map(|v| v.uv).collect::<Vec<_>>();
    let colors = vertex_slice.iter().map(|v| v.color).collect::<Vec<_>>();
    let dark_colors = vertex_slice
        .iter()
        .map(|v| v.dark_color)
        .collect::<Vec<_>>();

    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(DARK_COLOR_ATTRIBUTE, dark_colors);
}

fn spawn_mesh_children(
    commands: &mut Commands,
    render_assets: &mut SpineRenderAssets,
    atlas_dir: &str,
    spine_entity: Entity,
    draw_list: &spine2d::DrawList,
    render_layers: Option<&RenderLayers>,
) {
    commands.entity(spine_entity).with_children(|parent| {
        for (draw_index, draw) in draw_list.draws.iter().enumerate() {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            write_mesh_data(&mut mesh, draw_list, draw);

            let mesh_handle = render_assets.meshes.add(mesh);
            let texture_path = texture_asset_path(atlas_dir, draw.texture_path.as_str());
            let texture = render_assets.asset_server.load(texture_path.clone());
            let material_handle = render_assets.material_cache.get_or_create(
                SpineMaterialKey {
                    texture,
                    blend: draw.blend,
                    premultiplied_alpha: draw.premultiplied_alpha,
                },
                &mut render_assets.normal_mats,
                &mut render_assets.additive_mats,
                &mut render_assets.multiply_mats,
                &mut render_assets.screen_mats,
                &mut render_assets.normal_pma_mats,
                &mut render_assets.additive_pma_mats,
                &mut render_assets.multiply_pma_mats,
                &mut render_assets.screen_pma_mats,
            );

            let mut child = parent.spawn((
                SpineMeshChild {
                    mesh: mesh_handle.clone(),
                },
                Mesh2d(mesh_handle),
                Transform::from_xyz(0.0, 0.0, draw_index as f32 * 0.001),
            ));
            insert_spine_material(&mut child, material_handle);

            if let Some(layers) = render_layers {
                child.insert(layers.clone());
            }
        }
    });
}

pub(super) fn texture_asset_path(atlas_dir: &str, texture_path: &str) -> String {
    if atlas_dir.is_empty() {
        texture_path.to_string()
    } else {
        format!("{atlas_dir}/{texture_path}")
    }
}

pub(super) fn despawn_mesh_children(
    commands: &mut Commands,
    children_query: &Query<&Children>,
    mesh_child_query: &Query<&SpineMeshChild>,
    spine_entity: Entity,
) {
    let Ok(children) = children_query.get(spine_entity) else {
        return;
    };
    for child in children.iter() {
        if mesh_child_query.get(child).is_ok()
            && let Ok(mut entity_commands) = commands.get_entity(child)
        {
            entity_commands.despawn();
        }
    }
}

fn sync_mesh_child_render_layers(
    commands: &mut Commands,
    children_query: &Query<&Children>,
    mesh_child_query: &Query<&SpineMeshChild>,
    spine_entity: Entity,
    render_layers: Option<&RenderLayers>,
) {
    let Ok(children) = children_query.get(spine_entity) else {
        return;
    };

    for child in children.iter() {
        if mesh_child_query.get(child).is_ok()
            && let Ok(mut entity_commands) = commands.get_entity(child)
        {
            match render_layers {
                Some(layers) => {
                    entity_commands.insert(layers.clone());
                }
                None => {
                    entity_commands.remove::<RenderLayers>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use spine2d::{BlendMode, Draw, DrawList, Vertex};

    #[test]
    fn write_mesh_data_preserves_spine_y_axis() {
        let draw_list = DrawList {
            vertices: vec![
                Vertex {
                    position: [1.0, 2.0],
                    uv: [0.0, 0.0],
                    color: [1.0; 4],
                    dark_color: [0.0; 4],
                },
                Vertex {
                    position: [4.0, 5.0],
                    uv: [1.0, 1.0],
                    color: [1.0; 4],
                    dark_color: [0.0; 4],
                },
            ],
            indices: vec![0, 1],
            draws: Vec::new(),
        };
        let draw = Draw {
            texture_path: "page.png".to_owned(),
            blend: BlendMode::Normal,
            premultiplied_alpha: false,
            first_index: 0,
            index_count: 2,
        };
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        write_mesh_data(&mut mesh, &draw_list, &draw);

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("position attribute should be Float32x3");
        };
        assert_eq!(positions, &vec![[1.0, 2.0, 0.0], [4.0, 5.0, 0.0]]);
    }
}
