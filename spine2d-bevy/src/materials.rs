use bevy::prelude::*;
use bevy::asset::{uuid_handle, load_internal_binary_asset};
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
    RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexFormat,
};
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPlugin};
use bevy::shader::ShaderRef;
use spine2d::BlendMode;

pub const SPINE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

/// Custom vertex attribute for Spine's two-color tinting (dark color).
/// Must match @location(10) in spine.wgsl.
pub const DARK_COLOR_ATTRIBUTE: MeshVertexAttribute = MeshVertexAttribute::new(
    "Vertex_DarkColor",
    10,
    VertexFormat::Float32x4,
);

// ---------------------------------------------------------------------------
// Macro: generates one Material2d struct per blend mode / PMA combination.
// All 8 variants share the same shader; only the blend state differs.
// ---------------------------------------------------------------------------
macro_rules! spine_material {
    ($name:ident, $blend_state:expr) => {
        #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
        pub struct $name {
            #[texture(0)]
            #[sampler(1)]
            pub texture: Handle<Image>,
        }

        impl Material2d for $name {
            fn vertex_shader() -> ShaderRef {
                SPINE_SHADER_HANDLE.into()
            }

            fn fragment_shader() -> ShaderRef {
                SPINE_SHADER_HANDLE.into()
            }

            fn alpha_mode(&self) -> AlphaMode2d {
                AlphaMode2d::Blend
            }

            fn specialize(
                descriptor: &mut RenderPipelineDescriptor,
                layout: &MeshVertexBufferLayoutRef,
                _key: Material2dKey<Self>,
            ) -> Result<(), SpecializedMeshPipelineError> {
                let vertex_attributes = vec![
                    Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                    Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
                    Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
                    Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
                    DARK_COLOR_ATTRIBUTE.at_shader_location(10),
                ];
                let vertex_buffer_layout = layout.0.get_layout(&vertex_attributes)?;
                descriptor.vertex.buffers = vec![vertex_buffer_layout];

                if let Some(fragment) = &mut descriptor.fragment {
                    if let Some(target) = fragment.targets[0].as_mut() {
                        target.blend = Some($blend_state);
                    }
                }

                descriptor.primitive.cull_mode = None;
                Ok(())
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Non-premultiplied alpha variants
// ---------------------------------------------------------------------------

spine_material!(SpineNormalMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
});

spine_material!(SpineAdditiveMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
});

spine_material!(SpineMultiplyMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::Dst,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::OneMinusSrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
});

spine_material!(SpineScreenMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::OneMinusSrc,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
});

// ---------------------------------------------------------------------------
// Premultiplied alpha variants
// ---------------------------------------------------------------------------

spine_material!(SpineNormalPmaMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
});

spine_material!(SpineAdditivePmaMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
});

spine_material!(SpineMultiplyPmaMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::Dst,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::OneMinusSrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
});

spine_material!(SpineScreenPmaMaterial, BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::OneMinusSrc,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
});

// ---------------------------------------------------------------------------
// Convenience enum so render_spines can pass a single typed material handle
// without matching on 8 separate asset collections.
// ---------------------------------------------------------------------------

/// A handle to whichever Spine material variant was chosen for a draw call.
pub enum SpineMaterialHandle {
    Normal(Handle<SpineNormalMaterial>),
    Additive(Handle<SpineAdditiveMaterial>),
    Multiply(Handle<SpineMultiplyMaterial>),
    Screen(Handle<SpineScreenMaterial>),
    NormalPma(Handle<SpineNormalPmaMaterial>),
    AdditivePma(Handle<SpineAdditivePmaMaterial>),
    MultiplyPma(Handle<SpineMultiplyPmaMaterial>),
    ScreenPma(Handle<SpineScreenPmaMaterial>),
}

/// Adds the correct `MeshMaterial2d` component for a given `SpineMaterialHandle`
/// to a `EntityCommands`.
pub fn insert_spine_material(
    entity: &mut bevy::ecs::system::EntityCommands,
    handle: SpineMaterialHandle,
) {
    use bevy::sprite_render::MeshMaterial2d;
    match handle {
        SpineMaterialHandle::Normal(h)      => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::Additive(h)    => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::Multiply(h)    => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::Screen(h)      => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::NormalPma(h)   => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::AdditivePma(h) => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::MultiplyPma(h) => { entity.insert(MeshMaterial2d(h)); }
        SpineMaterialHandle::ScreenPma(h)   => { entity.insert(MeshMaterial2d(h)); }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SpineMaterialPlugin;

impl Plugin for SpineMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            SPINE_SHADER_HANDLE,
            "spine.wgsl",
            |bytes: &[u8], path: String| Shader::from_wgsl(
                std::str::from_utf8(bytes).unwrap().to_owned(),
                path
            )
        );

        app.add_plugins((
            Material2dPlugin::<SpineNormalMaterial>::default(),
            Material2dPlugin::<SpineAdditiveMaterial>::default(),
            Material2dPlugin::<SpineMultiplyMaterial>::default(),
            Material2dPlugin::<SpineScreenMaterial>::default(),
            Material2dPlugin::<SpineNormalPmaMaterial>::default(),
            Material2dPlugin::<SpineAdditivePmaMaterial>::default(),
            Material2dPlugin::<SpineMultiplyPmaMaterial>::default(),
            Material2dPlugin::<SpineScreenPmaMaterial>::default(),
        ));
    }
}