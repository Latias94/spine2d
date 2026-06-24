#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;
    use std::rc::Rc;

    use serde::Deserialize;
    use spine2d::{AnimationState, AnimationStateData, Atlas, DrawList, Skeleton, SkeletonData};
    use spine2d_wgpu::{
        HashMapTextureProvider, SpineRenderer, create_sampler_for_atlas_page,
        create_texture_bind_group,
    };
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen_futures::spawn_local;

    const UPSTREAM_MANIFEST_URL: &str = "assets/spine-runtimes/web_manifest.json";

    #[derive(Clone, Debug, Deserialize)]
    struct WebManifest {
        version: u32,
        #[serde(default)]
        base: Option<String>,
        examples: Vec<WebManifestExample>,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct WebManifestExample {
        name: String,
        skeleton: String,
        atlas: String,
    }

    struct ExampleBundle {
        example_name: String,
        atlas: Atlas,
        data: std::sync::Arc<SkeletonData>,
        page_images: Vec<(String, Vec<u8>)>,
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("missing window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("missing document"))?;

        let canvas: web_sys::HtmlCanvasElement = element_by_id(&document, "spine2d-canvas")?;

        spawn_local(async move {
            if let Err(e) = run(document, canvas).await {
                log::error!("spine2d-web init failed: {e:?}");
            }
        });

        Ok(())
    }

    async fn run(
        document: web_sys::Document,
        canvas: web_sys::HtmlCanvasElement,
    ) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("missing window"))?;
        // Keep the CSS size (logical pixels) controlled by CSS, and set the backing buffer size
        // (physical pixels) based on `devicePixelRatio`.
        let (width, height) = physical_canvas_size(&canvas);
        canvas.set_width(width);
        canvas.set_height(height);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| JsValue::from_str(&format!("create_surface: {e:?}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| JsValue::from_str(&format!("request_adapter: {e:?}")))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| JsValue::from_str(&format!("request_device: {e:?}")))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .first()
            .copied()
            .ok_or_else(|| JsValue::from_str("surface has no formats"))?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            desired_maximum_frame_latency: 2,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let renderer = SpineRenderer::new(&device, config.format);
        renderer.update_globals_ortho_centered(&queue, width as f32, height as f32);

        let manifest = load_upstream_manifest().await.ok();
        let requested_example = query_param(&window, "example");
        let requested_anim = query_param(&window, "anim");
        let requested_skin = query_param(&window, "skin");

        let bundle = if let Some((m, entry)) = manifest.as_ref().and_then(|m| {
            let entry = choose_manifest_example(m, requested_example.as_deref());
            entry.map(|e| (m, e))
        }) {
            match load_manifest_example_bundle(m, entry).await {
                Ok(bundle) => Some(bundle),
                Err(e) => {
                    log::warn!("failed to load upstream example bundle: {e:?}");
                    None
                }
            }
        } else {
            None
        };

        let bundle = match bundle {
            Some(bundle) => bundle,
            None => match load_demo_bundle().await {
                Ok(bundle) => bundle,
                Err(e) => {
                    log::warn!("falling back to embedded demo assets: {e:?}");
                    ExampleBundle {
                        example_name: "embedded".to_string(),
                        atlas: Atlas::parse(ATLAS)
                            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?,
                        data: SkeletonData::from_json_str(SKELETON_JSON)
                            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?,
                        page_images: vec![],
                    }
                }
            },
        };

        let default_animation = pick_default_animation(&bundle.data, requested_anim.as_deref());
        let default_skin = pick_default_skin(
            &bundle.example_name,
            &bundle.data,
            requested_skin.as_deref(),
        );

        let mut skeleton = Skeleton::new(bundle.data.clone());
        if let Some(skin) = default_skin.as_deref() {
            skeleton.set_skin(Some(skin));
        }
        skeleton.setup_pose();
        skeleton.update_world_transform_with_physics(spine2d::Physics::None);

        let state_data = AnimationStateData::new(bundle.data.clone());
        let mut state = AnimationState::new(state_data);
        state.set_animation(0, default_animation.as_str(), true);

        let textures = build_textures(
            &device,
            &queue,
            &renderer,
            &bundle.atlas,
            bundle.page_images,
        );

        let draw_list = DrawList::default();

        let state = Rc::new(RefCell::new(WebState {
            canvas,
            surface,
            device,
            queue,
            config,
            renderer,
            atlas: bundle.atlas,
            data: bundle.data,
            skeleton,
            state,
            textures,
            draw_list,
            initial_bounds: None,
            fit_margin: 0.9,
            last_ts_ms: None,
            paused: false,
            speed: 1.0,
            current_animation: default_animation,
            manifest,
            current_example: bundle.example_name,
            current_skin: default_skin,
        }));

        init_ui(&document, state.clone())?;

        let raf = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
        let raf2 = raf.clone();
        *raf2.borrow_mut() = Some(Closure::wrap(Box::new(move |ts_ms: f64| {
            let mut st = state.borrow_mut();
            if let Err(e) = st.render(ts_ms) {
                log::error!("render: {e:?}");
            }

            let window = web_sys::window().expect("missing window");
            window
                .request_animation_frame(
                    raf.borrow()
                        .as_ref()
                        .expect("missing closure")
                        .as_ref()
                        .unchecked_ref(),
                )
                .expect("requestAnimationFrame");
        }) as Box<dyn FnMut(f64)>));

        web_sys::window()
            .ok_or_else(|| JsValue::from_str("missing window"))?
            .request_animation_frame(
                raf2.borrow()
                    .as_ref()
                    .expect("missing closure")
                    .as_ref()
                    .unchecked_ref(),
            )?;

        Ok(())
    }

    fn element_by_id<T: JsCast>(document: &web_sys::Document, id: &str) -> Result<T, JsValue> {
        let el = document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("missing element #{id}")))?;
        el.dyn_into::<T>()
            .map_err(|_| JsValue::from_str(&format!("element #{id} has unexpected type")))
    }

    fn query_param(window: &web_sys::Window, key: &str) -> Option<String> {
        let search = window.location().search().ok()?;
        let search = search.strip_prefix('?').unwrap_or(&search);
        if search.is_empty() {
            return None;
        }

        for part in search.split('&') {
            let (k, v) = part.split_once('=').unwrap_or((part, ""));
            if k != key {
                continue;
            }
            let v = v.replace('+', " ");
            if let Ok(v) = js_sys::decode_uri_component(&v) {
                if let Some(v) = v.as_string() {
                    return Some(v);
                }
            }
            return Some(v);
        }
        None
    }

    fn choose_manifest_example<'a>(
        manifest: &'a WebManifest,
        requested: Option<&str>,
    ) -> Option<&'a WebManifestExample> {
        if manifest.examples.is_empty() {
            return None;
        }
        if let Some(name) = requested {
            if let Some(found) = manifest.examples.iter().find(|e| e.name == name) {
                return Some(found);
            }
        }
        manifest.examples.first()
    }

    fn pick_default_animation(data: &SkeletonData, requested: Option<&str>) -> String {
        if let Some(name) = requested {
            if data.animation(name).is_some() {
                return name.to_string();
            }
        }

        if let Some(name) = recommended_animation(data) {
            return name;
        }

        for name in ["run", "walk", "idle", "spin"] {
            if data.animation(name).is_some() {
                return name.to_string();
            }
        }
        data.animations
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "spin".to_string())
    }

    fn pick_default_skin(
        example_name: &str,
        data: &SkeletonData,
        requested: Option<&str>,
    ) -> Option<String> {
        if let Some(name) = requested {
            if name.is_empty() {
                return None;
            }
            if data.skin(name).is_some() {
                return Some(name.to_string());
            }
        }

        if let Some(name) = recommended_skin(example_name, data) {
            return Some(name);
        }

        if data.skin("default").is_some() {
            return Some("default".to_string());
        }

        // Fallback: pick the skin with the most attachments so "content-heavy" examples render by
        // default (e.g. mix-and-match).
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

    fn recommended_animation(data: &SkeletonData) -> Option<String> {
        // Mirrors upstream webgl example defaults for common test exports.
        for name in ["dance", "flying", "animation"] {
            if data.animation(name).is_some() {
                return Some(name.to_string());
            }
        }
        None
    }

    fn recommended_skin(example_name: &str, data: &SkeletonData) -> Option<String> {
        // Mirrors upstream webgl example defaults for common test exports.
        let name = match example_name {
            "goblins" => "goblin",
            "mix-and-match" => "full-skins/girl-blue-cape",
            "chibi-stickers" => "spineboy",
            _ => return None,
        };
        data.skin(name).map(|_| name.to_string())
    }

    async fn load_upstream_manifest() -> Result<WebManifest, JsValue> {
        let text = fetch_text(UPSTREAM_MANIFEST_URL).await?;
        let manifest = serde_json::from_str::<WebManifest>(&text)
            .map_err(|e| JsValue::from_str(&format!("parse web manifest: {e}")))?;
        if manifest.version != 1 {
            log::warn!("unexpected web manifest version: {}", manifest.version);
        }
        Ok(manifest)
    }

    async fn load_demo_bundle() -> Result<ExampleBundle, JsValue> {
        load_example_bundle("demo", "assets/demo.json", "assets/demo.atlas").await
    }

    async fn load_manifest_example_bundle(
        manifest: &WebManifest,
        entry: &WebManifestExample,
    ) -> Result<ExampleBundle, JsValue> {
        let base = manifest.base.as_deref().unwrap_or("assets/spine-runtimes");
        let base = base.trim_end_matches('/');
        let skeleton_url = format!("{base}/{}", entry.skeleton);
        let atlas_url = format!("{base}/{}", entry.atlas);
        load_example_bundle(&entry.name, &skeleton_url, &atlas_url).await
    }

    async fn load_example_bundle(
        example_name: &str,
        skeleton_url: &str,
        atlas_url: &str,
    ) -> Result<ExampleBundle, JsValue> {
        let atlas_text = fetch_text(atlas_url).await?;
        let json_text = fetch_text(skeleton_url).await?;

        let atlas = Atlas::parse(&atlas_text).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        let data = SkeletonData::from_json_str(&json_text)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        let atlas_dir = atlas_url
            .rsplit_once('/')
            .map(|(d, _)| format!("{d}/"))
            .unwrap_or_default();

        let mut page_images = Vec::new();
        for page in &atlas.pages {
            let url = format!("{atlas_dir}{}", page.name);
            match fetch_bytes(&url).await {
                Ok(bytes) => page_images.push((page.name.clone(), bytes)),
                Err(e) => log::warn!("failed to fetch page image {}: {e:?}", url),
            }
        }

        Ok(ExampleBundle {
            example_name: example_name.to_string(),
            atlas,
            data,
            page_images,
        })
    }

    fn build_textures(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &SpineRenderer,
        atlas: &Atlas,
        page_images: Vec<(String, Vec<u8>)>,
    ) -> HashMapTextureProvider {
        let mut by_name = std::collections::HashMap::new();
        for (name, bytes) in page_images {
            by_name.insert(name, bytes);
        }

        let mut textures = HashMapTextureProvider {
            bind_groups: std::collections::HashMap::new(),
        };

        for page in &atlas.pages {
            let Some(image_bytes) = by_name.remove(&page.name) else {
                log::warn!("missing page image bytes for {}", page.name);
                continue;
            };
            let (w, h, pixels) = match decode_png_rgba8(&page.name, &image_bytes) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{e}");
                    continue;
                }
            };

            let sampler = create_sampler_for_atlas_page(device, page);
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("spine2d-web page texture"),
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

        if !textures.bind_groups.is_empty() {
            return textures;
        }

        // Fallback: a procedural RGBA texture (64x64) so the demo still renders even if assets are
        // missing or fetch is blocked.
        let w = 64u32;
        let h = 64u32;
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                pixels[i + 0] = (x * 255 / (w - 1)) as u8;
                pixels[i + 1] = (y * 255 / (h - 1)) as u8;
                pixels[i + 2] = 200;
                pixels[i + 3] = 255;
            }
        }
        let page_name = atlas
            .pages
            .first()
            .map(|p| p.name.as_str())
            .unwrap_or("page.png");
        let sampler = atlas
            .pages
            .first()
            .map(|p| create_sampler_for_atlas_page(device, p))
            .unwrap_or_else(|| device.create_sampler(&wgpu::SamplerDescriptor::default()));

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("spine2d-web fallback texture"),
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
        textures
            .bind_groups
            .insert(page_name.to_string(), bind_group);

        textures
    }

    fn clear_select(select: &web_sys::HtmlSelectElement) {
        while select.length() > 0 {
            select.remove_with_index(0);
        }
    }

    fn populate_select_options(
        document: &web_sys::Document,
        select: &web_sys::HtmlSelectElement,
        options: impl IntoIterator<Item = String>,
        selected: &str,
    ) -> Result<(), JsValue> {
        clear_select(select);
        for name in options {
            let option: web_sys::HtmlOptionElement = document
                .create_element("option")?
                .dyn_into::<web_sys::HtmlOptionElement>()?;
            option.set_value(&name);
            option.set_text(&name);
            select.append_child(&option)?;
        }
        select.set_value(selected);
        Ok(())
    }

    fn populate_skin_select(
        document: &web_sys::Document,
        select: &web_sys::HtmlSelectElement,
        skin_names: Vec<String>,
        selected: Option<&str>,
    ) -> Result<(), JsValue> {
        clear_select(select);

        let none_opt: web_sys::HtmlOptionElement = document
            .create_element("option")?
            .dyn_into::<web_sys::HtmlOptionElement>()?;
        none_opt.set_value("");
        none_opt.set_text("(none)");
        select.append_child(&none_opt)?;

        for name in skin_names {
            let option: web_sys::HtmlOptionElement = document
                .create_element("option")?
                .dyn_into::<web_sys::HtmlOptionElement>()?;
            option.set_value(&name);
            option.set_text(&name);
            select.append_child(&option)?;
        }

        select.set_value(selected.unwrap_or(""));
        Ok(())
    }

    fn init_ui(document: &web_sys::Document, state: Rc<RefCell<WebState>>) -> Result<(), JsValue> {
        let play_button: web_sys::HtmlButtonElement = element_by_id(document, "btn-play")?;
        let restart_button: web_sys::HtmlButtonElement = element_by_id(document, "btn-restart")?;
        let fit_button: web_sys::HtmlButtonElement = element_by_id(document, "btn-fit")?;
        let example_select: web_sys::HtmlSelectElement = element_by_id(document, "example")?;
        let speed_input: web_sys::HtmlInputElement = element_by_id(document, "speed")?;
        let speed_value: web_sys::HtmlSpanElement = element_by_id(document, "speed-value")?;
        let anim_select: web_sys::HtmlSelectElement = element_by_id(document, "anim")?;
        let skin_select: web_sys::HtmlSelectElement = element_by_id(document, "skin")?;

        {
            let st = state.borrow();
            speed_input.set_value(&format!("{:.2}", st.speed));
            speed_value.set_inner_text(&format!("{:.2}", st.speed));
            let example_names = st
                .manifest
                .as_ref()
                .map(|m| {
                    m.examples
                        .iter()
                        .map(|e| e.name.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vec![st.current_example.clone()]);
            let selected = example_names
                .iter()
                .find(|n| n.as_str() == st.current_example)
                .cloned()
                .or_else(|| example_names.first().cloned())
                .unwrap_or_else(|| "demo".to_string());
            populate_select_options(document, &example_select, example_names, &selected)?;
        }

        {
            let st = state.borrow();
            let anim_names = st
                .data
                .animations
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>();
            populate_select_options(document, &anim_select, anim_names, &st.current_animation)?;
        }

        {
            let st = state.borrow();
            let mut skin_names = st.data.skins.keys().cloned().collect::<Vec<_>>();
            skin_names.sort();
            populate_skin_select(
                document,
                &skin_select,
                skin_names,
                st.current_skin.as_deref(),
            )?;
        }

        {
            let state = state.clone();
            let play_button_for_cb = play_button.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let Ok(mut st) = state.try_borrow_mut() else {
                    return;
                };
                st.paused = !st.paused;
                st.last_ts_ms = None;
                play_button_for_cb.set_inner_text(if st.paused { "Play" } else { "Pause" });
            }) as Box<dyn FnMut(_)>);
            play_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let state = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let Ok(mut st) = state.try_borrow_mut() else {
                    return;
                };
                st.restart_current_animation();
            }) as Box<dyn FnMut(_)>);
            restart_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let state = state.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let Ok(mut st) = state.try_borrow_mut() else {
                    return;
                };
                st.initial_bounds = None;
            }) as Box<dyn FnMut(_)>);
            fit_button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let state = state.clone();
            let speed_value = speed_value.clone();
            let speed_input_for_cb = speed_input.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let value = speed_input_for_cb
                    .value()
                    .parse::<f32>()
                    .unwrap_or(1.0)
                    .clamp(0.0, 2.0);
                speed_value.set_inner_text(&format!("{value:.2}"));
                let Ok(mut st) = state.try_borrow_mut() else {
                    return;
                };
                st.speed = value;
            }) as Box<dyn FnMut(_)>);
            speed_input
                .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let state = state.clone();
            let anim_select_for_cb = anim_select.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let name = anim_select_for_cb.value();
                let Ok(mut st) = state.try_borrow_mut() else {
                    return;
                };
                st.set_animation(&name, true);
            }) as Box<dyn FnMut(_)>);
            anim_select
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let state = state.clone();
            let skin_select_for_cb = skin_select.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let value = skin_select_for_cb.value();
                let skin = if value.is_empty() {
                    None
                } else {
                    Some(value.as_str())
                };
                let Ok(mut st) = state.try_borrow_mut() else {
                    return;
                };
                st.set_skin(skin);
            }) as Box<dyn FnMut(_)>);
            skin_select
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let state = state.clone();
            let document = document.clone();
            let example_select_for_cb = example_select.clone();
            let anim_select_for_cb = anim_select.clone();
            let skin_select_for_cb = skin_select.clone();
            let closure = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                let example_name = example_select_for_cb.value();
                let state = state.clone();
                let document = document.clone();
                let anim_select_for_cb = anim_select_for_cb.clone();
                let skin_select_for_cb = skin_select_for_cb.clone();

                spawn_local(async move {
                    let manifest = {
                        let Ok(st) = state.try_borrow() else {
                            return;
                        };
                        st.manifest.clone()
                    };
                    let Some(manifest) = manifest else {
                        return;
                    };
                    let Some(entry) = manifest.examples.iter().find(|e| e.name == example_name)
                    else {
                        return;
                    };
                    let bundle = match load_manifest_example_bundle(&manifest, entry).await {
                        Ok(b) => b,
                        Err(e) => {
                            log::warn!("failed to load example {example_name}: {e:?}");
                            return;
                        }
                    };

                    let Ok(mut st) = state.try_borrow_mut() else {
                        return;
                    };
                    st.apply_example_bundle(bundle, None);
                    let anim_names = st
                        .data
                        .animations
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<_>>();
                    let mut skin_names = st.data.skins.keys().cloned().collect::<Vec<_>>();
                    skin_names.sort();
                    if let Err(e) = populate_select_options(
                        &document,
                        &anim_select_for_cb,
                        anim_names,
                        &st.current_animation,
                    ) {
                        log::warn!("update animation select failed: {e:?}");
                    }
                    if let Err(e) = populate_skin_select(
                        &document,
                        &skin_select_for_cb,
                        skin_names,
                        st.current_skin.as_deref(),
                    ) {
                        log::warn!("update skin select failed: {e:?}");
                    }
                });
            }) as Box<dyn FnMut(_)>);
            example_select
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }

    struct WebState {
        canvas: web_sys::HtmlCanvasElement,
        surface: wgpu::Surface<'static>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,

        renderer: SpineRenderer,
        atlas: Atlas,
        data: std::sync::Arc<SkeletonData>,
        skeleton: Skeleton,
        state: AnimationState,
        textures: HashMapTextureProvider,
        draw_list: DrawList,
        initial_bounds: Option<Bounds2>,
        fit_margin: f32,

        last_ts_ms: Option<f64>,
        paused: bool,
        speed: f32,
        current_animation: String,
        manifest: Option<WebManifest>,
        current_example: String,
        current_skin: Option<String>,
    }

    impl WebState {
        fn apply_example_bundle(&mut self, bundle: ExampleBundle, requested_anim: Option<&str>) {
            self.current_example = bundle.example_name;
            self.atlas = bundle.atlas;
            self.data = bundle.data;

            self.skeleton = Skeleton::new(self.data.clone());
            self.current_skin = pick_default_skin(&self.current_example, &self.data, None);
            if let Some(skin) = self.current_skin.as_deref() {
                self.skeleton.set_skin(Some(skin));
            }
            self.skeleton.setup_pose();
            self.skeleton
                .update_world_transform_with_physics(spine2d::Physics::None);

            let state_data = AnimationStateData::new(self.data.clone());
            self.state = AnimationState::new(state_data);

            self.current_animation = pick_default_animation(&self.data, requested_anim);
            self.state
                .set_animation(0, self.current_animation.as_str(), true);

            self.textures = build_textures(
                &self.device,
                &self.queue,
                &self.renderer,
                &self.atlas,
                bundle.page_images,
            );

            self.last_ts_ms = None;
            self.initial_bounds = None;
        }

        fn set_skin(&mut self, skin: Option<&str>) {
            let skin = skin.filter(|s| !s.is_empty());
            if self.current_skin.as_deref() == skin {
                return;
            }
            self.current_skin = skin.map(|s| s.to_string());
            self.last_ts_ms = None;
            self.initial_bounds = None;

            self.skeleton.set_skin(skin);
        }

        fn set_animation(&mut self, name: &str, looping: bool) {
            if self.current_animation == name {
                self.restart_current_animation();
                return;
            }

            self.current_animation = name.to_string();
            self.last_ts_ms = None;
            self.initial_bounds = None;

            self.state.set_animation(0, name, looping);
        }

        fn restart_current_animation(&mut self) {
            let name = self.current_animation.clone();
            self.last_ts_ms = None;
            self.initial_bounds = None;
            self.state.set_animation(0, name.as_str(), true);
        }

        fn render(&mut self, ts_ms: f64) -> Result<(), wgpu::SurfaceError> {
            self.resize_if_needed();

            let mut dt = if let Some(prev) = self.last_ts_ms {
                ((ts_ms - prev) * 0.001).max(0.0) as f32
            } else {
                0.0
            };
            self.last_ts_ms = Some(ts_ms);
            if self.paused {
                dt = 0.0;
            } else {
                dt *= self.speed.max(0.0);
            }

            self.state.update(dt);
            self.skeleton.setup_pose();
            self.state.apply(&mut self.skeleton);
            self.skeleton.update(dt);
            self.skeleton
                .update_world_transform_with_physics(spine2d::Physics::None);

            self.draw_list.clear();
            spine2d::append_draw_list_with_atlas(&mut self.draw_list, &self.skeleton, &self.atlas);
            self.renderer
                .upload(&self.device, &self.queue, &self.draw_list);
            self.update_fit_bounds();

            let frame = match self.surface.get_current_texture() {
                Ok(f) => f,
                Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                    self.surface.configure(&self.device, &self.config);
                    return Ok(());
                }
                Err(e) => return Err(e),
            };
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("spine2d-web encoder"),
                });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("spine2d-web pass"),
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
                self.renderer
                    .render(&mut pass, &self.draw_list, &self.textures);
            }

            self.queue.submit([encoder.finish()]);
            frame.present();
            Ok(())
        }

        fn resize_if_needed(&mut self) {
            let (width, height) = physical_canvas_size(&self.canvas);
            if width == self.config.width && height == self.config.height {
                return;
            }

            self.canvas.set_width(width);
            self.canvas.set_height(height);

            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            // Keep previous camera fit if we have one; otherwise default to a centered ortho.
            if let Some(bounds) = self.initial_bounds {
                self.renderer.update_globals_matrix(
                    &self.queue,
                    clip_from_world_fit_bounds(bounds, width, height, self.fit_margin),
                );
            } else {
                self.renderer.update_globals_ortho_centered(
                    &self.queue,
                    width as f32,
                    height as f32,
                );
            }
        }

        fn update_fit_bounds(&mut self) {
            if self.initial_bounds.is_some() {
                return;
            }
            let Some(bounds) = bounds_from_draw_list(&self.draw_list) else {
                return;
            };
            self.initial_bounds = Some(bounds);
            self.renderer.update_globals_matrix(
                &self.queue,
                clip_from_world_fit_bounds(
                    bounds,
                    self.config.width,
                    self.config.height,
                    self.fit_margin,
                ),
            );
        }
    }

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
  "animations": {
    "spin": {
      "bones": {
        "root": {
          "rotate": [
            { "time": 0, "angle": 0 },
            { "time": 1, "angle": 360 }
          ]
        }
      }
    }
  }
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

    async fn fetch_text(path: &str) -> Result<String, JsValue> {
        let bytes = fetch_bytes(path).await?;
        let text = String::from_utf8(bytes).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        Ok(text)
    }

    async fn fetch_bytes(path: &str) -> Result<Vec<u8>, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("missing window"))?;
        let resp = JsFuture::from(window.fetch_with_str(path)).await?;
        let resp: web_sys::Response = resp.dyn_into()?;

        let ab = JsFuture::from(resp.array_buffer()?).await?;
        let u8 = js_sys::Uint8Array::new(&ab);
        Ok(u8.to_vec())
    }

    fn decode_png_rgba8(label: &str, bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
        // For now we use a minimal pure-Rust decode path (works on wasm32).
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .map_err(|e| format!("decode png {label}: {e}"))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Ok((w, h, rgba.into_raw()))
    }

    fn physical_canvas_size(canvas: &web_sys::HtmlCanvasElement) -> (u32, u32) {
        let cw = canvas.client_width().max(1) as f64;
        let ch = canvas.client_height().max(1) as f64;
        let dpr = web_sys::window()
            .map(|w| w.device_pixel_ratio())
            .unwrap_or(1.0)
            .max(0.1);

        let w = (cw * dpr).round().max(1.0) as u32;
        let h = (ch * dpr).round().max(1.0) as u32;
        (w, h)
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

        let scale_px = (vw * margin / world_w).min(vh * margin / world_h);
        let sx = 2.0 * scale_px / vw;
        let sy = 2.0 * scale_px / vh;

        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-cx * sx, -cy * sy, 0.0, 1.0],
        ]
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod web {
    // This crate is intended to be built via Trunk for `wasm32-unknown-unknown`.
    // Keep a tiny native stub so `cargo test` for the workspace stays green.
}
