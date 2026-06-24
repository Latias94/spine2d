use egui_wgpu::Renderer as EguiRenderer;
use egui_wgpu::RendererOptions as EguiRendererOptions;
use egui_wgpu::ScreenDescriptor as EguiScreenDescriptor;
use spine2d::{AnimationState, AnimationStateData, Atlas, DrawList, Skeleton, SkeletonData};
use spine2d_wgpu::{
    HashMapTextureProvider, SpineRenderer, create_sampler_for_atlas_page, create_texture_bind_group,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

fn sanitize_example_arg(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else {
            break;
        }
    }
    if out.is_empty() {
        trimmed.to_string()
    } else {
        out
    }
}

fn upstream_examples_root() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("SPINE2D_UPSTREAM_EXAMPLES_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Ok(p);
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        root.join("../assets/spine-runtimes/examples"),
        root.join("../third_party/spine-runtimes/examples"),
        root.join("../.cache/spine-runtimes/examples"),
    ];
    for p in candidates {
        if p.is_dir() {
            return Ok(p);
        }
    }

    Err("Upstream Spine examples not found.\n\
Run `python3 ./scripts/fetch_spine_runtimes_examples.py --mode export --scope tests` or set \
`SPINE2D_UPSTREAM_EXAMPLES_DIR` to `<spine-runtimes>/examples`."
        .to_string())
}

fn available_examples() -> Vec<String> {
    let Ok(root) = upstream_examples_root() else {
        return Vec::new();
    };
    let Ok(rd) = std::fs::read_dir(&root) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.join("export").is_dir() {
            out.push(name.to_string());
        }
    }
    out.sort();
    out
}

fn first_file_with_extension(dir: &Path, ext: &str) -> Option<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return None;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == ext) {
            return Some(path);
        }
    }
    None
}

fn preferred_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|p| p.is_file()).cloned()
}

fn pick_export_files(example: &str, prefer_pma_atlas: bool) -> Result<(PathBuf, PathBuf), String> {
    let examples = upstream_examples_root()?;
    let export_dir = examples.join(example).join("export");
    if !export_dir.is_dir() {
        return Err(format!(
            "Missing export dir: {export_dir:?}\n\
Example name should match a directory under `<spine-runtimes>/examples/` (e.g. `spineboy`, `alien`)."
        ));
    }

    let json = preferred_existing(&[
        export_dir.join(format!("{example}-pro.json")),
        export_dir.join(format!("{example}-ess.json")),
        export_dir.join(format!("{example}.json")),
    ])
    .or_else(|| first_file_with_extension(&export_dir, "json"))
    .ok_or_else(|| format!("No json found in {export_dir:?}"))?;

    let atlas_candidates = if prefer_pma_atlas {
        vec![
            export_dir.join(format!("{example}-pma.atlas")),
            export_dir.join(format!("{example}-run.atlas")),
            export_dir.join(format!("{example}.atlas")),
        ]
    } else {
        vec![
            export_dir.join(format!("{example}.atlas")),
            export_dir.join(format!("{example}-run.atlas")),
            export_dir.join(format!("{example}-pma.atlas")),
        ]
    };

    let atlas = preferred_existing(&atlas_candidates)
        .or_else(|| first_file_with_extension(&export_dir, "atlas"))
        .ok_or_else(|| {
            format!(
                "No atlas found in {export_dir:?}\n\
You probably imported only JSON. Re-import with:\n\
  `python3 ./scripts/fetch_spine_runtimes_examples.py --mode export --scope tests`"
            )
        })?;

    Ok((json, atlas))
}

fn load_png_rgba8(path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read png {path:?}: {e}"))?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .map_err(|e| format!("decode png {path:?}: {e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((w, h, rgba.into_raw()))
}

const SKIN_NONE_SENTINEL: &str = "__none__";

#[derive(Clone, Debug)]
struct UiSelection {
    example: String,
    animation: String,
    skin: String,
    prefer_pma_atlas: bool,
    speed: f32,
    margin: f32,
}

#[derive(Copy, Clone, Debug)]
struct Bounds2 {
    min: [f32; 2],
    max: [f32; 2],
}

fn bounds_from_draw_list(draw_list: &DrawList) -> Option<Bounds2> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for v in &draw_list.vertices {
        min_x = min_x.min(v.position[0]);
        min_y = min_y.min(v.position[1]);
        max_x = max_x.max(v.position[0]);
        max_y = max_y.max(v.position[1]);
    }

    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        Some(Bounds2 {
            min: [min_x, min_y],
            max: [max_x, max_y],
        })
    } else {
        None
    }
}

