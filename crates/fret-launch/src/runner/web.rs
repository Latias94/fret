//! Web launcher implementation (winit + wgpu via WebGPU).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

mod dispatcher;
use dispatcher::WebDispatcher;

use fret_app::{App, Effect};
use fret_core::{AppWindowId, Event, Point, Px, Rect, Scene, Size};
use fret_render::{RenderSceneParams, Renderer, SurfaceState, UploadedRgba8Image, WgpuContext};
use fret_runtime::{
    FrameId, PlatformCapabilities, TickId, WindowRequest, apply_window_metrics_event,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::wasm_bindgen::JsCast as _;
use winit::application::ApplicationHandler;
use winit::cursor::Cursor;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId};

use winit::platform::web::{WindowAttributesWeb, WindowExtWeb};

use fret_platform_web::WebPlatformServices;

use super::streaming_upload::StreamingUploadQueue;
use super::{
    RenderTargetUpdate, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitGlobalContext,
    WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use crate::RunnerError;

fn ensure_canvas_ime_mount(canvas: &web_sys::HtmlCanvasElement) -> Option<web_sys::HtmlElement> {
    let canvas_el: web_sys::HtmlElement = canvas.clone().unchecked_into();

    if let Some(parent) = canvas_el.parent_element() {
        // Preferred: a dedicated overlay layer inside the wrapper.
        if parent.get_attribute("data-fret-ime-wrapper").as_deref() == Some("1") {
            if let Ok(Some(overlay)) = parent.query_selector("[data-fret-ime-overlay='1']") {
                if let Ok(overlay) = overlay.dyn_into::<web_sys::HtmlElement>() {
                    return Some(overlay);
                }
            }

            let document = canvas_el.owner_document()?;
            let el = document.create_element("div").ok()?;
            let overlay: web_sys::HtmlElement = el.dyn_into().ok()?;
            let _ = overlay.set_attribute("data-fret-ime-overlay", "1");
            let _ = overlay.set_attribute("data-fret-ime-mount", "1");

            let style = overlay.style();
            let _ = style.set_property("position", "absolute");
            let _ = style.set_property("left", "0");
            let _ = style.set_property("top", "0");
            let _ = style.set_property("width", "100%");
            let _ = style.set_property("height", "100%");
            let _ = style.set_property("pointer-events", "none");
            let _ = style.set_property("overflow", "hidden");

            let _ = parent.append_child(&overlay);
            return Some(overlay);
        }

        // Back-compat: older mount strategy uses the direct parent as the mount.
        if parent.get_attribute("data-fret-ime-mount").as_deref() == Some("1") {
            if let Ok(parent) = parent.dyn_into::<web_sys::HtmlElement>() {
                return Some(parent);
            }
        }
    }

    let document = canvas_el.owner_document()?;
    let el = document.create_element("div").ok()?;
    let wrapper: web_sys::HtmlElement = el.dyn_into().ok()?;
    let _ = wrapper.set_attribute("data-fret-ime-wrapper", "1");

    let style = wrapper.style();
    let _ = style.set_property("position", "relative");
    let _ = style.set_property("margin", "0");
    let _ = style.set_property("padding", "0");
    let _ = style.set_property("border", "0");
    let _ = style.set_property("overflow", "hidden");
    let _ = style.set_property("display", "block");

    let parent = canvas_el.parent_node()?;
    let wrapper_node: web_sys::Node = wrapper.clone().unchecked_into();
    let canvas_node: web_sys::Node = canvas_el.clone().unchecked_into();

    let _ = parent.replace_child(&wrapper_node, &canvas_node);
    let _ = wrapper.append_child(&canvas_node);

    let el = document.create_element("div").ok()?;
    let overlay: web_sys::HtmlElement = el.dyn_into().ok()?;
    let _ = overlay.set_attribute("data-fret-ime-overlay", "1");
    let _ = overlay.set_attribute("data-fret-ime-mount", "1");
    let style = overlay.style();
    let _ = style.set_property("position", "absolute");
    let _ = style.set_property("left", "0");
    let _ = style.set_property("top", "0");
    let _ = style.set_property("width", "100%");
    let _ = style.set_property("height", "100%");
    let _ = style.set_property("pointer-events", "none");
    let _ = style.set_property("overflow", "hidden");
    let _ = wrapper.append_child(&overlay);

    Some(overlay)
}

struct GfxState {
    ctx: WgpuContext,
    surface_state: SurfaceState<'static>,
    renderer: Renderer,
    last_surface_error: Option<wgpu::SurfaceError>,
}

pub struct WinitRunner<D: WinitAppDriver> {
    pub config: WinitRunnerConfig,
    pub app: App,
    pub driver: D,

    event_loop_proxy: Option<EventLoopProxy>,
    dispatcher: WebDispatcher,

    window: Option<Arc<dyn Window>>,
    window_id: Option<WindowId>,
    app_window: AppWindowId,
    window_state: Option<D::WindowState>,

    pending_gfx: Rc<RefCell<Option<GfxState>>>,
    gfx: Option<GfxState>,
    scene: Scene,

    pending_events: Vec<Event>,
    tick_id: TickId,
    frame_id: FrameId,

    uploaded_images: HashMap<fret_core::ImageId, UploadedImageEntry>,
    streaming_uploads: StreamingUploadQueue,
    nv12_gpu: Option<super::yuv_gpu::Nv12GpuConverter>,
    renderer_caps: Option<fret_render::RendererCapabilities>,

    platform: fret_runner_winit::WinitPlatform,
    web_cursor: Option<fret_runner_winit::WebCursorListener>,
    web_services: WebPlatformServices,
    gpu_ready_called: bool,
    exiting: bool,
    exit_requested: Rc<Cell<bool>>,
}

struct UploadedImageEntry {
    uploaded: UploadedRgba8Image,
    stream_generation: u64,
    alpha_mode: fret_core::AlphaMode,
    nv12_planes: Option<super::yuv_gpu::Nv12Planes>,
}

#[derive(Clone)]
pub struct WebRunnerHandle {
    proxy: EventLoopProxy,
    exit_requested: Rc<Cell<bool>>,
}

impl WebRunnerHandle {
    pub fn destroy(&self) {
        self.exit_requested.set(true);
        self.proxy.wake_up();
    }
}

pub fn run_app<D: WinitAppDriver + 'static>(
    config: WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<(), RunnerError> {
    run_app_with_event_loop(EventLoop::new()?, config, app, driver)
}

pub fn run_app_with_handle<D: WinitAppDriver + 'static>(
    config: WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<WebRunnerHandle, RunnerError> {
    run_app_with_event_loop_and_handle(EventLoop::new()?, config, app, driver)
}

pub fn run_app_with_event_loop<D: WinitAppDriver + 'static>(
    event_loop: EventLoop,
    config: WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<(), RunnerError> {
    let mut runner = WinitRunner::new_app(config, app, driver);
    runner.set_event_loop_proxy(event_loop.create_proxy());
    event_loop.run_app(runner)?;
    Ok(())
}

pub fn run_app_with_event_loop_and_handle<D: WinitAppDriver + 'static>(
    event_loop: EventLoop,
    config: WinitRunnerConfig,
    app: App,
    driver: D,
) -> Result<WebRunnerHandle, RunnerError> {
    let mut runner = WinitRunner::new_app(config, app, driver);
    let proxy = event_loop.create_proxy();
    runner.set_event_loop_proxy(proxy.clone());
    let handle = runner.handle();
    event_loop.run_app(runner)?;
    Ok(handle)
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub fn new_app(config: WinitRunnerConfig, app: App, driver: D) -> Self {
        let mut app = app;
        let requested = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_else(|| {
                let caps = PlatformCapabilities::default();
                app.set_global(caps.clone());
                caps
            });
        let caps = Self::effective_platform_capabilities(&requested);
        if caps != requested {
            app.set_global(caps.clone());
        }

        let dispatcher = WebDispatcher::new(caps.exec);
        app.set_global::<fret_runtime::DispatcherHandle>(dispatcher.handle());

        Self {
            config,
            app,
            driver,
            event_loop_proxy: None,
            dispatcher,
            window: None,
            window_id: None,
            app_window: AppWindowId::default(),
            window_state: None,
            pending_gfx: Rc::new(RefCell::new(None)),
            gfx: None,
            scene: Scene::default(),
            pending_events: Vec::new(),
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            uploaded_images: HashMap::new(),
            streaming_uploads: StreamingUploadQueue::default(),
            nv12_gpu: None,
            renderer_caps: None,
            platform: fret_runner_winit::WinitPlatform::default(),
            web_cursor: None,
            web_services: WebPlatformServices::default(),
            gpu_ready_called: false,
            exiting: false,
            exit_requested: Rc::new(Cell::new(false)),
        }
    }

    pub fn set_event_loop_proxy(&mut self, proxy: EventLoopProxy) {
        let wake = proxy.clone();
        self.web_services.set_waker(move || wake.wake_up());
        self.dispatcher.set_event_loop_proxy(proxy.clone());
        self.event_loop_proxy = Some(proxy);
    }

    pub fn handle(&self) -> WebRunnerHandle {
        let proxy = self
            .event_loop_proxy
            .clone()
            .expect("event loop proxy must be set before creating a WebRunnerHandle");
        WebRunnerHandle {
            proxy,
            exit_requested: self.exit_requested.clone(),
        }
    }

    fn maybe_exit(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        if self.exiting {
            return true;
        }

        if !self.exit_requested.get() {
            return false;
        }

        self.exiting = true;
        self.dispatcher.shutdown();
        self.web_cursor.take();
        event_loop.exit();
        true
    }

    fn effective_platform_capabilities(requested: &PlatformCapabilities) -> PlatformCapabilities {
        let mut available = PlatformCapabilities::default();
        available.exec.background_work = fret_runtime::ExecBackgroundWork::Cooperative;
        available.exec.wake = fret_runtime::ExecWake::BestEffort;
        available.exec.timers = fret_runtime::ExecTimers::BestEffort;

        available.ui.multi_window = false;
        available.ui.window_tear_off = false;
        available.ui.cursor_icons = true;
        available.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
        available.ui.window_set_outer_position = fret_runtime::WindowSetOuterPositionQuality::None;
        available.ui.window_z_level = fret_runtime::WindowZLevelQuality::None;
        available.clipboard.text = true;
        available.clipboard.files = false;
        available.dnd.external = false;
        available.dnd.external_payload = fret_runtime::ExternalDragPayloadKind::None;
        available.dnd.external_position = fret_runtime::ExternalDragPositionQuality::None;
        available.ime.enabled = true;
        available.ime.set_cursor_area = true;
        available.fs.real_paths = false;
        available.fs.file_dialogs = true;
        available.shell.open_url = true;
        available.gfx.native_gpu = false;
        available.gfx.webgpu = true;

        let mut caps = requested.clone();
        caps.exec.background_work = caps
            .exec
            .background_work
            .clamp_to_available(available.exec.background_work);
        caps.exec.wake = caps.exec.wake.clamp_to_available(available.exec.wake);
        caps.exec.timers = caps.exec.timers.clamp_to_available(available.exec.timers);

        caps.ui.multi_window &= available.ui.multi_window;
        caps.ui.window_tear_off &= available.ui.window_tear_off;
        caps.ui.cursor_icons &= available.ui.cursor_icons;
        caps.ui.window_hover_detection = caps
            .ui
            .window_hover_detection
            .clamp_to_available(available.ui.window_hover_detection);
        caps.ui.window_set_outer_position = caps
            .ui
            .window_set_outer_position
            .clamp_to_available(available.ui.window_set_outer_position);
        caps.ui.window_z_level = caps
            .ui
            .window_z_level
            .clamp_to_available(available.ui.window_z_level);
        caps.clipboard.text &= available.clipboard.text;
        caps.clipboard.files &= available.clipboard.files;
        caps.dnd.external &= available.dnd.external;
        caps.dnd.external_payload = if caps.dnd.external {
            available.dnd.external_payload
        } else {
            fret_runtime::ExternalDragPayloadKind::None
        };
        caps.dnd.external_position = if caps.dnd.external {
            caps.dnd
                .external_position
                .clamp_to_available(available.dnd.external_position)
        } else {
            fret_runtime::ExternalDragPositionQuality::None
        };
        caps.ime.enabled &= available.ime.enabled;
        caps.ime.set_cursor_area &= available.ime.set_cursor_area;
        caps.fs.real_paths &= available.fs.real_paths;
        caps.fs.file_dialogs &= available.fs.file_dialogs;
        caps.shell.open_url &= available.shell.open_url;
        caps.gfx.native_gpu &= available.gfx.native_gpu;
        caps.gfx.webgpu &= available.gfx.webgpu;
        caps
    }

    fn adopt_gfx_if_ready(&mut self) {
        if self.gfx.is_some() {
            return;
        }
        let pending = self.pending_gfx.borrow_mut().take();
        let Some(mut gfx) = pending else {
            return;
        };

        let renderer_caps = fret_render::RendererCapabilities::from_wgpu_context(&gfx.ctx);
        self.app
            .set_global::<fret_render::RendererCapabilities>(renderer_caps.clone());
        self.renderer_caps = Some(renderer_caps);

        self.app
            .set_global::<fret_core::TextFontFamilyConfig>(self.config.text_font_families.clone());
        let _ = gfx
            .renderer
            .set_text_font_families(&self.config.text_font_families);

        // Web/WASM cannot access system fonts. Load our bundled defaults as soon as the renderer
        // becomes available, then seed `TextFontFamilyConfig` deterministically.
        let default_fonts = fret_fonts::default_fonts()
            .iter()
            .map(|bytes| bytes.to_vec())
            .collect::<Vec<_>>();
        let _ = gfx.renderer.add_fonts(default_fonts);

        let update = fret_runtime::apply_font_catalog_update(
            &mut self.app,
            gfx.renderer.all_font_names(),
            fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
        );
        let _ = gfx.renderer.set_text_font_families(&update.config);
        self.app
            .set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
                gfx.renderer.text_font_stack_key(),
            ));

        self.gfx = Some(gfx);
    }

    fn ensure_gpu_ready_hook(&mut self) {
        if self.gpu_ready_called {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        self.driver
            .gpu_ready(&mut self.app, &gfx.ctx, &mut gfx.renderer);
        self.gpu_ready_called = true;
    }

    fn desired_surface_size(window: &dyn Window) -> Option<PhysicalSize<u32>> {
        let canvas: web_sys::HtmlCanvasElement = window.canvas()?.clone();
        let web_window = web_sys::window()?;
        let dpr = web_window.device_pixel_ratio().max(1.0);
        let css_w = canvas.client_width().max(0) as f64;
        let css_h = canvas.client_height().max(0) as f64;
        let physical = PhysicalSize::new(
            (css_w * dpr).round().max(1.0) as u32,
            (css_h * dpr).round().max(1.0) as u32,
        );

        if canvas.width() != physical.width {
            canvas.set_width(physical.width);
        }
        if canvas.height() != physical.height {
            canvas.set_height(physical.height);
        }

        Some(physical)
    }

    fn apply_streaming_image_update_rgba8(
        &mut self,
        window: &dyn Window,
        gfx: &mut GfxState,
        stats: &mut super::streaming_upload::StreamingUploadStats,
        target_window: Option<AppWindowId>,
        token: fret_core::ImageUpdateToken,
        image: fret_core::ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<fret_core::RectPx>,
        bytes_per_row: u32,
        bytes: &[u8],
        color_info: fret_core::ImageColorInfo,
        alpha_mode: fret_core::AlphaMode,
    ) {
        if let Some(target_window) = target_window
            && target_window != self.app_window
        {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::Unsupported,
                });
            }
            return;
        }

        if width == 0 || height == 0 {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return;
        }

        let Some(entry) = self.uploaded_images.get_mut(&image) else {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::UnknownImage,
                });
            }
            return;
        };

        if stream_generation < entry.stream_generation {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::Coalesced,
                });
            }
            return;
        }
        entry.stream_generation = stream_generation;

        let rect = update_rect_px.unwrap_or_else(|| fret_core::RectPx::full(width, height));
        if rect.is_empty() {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return;
        }

        if rect.x > width
            || rect.y > height
            || rect.x.saturating_add(rect.w) > width
            || rect.y.saturating_add(rect.h) > height
        {
            tracing::warn!(
                image = ?image,
                width,
                height,
                rect = ?rect,
                "ignoring ImageUpdateRgba8 with out-of-bounds update rect"
            );
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return;
        }

        let color_space = match color_info.encoding {
            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
            fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
        };

        let row_bytes = rect.w.saturating_mul(4);
        if bytes_per_row < row_bytes {
            tracing::warn!(
                image = ?image,
                bytes_per_row,
                row_bytes,
                "ignoring ImageUpdateRgba8 with undersized bytes_per_row"
            );
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return;
        }

        let expected_len = (bytes_per_row as usize).saturating_mul(rect.h as usize);
        if bytes.len() != expected_len {
            tracing::warn!(
                image = ?image,
                got = bytes.len(),
                expected = expected_len,
                "ignoring ImageUpdateRgba8 with invalid byte length"
            );
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return;
        }

        if entry.alpha_mode != alpha_mode {
            if !gfx.renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view: entry.uploaded.view.clone(),
                    size: entry.uploaded.size,
                    format: entry.uploaded.format,
                    color_space: entry.uploaded.color_space,
                    alpha_mode,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    self.pending_events.push(Event::ImageUpdateDropped {
                        token,
                        image,
                        reason: fret_core::ImageUpdateDropReason::UnknownImage,
                    });
                }
                return;
            }
            entry.alpha_mode = alpha_mode;
        }

        let needs_replace =
            entry.uploaded.size != (width, height) || entry.uploaded.color_space != color_space;
        let mut applied_upload_bytes: Option<u64> = None;
        if needs_replace {
            let is_full_update = rect.x == 0 && rect.y == 0 && rect.w == width && rect.h == height;
            if !is_full_update {
                tracing::warn!(
                    image = ?image,
                    old_size = ?entry.uploaded.size,
                    new_size = ?(width, height),
                    "ignoring partial ImageUpdateRgba8 while image storage needs replace"
                );
                if self.config.streaming_update_ack_enabled {
                    self.pending_events.push(Event::ImageUpdateDropped {
                        token,
                        image,
                        reason: fret_core::ImageUpdateDropReason::Unsupported,
                    });
                }
                return;
            }

            let uploaded = if bytes_per_row == width.saturating_mul(4)
                && bytes.len()
                    == (width as usize)
                        .saturating_mul(height as usize)
                        .saturating_mul(4)
            {
                applied_upload_bytes = Some(
                    super::streaming_upload::estimate_rgba8_upload_bytes_for_rect(
                        fret_core::RectPx::full(width, height),
                        width.saturating_mul(4),
                    ),
                );
                fret_render::upload_rgba8_image(
                    &gfx.ctx.device,
                    &gfx.ctx.queue,
                    (width, height),
                    bytes,
                    color_space,
                )
            } else {
                applied_upload_bytes = Some(
                    super::streaming_upload::estimate_rgba8_upload_bytes_for_rect(
                        fret_core::RectPx::full(width, height),
                        bytes_per_row,
                    ),
                );
                let uploaded = fret_render::create_rgba8_image_storage(
                    &gfx.ctx.device,
                    (width, height),
                    color_space,
                );
                uploaded.write_region(
                    &gfx.ctx.queue,
                    (0, 0),
                    (width, height),
                    bytes_per_row,
                    bytes,
                );
                uploaded
            };

            let view = uploaded
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            if !gfx.renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view,
                    size: uploaded.size,
                    format: uploaded.format,
                    color_space: uploaded.color_space,
                    alpha_mode,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    self.pending_events.push(Event::ImageUpdateDropped {
                        token,
                        image,
                        reason: fret_core::ImageUpdateDropReason::UnknownImage,
                    });
                }
                return;
            }
            entry.uploaded = uploaded;
            entry.alpha_mode = alpha_mode;
            entry.nv12_planes = None;
        } else {
            entry.uploaded.write_region(
                &gfx.ctx.queue,
                (rect.x, rect.y),
                (rect.w, rect.h),
                bytes_per_row,
                bytes,
            );
            applied_upload_bytes = Some(
                super::streaming_upload::estimate_rgba8_upload_bytes_for_rect(rect, bytes_per_row),
            );
        }

        let applied_upload_bytes = applied_upload_bytes.unwrap_or(0);
        stats.upload_bytes_applied = stats
            .upload_bytes_applied
            .saturating_add(applied_upload_bytes);

        if self.config.streaming_update_ack_enabled {
            self.pending_events
                .push(Event::ImageUpdateApplied { token, image });
        }

        window.request_redraw();
    }

    fn try_apply_streaming_image_update_nv12_gpu(
        &mut self,
        window: &dyn Window,
        gfx: &mut GfxState,
        stats: &mut super::streaming_upload::StreamingUploadStats,
        target_window: Option<AppWindowId>,
        token: fret_core::ImageUpdateToken,
        image: fret_core::ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<fret_core::RectPx>,
        y_bytes_per_row: u32,
        y_plane: &[u8],
        uv_bytes_per_row: u32,
        uv_plane: &[u8],
        color_info: fret_core::ImageColorInfo,
    ) -> bool {
        let requested = self.config.streaming_nv12_gpu_convert_enabled
            || std::env::var_os("FRET_STREAMING_GPU_YUV").is_some_and(|v| !v.is_empty());
        if !requested {
            return false;
        }

        let supported = self
            .renderer_caps
            .as_ref()
            .is_some_and(|c| c.streaming_images.nv12_gpu_convert);
        if !supported {
            return false;
        }

        if let Some(target_window) = target_window
            && target_window != self.app_window
        {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::Unsupported,
                });
            }
            return true;
        }

        if width == 0 || height == 0 {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return true;
        }

        let Some(entry) = self.uploaded_images.get_mut(&image) else {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::UnknownImage,
                });
            }
            return true;
        };

        if stream_generation < entry.stream_generation {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::Coalesced,
                });
            }
            return true;
        }
        entry.stream_generation = stream_generation;

        let Ok(rect) = super::yuv::normalize_update_rect_420(width, height, update_rect_px) else {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return true;
        };

        let color_space = match color_info.encoding {
            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
            fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
        };

        if entry.uploaded.format != wgpu::TextureFormat::Rgba8UnormSrgb {
            return false;
        }
        if color_space != fret_render::ImageColorSpace::Srgb {
            return false;
        }

        let is_full_update = rect.x == 0 && rect.y == 0 && rect.w == width && rect.h == height;
        let needs_replace =
            entry.uploaded.size != (width, height) || entry.uploaded.color_space != color_space;
        if needs_replace {
            if !is_full_update {
                if self.config.streaming_update_ack_enabled {
                    self.pending_events.push(Event::ImageUpdateDropped {
                        token,
                        image,
                        reason: fret_core::ImageUpdateDropReason::Unsupported,
                    });
                }
                return true;
            }

            let uploaded = fret_render::create_rgba8_image_storage(
                &gfx.ctx.device,
                (width, height),
                color_space,
            );
            let view = uploaded
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            if !gfx.renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view,
                    size: uploaded.size,
                    format: uploaded.format,
                    color_space: uploaded.color_space,
                    alpha_mode: fret_core::AlphaMode::Opaque,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    self.pending_events.push(Event::ImageUpdateDropped {
                        token,
                        image,
                        reason: fret_core::ImageUpdateDropReason::UnknownImage,
                    });
                }
                return true;
            }
            entry.uploaded = uploaded;
            entry.alpha_mode = fret_core::AlphaMode::Opaque;
            entry.nv12_planes = None;
        }

        if entry.alpha_mode != fret_core::AlphaMode::Opaque {
            if !gfx.renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view: entry.uploaded.view.clone(),
                    size: entry.uploaded.size,
                    format: entry.uploaded.format,
                    color_space: entry.uploaded.color_space,
                    alpha_mode: fret_core::AlphaMode::Opaque,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    self.pending_events.push(Event::ImageUpdateDropped {
                        token,
                        image,
                        reason: fret_core::ImageUpdateDropReason::UnknownImage,
                    });
                }
                return true;
            }
            entry.alpha_mode = fret_core::AlphaMode::Opaque;
        }

        if entry
            .nv12_planes
            .as_ref()
            .is_none_or(|p| p.size != (width, height))
        {
            entry.nv12_planes = Some(super::yuv_gpu::Nv12Planes::new(
                &gfx.ctx.device,
                (width, height),
            ));
        }
        if self.nv12_gpu.is_none() {
            self.nv12_gpu = Some(super::yuv_gpu::Nv12GpuConverter::new(&gfx.ctx.device));
        }

        let Some(planes) = entry.nv12_planes.as_ref() else {
            return false;
        };
        let Some(converter) = self.nv12_gpu.as_ref() else {
            return false;
        };

        let t0 = fret_core::time::Instant::now();
        let Ok(uploaded_bytes) = super::yuv_gpu::write_nv12_rect(
            &gfx.ctx.queue,
            planes,
            rect,
            y_bytes_per_row,
            y_plane,
            uv_bytes_per_row,
            uv_plane,
        ) else {
            if self.config.streaming_update_ack_enabled {
                self.pending_events.push(Event::ImageUpdateDropped {
                    token,
                    image,
                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                });
            }
            return true;
        };

        stats.upload_bytes_applied = stats.upload_bytes_applied.saturating_add(uploaded_bytes);

        converter.convert_rect_into(super::yuv_gpu::Nv12ConvertRectIntoArgs {
            device: &gfx.ctx.device,
            queue: &gfx.ctx.queue,
            dst_view: &entry.uploaded.view,
            rect,
            y_view: &planes.y_view,
            uv_view: &planes.uv_view,
            range: color_info.range,
            matrix: color_info.matrix,
        });

        stats.yuv_conversions_applied = stats.yuv_conversions_applied.saturating_add(1);
        stats.yuv_convert_us = stats
            .yuv_convert_us
            .saturating_add(t0.elapsed().as_micros() as u64);
        stats.yuv_convert_output_bytes = stats
            .yuv_convert_output_bytes
            .saturating_add(rect.w.saturating_mul(rect.h).saturating_mul(4) as u64);

        if self.config.streaming_update_ack_enabled {
            self.pending_events
                .push(Event::ImageUpdateApplied { token, image });
        }

        window.request_redraw();
        true
    }

    fn drain_effects(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window: &dyn Window,
        gfx: &mut GfxState,
        state: &mut D::WindowState,
    ) -> bool {
        let did_work = self.dispatcher.drain_turn() || self.drain_inboxes(Some(self.app_window));
        let effects = self.app.flush_effects();
        let effects = self.web_services.handle_effects(&mut self.app, effects);
        self.pending_events.extend(self.web_services.take_events());

        let (effects, mut stats, acks) = self.streaming_uploads.process_effects(
            self.frame_id,
            effects,
            self.config.streaming_upload_budget_bytes_per_frame,
            self.config.streaming_staging_budget_bytes,
            self.config.streaming_update_ack_enabled,
        );
        tracing::trace!(
            did_work,
            effects = effects.len(),
            acks = acks.len(),
            "driver: drain_effects turn"
        );
        if self.config.streaming_update_ack_enabled {
            for ack in acks {
                match ack.kind {
                    super::streaming_upload::StreamingUploadAckKind::Dropped(reason) => {
                        self.pending_events.push(Event::ImageUpdateDropped {
                            token: ack.token,
                            image: ack.image,
                            reason,
                        });
                    }
                }
            }
        }
        let had_effects = !effects.is_empty();
        if !had_effects {
            if self.streaming_uploads.has_pending() {
                window.request_redraw();
            }

            let streaming_snapshot_enabled = self.config.streaming_perf_snapshot_enabled
                || std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty());
            let streaming_stats_have_activity = stats.update_effects_seen > 0
                || stats.update_effects_enqueued > 0
                || stats.update_effects_replaced > 0
                || stats.update_effects_applied > 0
                || stats.update_effects_delayed_budget > 0
                || stats.update_effects_dropped_staging > 0
                || stats.upload_bytes_budgeted > 0
                || stats.upload_bytes_applied > 0
                || stats.pending_updates > 0
                || stats.pending_staging_bytes > 0
                || stats.yuv_conversions_attempted > 0
                || stats.yuv_convert_us > 0;
            if streaming_snapshot_enabled && streaming_stats_have_activity {
                self.app.set_global(fret_core::StreamingUploadPerfSnapshot {
                    frame_id: self.frame_id,
                    upload_budget_bytes_per_frame: stats.upload_budget_bytes_per_frame,
                    staging_budget_bytes: stats.staging_budget_bytes,
                    update_effects_seen: u64::from(stats.update_effects_seen),
                    update_effects_enqueued: u64::from(stats.update_effects_enqueued),
                    update_effects_replaced: u64::from(stats.update_effects_replaced),
                    update_effects_applied: u64::from(stats.update_effects_applied),
                    update_effects_delayed_budget: u64::from(stats.update_effects_delayed_budget),
                    update_effects_dropped_staging: u64::from(stats.update_effects_dropped_staging),
                    upload_bytes_budgeted: stats.upload_bytes_budgeted,
                    upload_bytes_applied: stats.upload_bytes_applied,
                    pending_updates: u64::from(stats.pending_updates),
                    pending_staging_bytes: stats.pending_staging_bytes,
                    yuv_convert_us: stats.yuv_convert_us,
                    yuv_convert_output_bytes: stats.yuv_convert_output_bytes,
                    yuv_conversions_attempted: u64::from(stats.yuv_conversions_attempted),
                    yuv_conversions_applied: u64::from(stats.yuv_conversions_applied),
                });
            }

            if std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty())
                && (stats.update_effects_delayed_budget > 0
                    || stats.update_effects_dropped_staging > 0
                    || stats.update_effects_replaced > 0
                    || stats.yuv_conversions_attempted > 0)
            {
                tracing::debug!(
                    seen = stats.update_effects_seen,
                    enqueued = stats.update_effects_enqueued,
                    replaced = stats.update_effects_replaced,
                    applied = stats.update_effects_applied,
                    delayed_budget = stats.update_effects_delayed_budget,
                    dropped_staging = stats.update_effects_dropped_staging,
                    upload_bytes_budgeted = stats.upload_bytes_budgeted,
                    upload_bytes_applied = stats.upload_bytes_applied,
                    upload_budget_bytes_per_frame = stats.upload_budget_bytes_per_frame,
                    staging_budget_bytes = stats.staging_budget_bytes,
                    pending_updates = stats.pending_updates,
                    pending_staging_bytes = stats.pending_staging_bytes,
                    yuv_attempted = stats.yuv_conversions_attempted,
                    yuv_applied = stats.yuv_conversions_applied,
                    yuv_convert_us = stats.yuv_convert_us,
                    yuv_output_bytes = stats.yuv_convert_output_bytes,
                    "streaming image updates queued/budgeted"
                );
            }
            return did_work;
        }

        for effect in effects {
            match effect {
                Effect::TextAddFonts { fonts } => {
                    let added = gfx.renderer.add_fonts(fonts);
                    if added == 0 {
                        continue;
                    }

                    let update = fret_runtime::apply_font_catalog_update(
                        &mut self.app,
                        gfx.renderer.all_font_names(),
                        fret_runtime::FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates,
                    );
                    let _ = gfx.renderer.set_text_font_families(&update.config);
                    self.app.set_global::<fret_runtime::TextFontStackKey>(
                        fret_runtime::TextFontStackKey(gfx.renderer.text_font_stack_key()),
                    );
                    window.request_redraw();
                }
                Effect::CursorSetIcon { icon, .. } => {
                    let cursor = match icon {
                        fret_core::CursorIcon::Default => winit::cursor::CursorIcon::Default,
                        fret_core::CursorIcon::Pointer => winit::cursor::CursorIcon::Pointer,
                        fret_core::CursorIcon::Text => winit::cursor::CursorIcon::Text,
                        fret_core::CursorIcon::ColResize => winit::cursor::CursorIcon::ColResize,
                        fret_core::CursorIcon::RowResize => winit::cursor::CursorIcon::RowResize,
                    };
                    window.set_cursor(Cursor::Icon(cursor));
                }
                Effect::ImageRegisterRgba8 {
                    window: target_window,
                    token,
                    width,
                    height,
                    bytes,
                    color_info,
                    alpha_mode,
                } => {
                    if target_window != self.app_window {
                        continue;
                    }

                    if width == 0 || height == 0 {
                        self.pending_events.push(Event::ImageRegisterFailed {
                            token,
                            message: format!("invalid image size: {width}x{height}"),
                        });
                        continue;
                    }

                    let expected_len = (width as usize)
                        .saturating_mul(height as usize)
                        .saturating_mul(4);
                    if bytes.len() != expected_len {
                        self.pending_events.push(Event::ImageRegisterFailed {
                            token,
                            message: format!(
                                "invalid rgba8 byte length: got {} expected {}",
                                bytes.len(),
                                expected_len
                            ),
                        });
                        continue;
                    }

                    let color_space = match color_info.encoding {
                        fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
                        fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
                    };

                    let uploaded = fret_render::upload_rgba8_image(
                        &gfx.ctx.device,
                        &gfx.ctx.queue,
                        (width, height),
                        &bytes,
                        color_space,
                    );

                    let view = uploaded
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let image = gfx.renderer.register_image(fret_render::ImageDescriptor {
                        view,
                        size: uploaded.size,
                        format: uploaded.format,
                        color_space: uploaded.color_space,
                        alpha_mode,
                    });
                    self.uploaded_images.insert(
                        image,
                        UploadedImageEntry {
                            uploaded,
                            stream_generation: 0,
                            alpha_mode,
                            nv12_planes: None,
                        },
                    );

                    self.pending_events.push(Event::ImageRegistered {
                        token,
                        image,
                        width,
                        height,
                    });
                    window.request_redraw();
                }
                Effect::ImageUpdateRgba8 {
                    window: target_window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    bytes_per_row,
                    bytes,
                    color_info,
                    alpha_mode,
                } => {
                    self.apply_streaming_image_update_rgba8(
                        window,
                        gfx,
                        &mut stats,
                        target_window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        bytes_per_row,
                        &bytes,
                        color_info,
                        alpha_mode,
                    );
                }
                Effect::ImageUpdateNv12 {
                    window: target_window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    y_bytes_per_row,
                    y_plane,
                    uv_bytes_per_row,
                    uv_plane,
                    color_info,
                    alpha_mode: _,
                } => {
                    stats.yuv_conversions_attempted =
                        stats.yuv_conversions_attempted.saturating_add(1);
                    if self.try_apply_streaming_image_update_nv12_gpu(
                        window,
                        gfx,
                        &mut stats,
                        target_window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        &y_plane,
                        uv_bytes_per_row,
                        &uv_plane,
                        color_info,
                    ) {
                        continue;
                    }
                    let t0 = fret_core::time::Instant::now();
                    match super::yuv::nv12_to_rgba8_rect(super::yuv::Nv12ToRgba8RectInput {
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        y_plane: &y_plane,
                        uv_bytes_per_row,
                        uv_plane: &uv_plane,
                        range: color_info.range,
                        matrix: color_info.matrix,
                    }) {
                        Ok((rect, rgba)) => {
                            stats.yuv_conversions_applied =
                                stats.yuv_conversions_applied.saturating_add(1);
                            stats.yuv_convert_us = stats
                                .yuv_convert_us
                                .saturating_add(t0.elapsed().as_micros() as u64);
                            stats.yuv_convert_output_bytes = stats
                                .yuv_convert_output_bytes
                                .saturating_add(rgba.len() as u64);

                            self.apply_streaming_image_update_rgba8(
                                window,
                                gfx,
                                &mut stats,
                                target_window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                Some(rect),
                                rect.w.saturating_mul(4),
                                &rgba,
                                fret_core::ImageColorInfo::srgb_rgba(),
                                fret_core::AlphaMode::Opaque,
                            );
                        }
                        Err(_message) => {
                            if self.config.streaming_update_ack_enabled {
                                self.pending_events.push(Event::ImageUpdateDropped {
                                    token,
                                    image,
                                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                                });
                            }
                        }
                    }
                }
                Effect::ImageUpdateI420 {
                    window: target_window,
                    token,
                    image,
                    stream_generation,
                    width,
                    height,
                    update_rect_px,
                    y_bytes_per_row,
                    y_plane,
                    u_bytes_per_row,
                    u_plane,
                    v_bytes_per_row,
                    v_plane,
                    color_info,
                    alpha_mode: _,
                } => {
                    stats.yuv_conversions_attempted =
                        stats.yuv_conversions_attempted.saturating_add(1);
                    let t0 = fret_core::time::Instant::now();
                    match super::yuv::i420_to_rgba8_rect(super::yuv::I420ToRgba8RectInput {
                        width,
                        height,
                        update_rect_px,
                        y_bytes_per_row,
                        y_plane: &y_plane,
                        u_bytes_per_row,
                        u_plane: &u_plane,
                        v_bytes_per_row,
                        v_plane: &v_plane,
                        range: color_info.range,
                        matrix: color_info.matrix,
                    }) {
                        Ok((rect, rgba)) => {
                            stats.yuv_conversions_applied =
                                stats.yuv_conversions_applied.saturating_add(1);
                            stats.yuv_convert_us = stats
                                .yuv_convert_us
                                .saturating_add(t0.elapsed().as_micros() as u64);
                            stats.yuv_convert_output_bytes = stats
                                .yuv_convert_output_bytes
                                .saturating_add(rgba.len() as u64);

                            self.apply_streaming_image_update_rgba8(
                                window,
                                gfx,
                                &mut stats,
                                target_window,
                                token,
                                image,
                                stream_generation,
                                width,
                                height,
                                Some(rect),
                                rect.w.saturating_mul(4),
                                &rgba,
                                fret_core::ImageColorInfo::srgb_rgba(),
                                fret_core::AlphaMode::Opaque,
                            );
                        }
                        Err(_message) => {
                            if self.config.streaming_update_ack_enabled {
                                self.pending_events.push(Event::ImageUpdateDropped {
                                    token,
                                    image,
                                    reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                                });
                            }
                        }
                    }
                }
                Effect::ImageUnregister { image } => {
                    self.uploaded_images.remove(&image);
                    if gfx.renderer.unregister_image(image) {
                        window.request_redraw();
                    }
                }
                Effect::ViewportInput(event) => {
                    self.driver.viewport_input(&mut self.app, event);
                }
                Effect::Dock(op) => {
                    self.driver.dock_op(&mut self.app, op);
                }
                Effect::Window(req) => match req {
                    WindowRequest::Close(target) => {
                        if target != self.app_window {
                            continue;
                        }
                        self.exiting = true;
                        self.dispatcher.shutdown();
                        self.web_cursor.take();
                        event_loop.exit();
                        return true;
                    }
                    WindowRequest::Create(_)
                    | WindowRequest::Raise { .. }
                    | WindowRequest::SetInnerSize { .. } => {}
                },
                Effect::QuitApp => {
                    self.exiting = true;
                    self.dispatcher.shutdown();
                    self.web_cursor.take();
                    event_loop.exit();
                    return true;
                }
                Effect::HideApp | Effect::HideOtherApps | Effect::UnhideAllApps => {}
                Effect::Command { window, command } => match window {
                    Some(target) if target == self.app_window => {
                        self.driver.handle_command(
                            WinitCommandContext {
                                app: &mut self.app,
                                services: &mut gfx.renderer,
                                window: self.app_window,
                                state,
                            },
                            command,
                        );
                    }
                    None => {
                        self.driver.handle_global_command(
                            WinitGlobalContext {
                                app: &mut self.app,
                                services: &mut gfx.renderer,
                            },
                            command,
                        );
                    }
                    _ => {}
                },
                Effect::Redraw(target) | Effect::RequestAnimationFrame(target) => {
                    if target == self.app_window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        }

        let streaming_snapshot_enabled = self.config.streaming_perf_snapshot_enabled
            || std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty());
        let streaming_stats_have_activity = stats.update_effects_seen > 0
            || stats.update_effects_enqueued > 0
            || stats.update_effects_replaced > 0
            || stats.update_effects_applied > 0
            || stats.update_effects_delayed_budget > 0
            || stats.update_effects_dropped_staging > 0
            || stats.upload_bytes_budgeted > 0
            || stats.upload_bytes_applied > 0
            || stats.pending_updates > 0
            || stats.pending_staging_bytes > 0
            || stats.yuv_conversions_attempted > 0
            || stats.yuv_convert_us > 0;
        if streaming_snapshot_enabled && streaming_stats_have_activity {
            self.app.set_global(fret_core::StreamingUploadPerfSnapshot {
                frame_id: self.frame_id,
                upload_budget_bytes_per_frame: stats.upload_budget_bytes_per_frame,
                staging_budget_bytes: stats.staging_budget_bytes,
                update_effects_seen: u64::from(stats.update_effects_seen),
                update_effects_enqueued: u64::from(stats.update_effects_enqueued),
                update_effects_replaced: u64::from(stats.update_effects_replaced),
                update_effects_applied: u64::from(stats.update_effects_applied),
                update_effects_delayed_budget: u64::from(stats.update_effects_delayed_budget),
                update_effects_dropped_staging: u64::from(stats.update_effects_dropped_staging),
                upload_bytes_budgeted: stats.upload_bytes_budgeted,
                upload_bytes_applied: stats.upload_bytes_applied,
                pending_updates: u64::from(stats.pending_updates),
                pending_staging_bytes: stats.pending_staging_bytes,
                yuv_convert_us: stats.yuv_convert_us,
                yuv_convert_output_bytes: stats.yuv_convert_output_bytes,
                yuv_conversions_attempted: u64::from(stats.yuv_conversions_attempted),
                yuv_conversions_applied: u64::from(stats.yuv_conversions_applied),
            });
        }

        if std::env::var_os("FRET_STREAMING_DEBUG").is_some_and(|v| !v.is_empty())
            && (stats.update_effects_delayed_budget > 0
                || stats.update_effects_dropped_staging > 0
                || stats.update_effects_replaced > 0
                || stats.yuv_conversions_attempted > 0)
        {
            tracing::debug!(
                seen = stats.update_effects_seen,
                enqueued = stats.update_effects_enqueued,
                replaced = stats.update_effects_replaced,
                applied = stats.update_effects_applied,
                delayed_budget = stats.update_effects_delayed_budget,
                dropped_staging = stats.update_effects_dropped_staging,
                upload_bytes_budgeted = stats.upload_bytes_budgeted,
                upload_bytes_applied = stats.upload_bytes_applied,
                upload_budget_bytes_per_frame = stats.upload_budget_bytes_per_frame,
                staging_budget_bytes = stats.staging_budget_bytes,
                pending_updates = stats.pending_updates,
                pending_staging_bytes = stats.pending_staging_bytes,
                yuv_attempted = stats.yuv_conversions_attempted,
                yuv_applied = stats.yuv_conversions_applied,
                yuv_convert_us = stats.yuv_convert_us,
                yuv_output_bytes = stats.yuv_convert_output_bytes,
                "streaming image updates queued/budgeted"
            );
        }

        true
    }

    fn drain_inboxes(&mut self, window: Option<AppWindowId>) -> bool {
        let did_work = self.app.with_global_mut_untracked(
            fret_runtime::InboxDrainRegistry::default,
            |registry, app| registry.drain_all(app, window),
        );
        tracing::trace!(?window, did_work, "driver: drain_inboxes");
        did_work
    }

    fn dispatch_events(&mut self, gfx: &mut GfxState, state: &mut D::WindowState) -> bool {
        let events = std::mem::take(&mut self.pending_events);
        let mut did_work = !events.is_empty();
        for event in events {
            apply_window_metrics_event(&mut self.app, self.app_window, &event);
            self.driver.handle_event(
                WinitEventContext {
                    app: &mut self.app,
                    services: &mut gfx.renderer,
                    window: self.app_window,
                    state,
                },
                &event,
            );
        }

        let changed_models = self.app.take_changed_models();
        if !changed_models.is_empty() {
            did_work = true;
            self.driver.handle_model_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window: self.app_window,
                    state,
                },
                &changed_models,
            );
        }

        let changed_globals = self.app.take_changed_globals();
        if !changed_globals.is_empty() {
            did_work = true;
            self.driver.handle_global_changes(
                WinitWindowContext {
                    app: &mut self.app,
                    window: self.app_window,
                    state,
                },
                &changed_globals,
            );
        }

        did_work
    }

    fn drain_turns(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window: &dyn Window,
        gfx: &mut GfxState,
        state: &mut D::WindowState,
    ) {
        // ADR 0034: coalesce and bound effect/event draining to prevent unbounded "effect storms"
        // while still allowing same-frame fixed-point progress for common chains.
        const MAX_EFFECT_DRAIN_TURNS: usize = 8;

        for _ in 0..MAX_EFFECT_DRAIN_TURNS {
            if self.exiting {
                break;
            }

            let mut did_work = self.drain_effects(event_loop, window, gfx, state);
            did_work |= self.dispatch_events(gfx, state);
            if !did_work {
                break;
            }
        }
    }

    fn render_frame(&mut self, event_loop: &dyn ActiveEventLoop, window: &dyn Window) {
        if self.maybe_exit(event_loop) {
            return;
        }
        if self.exiting {
            return;
        }
        self.adopt_gfx_if_ready();
        self.ensure_gpu_ready_hook();

        let Some(mut gfx) = self.gfx.take() else {
            return;
        };
        let Some(mut state) = self.window_state.take() else {
            self.gfx = Some(gfx);
            return;
        };

        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.frame_id.0 = self.frame_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.app.set_frame_id(self.frame_id);

        self.platform.prepare_frame(window);

        let scale = window.scale_factor();
        let physical = Self::desired_surface_size(window).unwrap_or_else(|| window.surface_size());
        let logical: LogicalSize<f32> = physical.to_logical(scale);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(logical.width), Px(logical.height)),
        );

        let (cur_w, cur_h) = gfx.surface_state.size();
        if (cur_w, cur_h) != (physical.width.max(1), physical.height.max(1)) {
            gfx.surface_state.resize(
                &gfx.ctx.device,
                physical.width.max(1),
                physical.height.max(1),
            );
        }

        self.drain_turns(event_loop, window, &mut gfx, &mut state);

        let scale_factor = scale as f32;
        self.driver.gpu_frame_prepare(
            &mut self.app,
            self.app_window,
            &mut state,
            &gfx.ctx,
            &mut gfx.renderer,
            scale_factor,
        );

        self.scene.clear();
        self.driver.render(WinitRenderContext {
            app: &mut self.app,
            services: &mut gfx.renderer,
            window: self.app_window,
            state: &mut state,
            bounds,
            scale_factor,
            scene: &mut self.scene,
        });

        let engine = self.driver.record_engine_frame(
            &mut self.app,
            self.app_window,
            &mut state,
            &gfx.ctx,
            &mut gfx.renderer,
            scale_factor,
            self.tick_id,
            self.frame_id,
        );
        for update in engine.target_updates {
            match update {
                RenderTargetUpdate::Update { id, desc } => {
                    let _ = gfx.renderer.update_render_target(id, desc);
                }
                RenderTargetUpdate::Unregister { id } => {
                    let _ = gfx.renderer.unregister_render_target(id);
                }
            }
        }

        let (frame, view) = match gfx.surface_state.get_current_frame_view() {
            Ok(v) => v,
            Err(err) => {
                if gfx.last_surface_error.as_ref() != Some(&err) {
                    gfx.last_surface_error = Some(err.clone());
                }
                match err {
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                        let (w, h) = gfx.surface_state.size();
                        gfx.surface_state.resize(&gfx.ctx.device, w, h);
                    }
                    wgpu::SurfaceError::Timeout => {}
                    wgpu::SurfaceError::OutOfMemory => panic!("wgpu surface out of memory"),
                    wgpu::SurfaceError::Other => {}
                }
                return;
            }
        };

        let cmd = gfx.renderer.render_scene(
            &gfx.ctx.device,
            &gfx.ctx.queue,
            RenderSceneParams {
                format: gfx.surface_state.format(),
                target_view: &view,
                scene: &self.scene,
                clear: self.config.clear_color,
                scale_factor,
                viewport_size: gfx.surface_state.size(),
            },
        );

        let mut submit: Vec<wgpu::CommandBuffer> = engine.command_buffers;
        submit.push(cmd);
        gfx.ctx.queue.submit(submit);
        frame.present();

        self.drain_turns(event_loop, window, &mut gfx, &mut state);

        self.window_state = Some(state);
        self.gfx = Some(gfx);
    }
}

impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.resumed(event_loop);
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let canvas = fret_runner_winit::canvas_by_id(&self.config.web_canvas_id).ok();
        let append = canvas.is_none();

        let mut attrs =
            WindowAttributes::default().with_title(self.config.main_window_title.clone());
        attrs = attrs.with_platform_attributes(Box::new(
            WindowAttributesWeb::default()
                .with_canvas(canvas)
                .with_append(append)
                .with_focusable(true)
                .with_prevent_default(true),
        ));

        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(_) => return,
        };
        let window: Arc<dyn Window> = Arc::<dyn Window>::from(window);
        self.window_id = Some(window.id());

        if self.window_state.is_none() {
            let state = self
                .driver
                .create_window_state(&mut self.app, self.app_window);
            self.window_state = Some(state);
            self.driver.init(&mut self.app, self.app_window);
        }

        if self.web_cursor.is_none() {
            if let Some(proxy) = self.event_loop_proxy.clone() {
                if let Ok(listener) =
                    fret_runner_winit::install_web_cursor_listener(window.as_ref(), move || {
                        proxy.wake_up();
                    })
                {
                    self.web_cursor = Some(listener);
                }
            }
        }

        if let Some(canvas) = window.canvas().map(|c| c.clone())
            && let Some(mount) = ensure_canvas_ime_mount(&canvas)
        {
            self.web_services.register_ime_mount(self.app_window, mount);
        }

        if let Some(canvas) = window.canvas().map(|c| c.clone()) {
            let gfx_slot = self.pending_gfx.clone();
            let proxy = self.event_loop_proxy.clone();
            let svg_budget = self.config.svg_raster_budget_bytes;
            let intermediate_budget = self.config.renderer_intermediate_budget_bytes;
            let msaa = self.config.path_msaa_samples;
            let font_config = self.config.text_font_families.clone();
            spawn_local(async move {
                let (width, height) = {
                    let web_window = match web_sys::window() {
                        Some(w) => w,
                        None => return,
                    };
                    let dpr = web_window.device_pixel_ratio().max(1.0);
                    let css_w = canvas.client_width().max(0) as f64;
                    let css_h = canvas.client_height().max(0) as f64;
                    let w = (css_w * dpr).round().max(1.0) as u32;
                    let h = (css_h * dpr).round().max(1.0) as u32;
                    canvas.set_width(w);
                    canvas.set_height(h);
                    (w, h)
                };

                let (ctx, surface) = match WgpuContext::new_with_surface(
                    wgpu::SurfaceTarget::Canvas(canvas),
                )
                .await
                {
                    Ok(v) => v,
                    Err(_) => return,
                };

                let surface_state =
                    match SurfaceState::new(&ctx.adapter, &ctx.device, surface, width, height) {
                        Ok(v) => v,
                        Err(_) => return,
                    };

                let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
                renderer.set_svg_raster_budget_bytes(svg_budget);
                renderer.set_intermediate_budget_bytes(intermediate_budget);
                renderer.set_path_msaa_samples(msaa);
                let _ = renderer.set_text_font_families(&font_config);

                *gfx_slot.borrow_mut() = Some(GfxState {
                    ctx,
                    surface_state,
                    renderer,
                    last_surface_error: None,
                });
                if let Some(proxy) = proxy {
                    proxy.wake_up();
                }
            });
        }

        self.window = Some(window);
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        if self.exiting {
            return;
        }
        let Some(window) = self.window.as_ref() else {
            return;
        };
        self.platform
            .input
            .poll_web_cursor_updates(window.scale_factor(), &mut self.pending_events);

        self.web_services.tick();
        self.pending_events.extend(self.web_services.take_events());

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.maybe_exit(event_loop) {
            return;
        }
        if self.exiting {
            return;
        }
        if Some(window_id) != self.window_id {
            return;
        }
        let Some(window) = self.window.as_ref().cloned() else {
            return;
        };
        let window = window.as_ref();

        match &event {
            WindowEvent::CloseRequested => {
                self.pending_events.push(Event::WindowCloseRequested);
                window.request_redraw();
            }
            WindowEvent::SurfaceResized(size) => {
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.surface_state.resize(
                        &gfx.ctx.device,
                        size.width.max(1),
                        size.height.max(1),
                    );
                }
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(gfx) = self.gfx.as_mut() {
                    let size =
                        Self::desired_surface_size(window).unwrap_or_else(|| window.surface_size());
                    gfx.surface_state.resize(
                        &gfx.ctx.device,
                        size.width.max(1),
                        size.height.max(1),
                    );
                }
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.render_frame(event_loop, window);
            }
            _ => {
                self.platform.handle_window_event(
                    window.scale_factor(),
                    &event,
                    &mut self.pending_events,
                );
                if !self.pending_events.is_empty() {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}
