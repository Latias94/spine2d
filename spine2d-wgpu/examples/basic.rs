use spine2d::{Atlas, SkeletonData};
use spine2d_wgpu::{
    HashMapTextureProvider, SpineRenderer, create_sampler_for_atlas_page, create_texture_bind_group,
};
use std::collections::HashMap;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const SKELETON_JSON: &str = r#"
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
          "vertices": [-128,-128, 128,-128, 128,128, -128,128],
          "triangles": [0,1,2, 2,3,0]
        }
      }
    }
  },
  "animations": {}
}
"#;

const ATLAS: &str = r#"
page.png
size: 64,64

mesh0
  rotate: false
  xy: 0, 0
  size: 64, 64
"#;

struct App {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    renderer: Option<SpineRenderer>,
    textures: HashMapTextureProvider,
    draw_list: spine2d::DrawList,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            renderer: None,
            textures: HashMapTextureProvider {
                bind_groups: HashMap::new(),
            },
            draw_list: spine2d::DrawList::default(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("spine2d-wgpu basic"))
                .unwrap(),
        );

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: Default::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: Default::default(),
        }))
        .unwrap();

        let size = window.inner_size().max(PhysicalSize::new(1, 1));
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut renderer = SpineRenderer::new(&device, config.format);
        renderer.update_globals_ortho_centered(&queue, config.width as f32, config.height as f32);

        let atlas = ATLAS.parse::<Atlas>().unwrap();
        let sampler = create_sampler_for_atlas_page(&device, &atlas.get_pages()[0]);

        // Simple procedural RGBA texture (64x64): a UV gradient to make UV mistakes obvious.
        let w = 64u32;
        let h = 64u32;
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                pixels[i] = (x * 255 / (w - 1)) as u8;
                pixels[i + 1] = (y * 255 / (h - 1)) as u8;
                pixels[i + 2] = 200;
                pixels[i + 3] = 255;
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("page texture"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * w),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = create_texture_bind_group(
            &device,
            renderer.texture_bind_group_layout(),
            &view,
            &sampler,
        );

        self.textures
            .bind_groups
            .insert("page.png".to_string(), bind_group);

        let data = SkeletonData::from_json_str(SKELETON_JSON).unwrap();
        let mut skeleton = spine2d::Skeleton::new(data);
        skeleton.setup_pose();
        skeleton.update_world_transform_with_physics(spine2d::Physics::None);
        self.draw_list = spine2d::build_draw_list_with_atlas(&skeleton, &atlas);

        renderer.upload(&device, &queue, &self.draw_list);

        window.request_redraw();
        self.window = Some(window);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.renderer = Some(renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let Some(surface) = self.surface.as_ref() else {
                    return;
                };
                let Some(device) = self.device.as_ref() else {
                    return;
                };
                let Some(queue) = self.queue.as_ref() else {
                    return;
                };
                let Some(config) = self.config.as_mut() else {
                    return;
                };
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                surface.configure(device, config);
                if let Some(renderer) = self.renderer.as_ref() {
                    renderer.update_globals_ortho_centered(
                        queue,
                        config.width as f32,
                        config.height as f32,
                    );
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let Some(surface) = self.surface.as_ref() else {
                    return;
                };
                let Some(device) = self.device.as_ref() else {
                    return;
                };
                let Some(queue) = self.queue.as_ref() else {
                    return;
                };
                let Some(config) = self.config.as_ref() else {
                    return;
                };
                let Some(renderer) = self.renderer.as_ref() else {
                    return;
                };

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(device, config);
                        return;
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.12,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    renderer.render(&mut pass, &self.draw_list, &self.textures);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