fn clip_from_world_fit_bounds(
    bounds: Bounds2,
    viewport_width: u32,
    viewport_height: u32,
    margin: f32,
) -> [[f32; 4]; 4] {
    let vw = viewport_width.max(1) as f32;
    let vh = viewport_height.max(1) as f32;
    let margin = margin.clamp(0.1, 1.0);

    let world_w = (bounds.max[0] - bounds.min[0]).abs().max(1.0e-3);
    let world_h = (bounds.max[1] - bounds.min[1]).abs().max(1.0e-3);
    let cx = 0.5 * (bounds.min[0] + bounds.max[0]);
    let cy = 0.5 * (bounds.min[1] + bounds.max[1]);

    // Keep world units isotropic in pixels, then convert to clip.
    let scale_px = (vw * margin / world_w).min(vh * margin / world_h);
    let sx = 2.0 * scale_px / vw;
    let sy = 2.0 * scale_px / vh;

    // WGSL matrices are column-major; translation lives in the last column.
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-cx * sx, -cy * sy, 0.0, 1.0],
    ]
}

struct App {
    example: String,
    animation: Option<String>,
    speed: f32,
    fit_margin: f32,
    prefer_pma_atlas: bool,
    json_path: PathBuf,
    atlas_path: PathBuf,
    initial_bounds: Option<Bounds2>,

    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    renderer: Option<SpineRenderer>,
    textures: HashMapTextureProvider,

    atlas: Option<Atlas>,
    data: Option<Arc<SkeletonData>>,
    skeleton: Option<Skeleton>,
    state: Option<AnimationState>,

    draw_list: spine2d::DrawList,
    last_frame: Option<Instant>,
    debug_time: bool,
    debug_accum: f32,
    debug_frames: u32,

    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<EguiRenderer>,

    examples: Vec<String>,
    ui: UiSelection,
    pending_reload: Option<UiSelection>,
    paused: bool,
    last_error: Option<String>,
    needs_fit: bool,
}

impl App {
    fn new(
        example: String,
        animation: Option<String>,
        speed: f32,
        fit_margin: f32,
        prefer_pma_atlas: bool,
        json_path: PathBuf,
        atlas_path: PathBuf,
    ) -> Self {
        let egui_ctx = egui::Context::default();
        let ui_animation = animation.clone().unwrap_or_default();
        Self {
            example: example.clone(),
            animation,
            speed: if speed.is_finite() { speed } else { 1.0 },
            fit_margin: if fit_margin.is_finite() {
                fit_margin.clamp(0.1, 1.0)
            } else {
                0.9
            },
            prefer_pma_atlas,
            json_path,
            atlas_path,
            initial_bounds: None,
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            renderer: None,
            textures: HashMapTextureProvider {
                bind_groups: HashMap::new(),
            },
            atlas: None,
            data: None,
            skeleton: None,
            state: None,
            draw_list: spine2d::DrawList::default(),
            last_frame: None,
            debug_time: std::env::var("SPINE2D_DEBUG_TIME").is_ok(),
            debug_accum: 0.0,
            debug_frames: 0,

            egui_ctx: egui_ctx.clone(),
            egui_state: None,
            egui_renderer: None,

            examples: available_examples(),
            ui: UiSelection {
                example,
                animation: ui_animation,
                skin: String::new(),
                prefer_pma_atlas,
                speed,
                margin: fit_margin,
            },
            pending_reload: None,
            paused: false,
            last_error: None,
            needs_fit: true,
        }
    }

    fn recompute_fit(&mut self) {
        let (Some(renderer), Some(queue), Some(config)) = (
            self.renderer.as_ref(),
            self.queue.as_ref(),
            self.config.as_ref(),
        ) else {
            return;
        };

        if let Some(bounds) = self.initial_bounds {
            renderer.update_globals_matrix(
                queue,
                clip_from_world_fit_bounds(bounds, config.width, config.height, self.fit_margin),
            );
        } else {
            renderer.update_globals_ortho_centered(
                queue,
                config.width as f32,
                config.height as f32,
            );
        }
    }

