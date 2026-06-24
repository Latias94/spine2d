use spine2d::{BlendMode, DrawList};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
    dark_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    clip_from_world: [[f32; 4]; 4],
}

pub struct SpineRenderer {
    pipelines: Pipelines,
    pipelines_pma: Pipelines,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
}

struct Pipelines {
    normal: wgpu::RenderPipeline,
    additive: wgpu::RenderPipeline,
    multiply: wgpu::RenderPipeline,
    screen: wgpu::RenderPipeline,
}

impl Pipelines {
    fn by_blend(&self, blend: BlendMode) -> &wgpu::RenderPipeline {
        match blend {
            BlendMode::Normal => &self.normal,
            BlendMode::Additive => &self.additive,
            BlendMode::Multiply => &self.multiply,
            BlendMode::Screen => &self.screen,
        }
    }
}

impl SpineRenderer {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("spine2d-wgpu shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("globals bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("spine2d-wgpu pipeline layout"),
            bind_group_layouts: &[&globals_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipelines = create_pipelines(device, &pipeline_layout, &shader, color_format, false);
        let pipelines_pma = create_pipelines(device, &pipeline_layout, &shader, color_format, true);

        let globals = Globals {
            clip_from_world: [[0.0; 4]; 4],
        };
        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals bind group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        let vertex_capacity = 1024;
        let index_capacity = 2048;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spine2d vertices"),
            size: (vertex_capacity * std::mem::size_of::<GpuVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spine2d indices"),
            size: (index_capacity * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipelines,
            pipelines_pma,
            globals_buffer,
            globals_bind_group,
            texture_bind_group_layout,
            vertex_buffer,
            index_buffer,
            vertex_capacity,
            index_capacity,
        }
    }

    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    pub fn update_globals_ortho_centered(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        // Treat world coordinates as centered pixels: x in [-w/2,w/2], y in [-h/2,h/2].
        let sx = 2.0 / width.max(1.0);
        let sy = 2.0 / height.max(1.0);
        let globals = Globals {
            clip_from_world: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));
    }

    pub fn update_globals_matrix(&self, queue: &wgpu::Queue, clip_from_world: [[f32; 4]; 4]) {
        let globals = Globals { clip_from_world };
        queue.write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, draw_list: &DrawList) {
        let vertices = draw_list
            .vertices
            .iter()
            .map(|v| GpuVertex {
                position: v.position,
                uv: v.uv,
                color: v.color,
                dark_color: v.dark_color,
            })
            .collect::<Vec<_>>();

        self.ensure_buffers(device, vertices.len(), draw_list.indices.len());
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&draw_list.indices),
        );
    }

    pub fn render<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        draw_list: &'a DrawList,
        textures: &'a dyn TextureProvider,
    ) {
        if draw_list.indices.is_empty() || draw_list.vertices.is_empty() {
            return;
        }

        pass.set_bind_group(0, &self.globals_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        for draw in &draw_list.draws {
            let pipeline = if draw.premultiplied_alpha {
                self.pipelines_pma.by_blend(draw.blend)
            } else {
                self.pipelines.by_blend(draw.blend)
            };
            pass.set_pipeline(pipeline);
            if let Some(bind_group) = textures.bind_group_for(&draw.texture_path) {
                pass.set_bind_group(1, bind_group, &[]);
            }
            let start = draw.first_index as u32;
            let end = (draw.first_index + draw.index_count) as u32;
            pass.draw_indexed(start..end, 0, 0..1);
        }
    }

    fn ensure_buffers(&mut self, device: &wgpu::Device, vertices: usize, indices: usize) {
        if vertices > self.vertex_capacity {
            while self.vertex_capacity < vertices {
                self.vertex_capacity *= 2;
            }
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("spine2d vertices"),
                size: (self.vertex_capacity * std::mem::size_of::<GpuVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if indices > self.index_capacity {
            while self.index_capacity < indices {
                self.index_capacity *= 2;
            }
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("spine2d indices"),
                size: (self.index_capacity * std::mem::size_of::<u32>()) as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
    }
}

fn create_pipelines(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    premultiplied_alpha: bool,
) -> Pipelines {
    Pipelines {
        normal: create_pipeline(
            device,
            layout,
            shader,
            color_format,
            BlendMode::Normal,
            premultiplied_alpha,
        ),
        additive: create_pipeline(
            device,
            layout,
            shader,
            color_format,
            BlendMode::Additive,
            premultiplied_alpha,
        ),
        multiply: create_pipeline(
            device,
            layout,
            shader,
            color_format,
            BlendMode::Multiply,
            premultiplied_alpha,
        ),
        screen: create_pipeline(
            device,
            layout,
            shader,
            color_format,
            BlendMode::Screen,
            premultiplied_alpha,
        ),
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    blend: BlendMode,
    premultiplied_alpha: bool,
) -> wgpu::RenderPipeline {
    let label = match (blend, premultiplied_alpha) {
        (BlendMode::Normal, false) => "spine2d-wgpu pipeline normal",
        (BlendMode::Additive, false) => "spine2d-wgpu pipeline additive",
        (BlendMode::Multiply, false) => "spine2d-wgpu pipeline multiply",
        (BlendMode::Screen, false) => "spine2d-wgpu pipeline screen",
        (BlendMode::Normal, true) => "spine2d-wgpu pipeline normal pma",
        (BlendMode::Additive, true) => "spine2d-wgpu pipeline additive pma",
        (BlendMode::Multiply, true) => "spine2d-wgpu pipeline multiply pma",
        (BlendMode::Screen, true) => "spine2d-wgpu pipeline screen pma",
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float32x2,
                    1 => Float32x2,
                    2 => Float32x4,
                    3 => Float32x4
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(blend_state(blend, premultiplied_alpha)),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn blend_state(blend: BlendMode, premultiplied_alpha: bool) -> wgpu::BlendState {
    use wgpu::{BlendComponent, BlendFactor, BlendOperation};

    // Mirrors the renderer oracle blend convention:
    // glBlendFuncSeparate(srcColorBlend, dstBlend, srcAlphaBlend, dstBlend)
    // where `srcAlphaBlend` is always ONE.
    let (src_color, dst) = match blend {
        BlendMode::Normal => (
            src_color_for_alpha(premultiplied_alpha),
            BlendFactor::OneMinusSrcAlpha,
        ),
        BlendMode::Additive => (src_color_for_alpha(premultiplied_alpha), BlendFactor::One),
        BlendMode::Multiply => (BlendFactor::Dst, BlendFactor::OneMinusSrcAlpha),
        BlendMode::Screen => (BlendFactor::One, BlendFactor::OneMinusSrc),
    };

    wgpu::BlendState {
        color: BlendComponent {
            src_factor: src_color,
            dst_factor: dst,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: dst,
            operation: BlendOperation::Add,
        },
    }
}

fn src_color_for_alpha(premultiplied_alpha: bool) -> wgpu::BlendFactor {
    if premultiplied_alpha {
        wgpu::BlendFactor::One
    } else {
        wgpu::BlendFactor::SrcAlpha
    }
}

pub trait TextureProvider {
    fn bind_group_for(&self, texture_path: &str) -> Option<&wgpu::BindGroup>;
}

pub struct HashMapTextureProvider {
    pub bind_groups: std::collections::HashMap<String, wgpu::BindGroup>,
}

impl TextureProvider for HashMapTextureProvider {
    fn bind_group_for(&self, texture_path: &str) -> Option<&wgpu::BindGroup> {
        self.bind_groups.get(texture_path)
    }
}

pub fn create_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("spine2d texture bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

pub fn create_sampler_for_atlas_page(
    device: &wgpu::Device,
    page: &spine2d::AtlasPage,
) -> wgpu::Sampler {
    let (min_filter, mipmap_filter) = to_wgpu_min_mipmap_filter(&page.min_filter);
    let mag_filter = to_wgpu_mag_filter(&page.mag_filter);
    let address_mode_u = to_wgpu_address_mode(page.wrap_u);
    let address_mode_v = to_wgpu_address_mode(page.wrap_v);

    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("spine2d atlas sampler"),
        mag_filter,
        min_filter,
        mipmap_filter,
        address_mode_u,
        address_mode_v,
        ..Default::default()
    })
}

fn to_wgpu_address_mode(wrap: spine2d::AtlasWrap) -> wgpu::AddressMode {
    match wrap {
        spine2d::AtlasWrap::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        spine2d::AtlasWrap::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        spine2d::AtlasWrap::Repeat => wgpu::AddressMode::Repeat,
    }
}

fn to_wgpu_mag_filter(filter: &spine2d::AtlasFilter) -> wgpu::FilterMode {
    match filter {
        spine2d::AtlasFilter::Unknown
        | spine2d::AtlasFilter::Nearest
        | spine2d::AtlasFilter::MipMapNearestNearest
        | spine2d::AtlasFilter::MipMapLinearNearest => wgpu::FilterMode::Nearest,
        spine2d::AtlasFilter::Linear
        | spine2d::AtlasFilter::MipMap
        | spine2d::AtlasFilter::MipMapNearestLinear
        | spine2d::AtlasFilter::MipMapLinearLinear => wgpu::FilterMode::Linear,
    }
}

fn to_wgpu_min_mipmap_filter(
    filter: &spine2d::AtlasFilter,
) -> (wgpu::FilterMode, wgpu::FilterMode) {
    match filter {
        spine2d::AtlasFilter::Unknown => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
        spine2d::AtlasFilter::Nearest => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
        spine2d::AtlasFilter::Linear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
        spine2d::AtlasFilter::MipMap | spine2d::AtlasFilter::MipMapLinearLinear => {
            (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear)
        }
        spine2d::AtlasFilter::MipMapNearestNearest => {
            (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
        }
        spine2d::AtlasFilter::MipMapNearestLinear => {
            (wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear)
        }
        spine2d::AtlasFilter::MipMapLinearNearest => {
            (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest)
        }
    }
}

const SHADER: &str = r#"
struct Globals {
  clip_from_world: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VsIn {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) light_color: vec4<f32>,
  @location(3) dark_color: vec4<f32>,
};

struct VsOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) light_color: vec4<f32>,
  @location(2) dark_color: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
  var out: VsOut;
  out.position = globals.clip_from_world * vec4<f32>(in.position, 0.0, 1.0);
  out.uv = in.uv;
  out.light_color = in.light_color;
  out.dark_color = in.dark_color;
  return out;
}

@group(1) @binding(0)
var tex: texture_2d<f32>;

@group(1) @binding(1)
var samp: sampler;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let tex_color = textureSample(tex, samp, in.uv);
  let alpha = tex_color.a * in.light_color.a;
  let rgb = ((tex_color.a - 1.0) * in.dark_color.a + 1.0 - tex_color.rgb) * in.dark_color.rgb
    + tex_color.rgb * in.light_color.rgb;
  return vec4<f32>(rgb, alpha);
}
"#;