    fn queue_reload(&mut self) {
        self.pending_reload = Some(self.ui.clone());
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn ui_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("spine2d")
            .default_pos([8.0, 8.0])
            .default_width(420.0)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.label("Controls");
                ui.separator();

                egui::Grid::new("spine2d_controls_grid")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Example");
                        let mut example = self.ui.example.clone();
                        egui::ComboBox::from_id_salt("example_combo")
                            .width(260.0)
                            .selected_text(&example)
                            .show_ui(ui, |ui| {
                                for name in &self.examples {
                                    ui.selectable_value(&mut example, name.clone(), name);
                                }
                            });
                        if example != self.ui.example {
                            self.ui.example = example;
                            // New skeleton almost certainly has a different animation/skin set.
                            // Reset to "(auto)" so we don't carry an invalid name across examples.
                            self.ui.animation.clear();
                            self.ui.skin.clear(); // "(auto)"
                            self.queue_reload();
                        }
                        ui.end_row();

                        ui.label("Anim");
                        let anims = self
                            .data
                            .as_ref()
                            .map(|d| {
                                d.animations
                                    .iter()
                                    .map(|a| a.name.clone())
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        let mut anims = anims;
                        anims.sort();
                        let mut animation = self.ui.animation.clone();
                        egui::ComboBox::from_id_salt("anim_combo")
                            .width(260.0)
                            .selected_text(if animation.is_empty() {
                                "(auto)"
                            } else {
                                &animation
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut animation, String::new(), "(auto)");
                                for name in &anims {
                                    ui.selectable_value(&mut animation, name.clone(), name);
                                }
                            });
                        if animation != self.ui.animation {
                            self.ui.animation = animation;
                            self.queue_reload();
                        }
                        ui.end_row();

                        ui.label("Skin");
                        let skins = self
                            .data
                            .as_ref()
                            .map(|d| d.skins.keys().cloned().collect::<Vec<_>>())
                            .unwrap_or_default();
                        let mut skins = skins;
                        skins.sort();
                        let mut skin = self.ui.skin.clone();
                        egui::ComboBox::from_id_salt("skin_combo")
                            .width(260.0)
                            .selected_text(match skin.as_str() {
                                "" => "(auto)",
                                SKIN_NONE_SENTINEL => "(none)",
                                _ => &skin,
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut skin, String::new(), "(auto)");
                                ui.selectable_value(
                                    &mut skin,
                                    SKIN_NONE_SENTINEL.to_string(),
                                    "(none)",
                                );
                                for name in &skins {
                                    ui.selectable_value(&mut skin, name.clone(), name);
                                }
                            });
                        if skin != self.ui.skin {
                            self.ui.skin = skin;
                            self.queue_reload();
                        }
                        ui.end_row();

                        ui.label("Speed");
                        if ui
                            .add(egui::Slider::new(&mut self.ui.speed, 0.0..=2.0).step_by(0.05))
                            .changed()
                        {
                            self.speed = self.ui.speed;
                        }
                        ui.end_row();

                        ui.label("Margin");
                        if ui
                            .add(egui::Slider::new(&mut self.ui.margin, 0.1..=1.0).step_by(0.01))
                            .changed()
                        {
                            self.fit_margin = self.ui.margin;
                            self.needs_fit = true;
                        }
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui
                        .checkbox(&mut self.ui.prefer_pma_atlas, "Prefer PMA")
                        .changed()
                    {
                        self.queue_reload();
                    }
                    if ui.button("Reload").clicked() {
                        self.queue_reload();
                    }
                    if ui.button("Re-fit").clicked() {
                        self.needs_fit = true;
                    }
                    if ui
                        .button(if self.paused { "Play" } else { "Pause" })
                        .clicked()
                    {
                        self.paused = !self.paused;
                    }
                });

                if let Some(err) = self.last_error.as_deref() {
                    ui.separator();
                    ui.colored_label(egui::Color32::LIGHT_RED, err);
                }
            });

        if self.needs_fit {
            self.needs_fit = false;
            self.recompute_fit();
        }
    }

    fn apply_pending_reload(&mut self) {
        let Some(next) = self.pending_reload.take() else {
            return;
        };
        let (Some(window), Some(device), Some(queue), Some(renderer), Some(config)) = (
            self.window.as_ref(),
            self.device.as_ref(),
            self.queue.as_ref(),
            self.renderer.as_mut(),
            self.config.as_ref(),
        ) else {
            self.pending_reload = Some(next);
            return;
        };

        let example = sanitize_example_arg(&next.example);
        let (json_path, atlas_path) = match pick_export_files(&example, next.prefer_pma_atlas) {
            Ok(v) => v,
            Err(e) => {
                self.last_error = Some(e);
                return;
            }
        };

        let atlas_dir = match atlas_path.parent() {
            Some(p) => p,
            None => {
                self.last_error = Some(format!("atlas path has no parent dir: {atlas_path:?}"));
                return;
            }
        };

        let json = match std::fs::read_to_string(&json_path) {
            Ok(s) => s,
            Err(e) => {
                self.last_error = Some(format!("read json {json_path:?}: {e}"));
                return;
            }
        };
        let data = match SkeletonData::from_json_str(&json) {
            Ok(d) => d,
            Err(e) => {
                self.last_error = Some(format!("parse json {json_path:?}: {e}"));
                return;
            }
        };

        let atlas_text = match std::fs::read_to_string(&atlas_path) {
            Ok(s) => s,
            Err(e) => {
                self.last_error = Some(format!("read atlas {atlas_path:?}: {e}"));
                return;
            }
        };
        let atlas = match Atlas::parse(&atlas_text) {
            Ok(a) => a,
            Err(e) => {
                self.last_error = Some(format!("parse atlas {atlas_path:?}: {e}"));
                return;
            }
        };

        let mut skeleton = Skeleton::new(data.clone());
        let chosen_skin = {
            let requested = next.skin.trim();
            if requested == SKIN_NONE_SENTINEL {
                None
            } else if !requested.is_empty() && data.skins.contains_key(requested) {
                Some(requested.to_string())
            } else {
                choose_default_skin(&example, &data)
            }
        };
        if let Some(skin_name) = chosen_skin.as_deref() {
            skeleton.set_skin(Some(skin_name));
        }
        skeleton.setup_pose();
        skeleton.update_world_transform_with_physics(spine2d::Physics::None);

        let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
        let chosen_animation = {
            let requested = next.animation.trim();
            if !requested.is_empty() && data.animations.iter().any(|a| a.name == requested) {
                Some(requested.to_string())
            } else {
                choose_default_animation(&data)
            }
        };
        let Some(chosen_animation) = chosen_animation else {
            self.last_error = Some("No animations in skeleton data.".to_string());
            return;
        };
        state.set_animation(0, chosen_animation.as_str(), true);

        skeleton.setup_pose();
        state.apply(&mut skeleton);
        skeleton.update_world_transform_with_physics(spine2d::Physics::None);

        let mut textures = HashMapTextureProvider {
            bind_groups: HashMap::new(),
        };
        for page in &atlas.pages {
            let image_path = atlas_dir.join(&page.name);
            let (w, h, pixels) = match load_png_rgba8(&image_path) {
                Ok(v) => v,
                Err(e) => {
                    self.last_error = Some(e);
                    return;
                }
            };

            let sampler = create_sampler_for_atlas_page(device, page);
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("spine2d page texture"),
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
                device,
                renderer.texture_bind_group_layout(),
                &view,
                &sampler,
            );
            textures.bind_groups.insert(page.name.clone(), bind_group);
        }

        self.draw_list.clear();
        spine2d::append_draw_list_with_atlas(&mut self.draw_list, &skeleton, &atlas);
        self.initial_bounds = bounds_from_draw_list(&self.draw_list);
        self.example = example;
        self.animation = Some(chosen_animation);
        self.speed = next.speed;
        self.fit_margin = next.margin;
        self.prefer_pma_atlas = next.prefer_pma_atlas;
        self.json_path = json_path;
        self.atlas_path = atlas_path;
        self.textures = textures;
        self.atlas = Some(atlas);
        let data_ref = data.clone();
        self.data = Some(data);
        self.skeleton = Some(skeleton);
        self.state = Some(state);
        self.last_frame = Some(Instant::now());
        self.last_error = None;

        self.ui.example = self.example.clone();
        // If the chosen animation doesn't exist in the newly loaded skeleton, fall back to auto.
        self.ui.animation = self
            .animation
            .clone()
            .filter(|name| data_ref.animations.iter().any(|a| a.name == *name))
            .unwrap_or_default();
        self.ui.skin = chosen_skin.unwrap_or_else(|| SKIN_NONE_SENTINEL.to_string());
        self.ui.prefer_pma_atlas = self.prefer_pma_atlas;
        self.ui.speed = self.speed;
        self.ui.margin = self.fit_margin;

        window.set_title(&format!("spine2d-wgpu: {}", self.example));
        if let Some(bounds) = self.initial_bounds {
            renderer.update_globals_matrix(
                queue,
                clip_from_world_fit_bounds(bounds, config.width, config.height, self.fit_margin),
            );
        } else {
            renderer.update_globals_ortho_centered(
                queue,
                config.width as f32,
                config.height as f32,
            );
        }
        renderer.upload(device, queue, &self.draw_list);
        window.request_redraw();
    }
}

fn choose_default_animation(data: &SkeletonData) -> Option<String> {
    // Prefer more "stable" animations to avoid defaulting to something very fast (e.g. run).
    for name in ["idle", "walk", "run"] {
        if data.animations.iter().any(|a| a.name == name) {
            return Some(name.to_string());
        }
    }
    data.animations.first().map(|a| a.name.clone())
}

fn choose_default_skin(example: &str, data: &SkeletonData) -> Option<String> {
    // Match the web demo defaults for common examples.
    let recommended = match example {
        "goblins" => Some("goblin"),
        "mix-and-match" => Some("full-skins/girl-blue-cape"),
        "chibi-stickers" => Some("spineboy"),
        _ => None,
    };
    if let Some(name) = recommended
        && data.skins.contains_key(name)
    {
        return Some(name.to_string());
    }

    if data.skins.contains_key("default") {
        return Some("default".to_string());
    }

    // Fallback: pick the skin with the most attachments.
    let mut best: Option<(&str, usize)> = None;
    for (name, skin) in &data.skins {
        let count = skin.attachments.iter().map(|m| m.len()).sum::<usize>();
        if count == 0 {
            continue;
        }
        match best {
            None => best = Some((name.as_str(), count)),
            Some((_, best_count)) if count > best_count => best = Some((name.as_str(), count)),
            _ => {}
        }
    }
    best.map(|(n, _)| n.to_string())
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        self.examples = available_examples();

        let atlas_dir = self.atlas_path.parent().expect("atlas parent dir");

        let json = match std::fs::read_to_string(&self.json_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read json {:?}: {e}", self.json_path);
                event_loop.exit();
                return;
            }
        };
        let data = match SkeletonData::from_json_str(&json) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("parse json {:?}: {e}", self.json_path);
                event_loop.exit();
                return;
            }
        };

        let atlas_text = match std::fs::read_to_string(&self.atlas_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read atlas {:?}: {e}", self.atlas_path);
                event_loop.exit();
                return;
            }
        };
        let atlas = match Atlas::parse(&atlas_text) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("parse atlas {:?}: {e}", self.atlas_path);
                event_loop.exit();
                return;
            }
        };

        let mut skeleton = Skeleton::new(data.clone());
        skeleton.setup_pose();
        skeleton.update_world_transform_with_physics(spine2d::Physics::None);

        let mut state = AnimationState::new(AnimationStateData::new(data.clone()));
        let chosen_animation = if let Some(anim_name) = self.animation.as_deref() {
            Some(anim_name.to_string())
        } else {
            // Prefer more "stable" animations to avoid defaulting to something very fast (e.g. run).
            let preferred = ["idle", "walk", "run"];
            let mut found = None;
            for name in preferred {
                if data.animations.iter().any(|a| a.name == name) {
                    found = Some(name.to_string());
                    break;
                }
            }
            found.or_else(|| data.animations.first().map(|a| a.name.clone()))
        };

        let Some(chosen_animation) = chosen_animation else {
            eprintln!("No animations in skeleton data.");
            event_loop.exit();
            return;
        };

        state.set_animation(0, chosen_animation.as_str(), true);

        // IMPORTANT: Our current runtime path assumes a clean base pose each frame
        // (otherwise constraints/local transforms can accumulate and look like "no keyframes").
        skeleton.setup_pose();
        state.apply(&mut skeleton);
        skeleton.update_world_transform_with_physics(spine2d::Physics::None);

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(format!("spine2d-wgpu: {}", self.example)),
                )
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
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
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

        let mut textures = HashMapTextureProvider {
            bind_groups: HashMap::new(),
        };
        for page in &atlas.pages {
            let image_path = atlas_dir.join(&page.name);
            let (w, h, pixels) = match load_png_rgba8(&image_path) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{e}");
                    event_loop.exit();
                    return;
                }
            };

            let sampler = create_sampler_for_atlas_page(&device, page);
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("spine2d page texture"),
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
            textures.bind_groups.insert(page.name.clone(), bind_group);
        }

        self.draw_list = spine2d::build_draw_list_with_atlas(&skeleton, &atlas);
        self.initial_bounds = bounds_from_draw_list(&self.draw_list);
        if let Some(bounds) = self.initial_bounds {
            renderer.update_globals_matrix(
                &queue,
                clip_from_world_fit_bounds(bounds, config.width, config.height, self.fit_margin),
            );
        } else {
            renderer.update_globals_ortho_centered(
                &queue,
                config.width as f32,
                config.height as f32,
            );
        }
        renderer.upload(&device, &queue, &self.draw_list);

        let egui_state = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            window.theme(),
            None,
        );
        let egui_renderer =
            EguiRenderer::new(&device, config.format, EguiRendererOptions::default());

        window.request_redraw();
        self.window = Some(window);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.renderer = Some(renderer);
        self.textures = textures;
        self.atlas = Some(atlas);
        self.data = Some(data);
        self.skeleton = Some(skeleton);
        self.state = Some(state);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
        self.ui = UiSelection {
            example: self.example.clone(),
            animation: chosen_animation,
            skin: String::new(),
            prefer_pma_atlas: self.prefer_pma_atlas,
            speed: self.speed,
            margin: self.fit_margin,
        };
        self.last_frame = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let window_ref = window.as_ref();

        if let Some(egui_state) = self.egui_state.as_mut() {
            let response = egui_state.on_window_event(window_ref, &event);
            if response.repaint {
                window_ref.request_redraw();
            }
        }

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
                    if let Some(bounds) = self.initial_bounds {
                        renderer.update_globals_matrix(
                            queue,
                            clip_from_world_fit_bounds(
                                bounds,
                                config.width,
                                config.height,
                                self.fit_margin,
                            ),
                        );
                    } else {
                        renderer.update_globals_ortho_centered(
                            queue,
                            config.width as f32,
                            config.height as f32,
                        );
                    }
                }
                window_ref.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.apply_pending_reload();

                let now = Instant::now();
                let mut dt = self
                    .last_frame
                    .replace(now)
                    .map(|t| (now - t).as_secs_f32())
                    .unwrap_or(0.0)
                    .min(1.0 / 15.0);
                dt = (dt * self.speed).max(0.0);

                if self.debug_time {
                    self.debug_accum += dt;
                    self.debug_frames += 1;
                    if self.debug_accum >= 1.0 {
                        let fps = self.debug_frames as f32 / self.debug_accum.max(1.0e-6);
                        eprintln!("dt={:.4} speed={:.2} fps~{:.1}", dt, self.speed, fps);
                        self.debug_accum = 0.0;
                        self.debug_frames = 0;
                    }
                }
                let Some(raw_input) = self
                    .egui_state
                    .as_mut()
                    .map(|s| s.take_egui_input(window_ref))
                else {
                    return;
                };
                let egui_ctx = self.egui_ctx.clone();
                let egui::FullOutput {
                    platform_output,
                    textures_delta,
                    shapes,
                    pixels_per_point,
                    ..
                } = egui_ctx.run(raw_input, |ctx| self.ui_panel(ctx));
                if let Some(egui_state) = self.egui_state.as_mut() {
                    egui_state.handle_platform_output(window_ref, platform_output);
                }
                let clipped_primitives = egui_ctx.tessellate(shapes, pixels_per_point);

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
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };
                let Some(egui_renderer) = self.egui_renderer.as_mut() else {
                    return;
                };

                if !self.paused
                    && let (Some(atlas), Some(skeleton), Some(state)) = (
                        self.atlas.as_ref(),
                        self.skeleton.as_mut(),
                        self.state.as_mut(),
                    )
                {
                    state.update(dt);
                    skeleton.setup_pose();
                    state.apply(skeleton);
                    skeleton.update_world_transform_with_physics(spine2d::Physics::None);

                    self.draw_list.clear();
                    spine2d::append_draw_list_with_atlas(&mut self.draw_list, skeleton, atlas);
                    renderer.upload(device, queue, &self.draw_list);
                }

                for (id, image_delta) in &textures_delta.set {
                    egui_renderer.update_texture(device, queue, *id, image_delta);
                }
                for id in &textures_delta.free {
                    egui_renderer.free_texture(id);
                }

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

                let egui_screen_descriptor = EguiScreenDescriptor {
                    size_in_pixels: [config.width, config.height],
                    pixels_per_point: window.scale_factor() as f32,
                };
                let callback_cmds = egui_renderer.update_buffers(
                    device,
                    queue,
                    &mut encoder,
                    &clipped_primitives,
                    &egui_screen_descriptor,
                );

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    renderer.render(&mut pass, &self.draw_list, &self.textures);
                }

                {
                    let desc = wgpu::RenderPassDescriptor {
                        label: Some("egui pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    };
                    let mut pass = encoder.begin_render_pass(&desc).forget_lifetime();
                    egui_renderer.render(&mut pass, &clipped_primitives, &egui_screen_descriptor);
                }

                queue.submit(
                    callback_cmds
                        .into_iter()
                        .chain(std::iter::once(encoder.finish())),
                );
                frame.present();

                window_ref.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let raw_example = args.next().unwrap_or_else(|| "spineboy".to_string());
    let mut animation: Option<String> = None;
    let mut speed: f32 = 1.0;
    let mut fit_margin: f32 = 0.9;
    let mut prefer_pma_atlas = true;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                eprintln!(
                    "Usage: spine_runtimes <example> [animation] [--speed <factor>]\n\
Examples:\n\
  spine_runtimes spineboy\n\
  spine_runtimes spineboy run\n\
  spine_runtimes spineboy run --speed 0.7\n\
  spine_runtimes spineboy run --margin 0.75\n\
  spine_runtimes spineboy run --no-pma\n\
\n\
Notes:\n\
  - If animation is omitted, it prefers idle/walk/run, else the first animation in JSON.\n\
  - Use `python3 ./scripts/fetch_spine_runtimes_examples.py --mode export --scope tests` to fetch assets."
                );
                return;
            }
            "--speed" => {
                let Some(v) = args.next() else {
                    eprintln!("--speed requires a value");
                    std::process::exit(2);
                };
                speed = v.parse::<f32>().unwrap_or(1.0);
            }
            "--margin" => {
                let Some(v) = args.next() else {
                    eprintln!("--margin requires a value");
                    std::process::exit(2);
                };
                fit_margin = v.parse::<f32>().unwrap_or(0.9);
            }
            "--no-pma" => {
                prefer_pma_atlas = false;
            }
            _ => {
                if animation.is_none() {
                    animation = Some(arg);
                } else {
                    eprintln!("Unexpected argument: {arg:?}");
                    std::process::exit(2);
                }
            }
        }
    }
    let example = sanitize_example_arg(&raw_example);
    if example != raw_example {
        eprintln!("Example argument normalized: {raw_example:?} -> {example:?}");
    }
    let (json_path, atlas_path) = match pick_export_files(&example, prefer_pma_atlas) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(
        example,
        animation,
        speed,
        fit_margin,
        prefer_pma_atlas,
        json_path,
        atlas_path,
    );
    event_loop.run_app(&mut app).unwrap();
}
