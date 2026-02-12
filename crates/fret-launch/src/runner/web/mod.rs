//! Web launcher implementation (winit + wgpu via WebGPU).
//!
//! Submodules:
//! - `app_handler`: winit `ApplicationHandler` glue (window creation + event dispatch entrypoints).
//! - `gfx_init`: async GPU adoption + font seeding + canvas DPI sizing.
//! - `render_loop`: per-frame driving and fixed-point effect/event draining.
//! - `effects`: `Effect` draining and side-effect integration.
//! - `streaming_images`: streaming image updates, including YUV conversion paths.
//! - `ime_mount`: DOM mounting helpers for IME overlays.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

mod dispatcher;
use dispatcher::WebDispatcher;

mod app_handler;
mod effects;
mod gfx_init;
mod ime_mount;
mod render_loop;
mod streaming_images;

use fret_app::App;
use fret_core::{AppWindowId, Event, Scene};
use fret_render::{Renderer, SurfaceState, WgpuContext};
use fret_runtime::{FrameId, PlatformCapabilities, TickId};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

use fret_platform_web::WebPlatformServices;

use super::streaming_upload::StreamingUploadQueue;
use super::{WinitAppDriver, WinitRunnerConfig};
use crate::RunnerError;

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

    exit_requested: Rc<Cell<bool>>,
    exiting: bool,

    event_loop_proxy: Option<EventLoopProxy>,
    dispatcher: WebDispatcher,

    app_window: AppWindowId,
    window: Option<Arc<dyn Window>>,
    window_id: Option<WindowId>,
    window_state: Option<D::WindowState>,

    pending_gfx: Rc<RefCell<Option<GfxState>>>,
    gfx: Option<GfxState>,
    renderer_caps: Option<fret_render::RendererCapabilities>,
    gpu_ready_called: bool,
    scene: Scene,

    pending_events: Vec<Event>,
    tick_id: TickId,
    frame_id: FrameId,

    uploaded_images: HashMap<fret_core::ImageId, streaming_images::UploadedImageEntry>,
    streaming_uploads: StreamingUploadQueue,
    nv12_gpu: Option<super::yuv_gpu::Nv12GpuConverter>,

    platform: fret_runner_winit::WinitPlatform,
    web_cursor: Option<fret_runner_web::WebCursorListener>,
    web_services: WebPlatformServices,
    diag_clipboard_force_unavailable: bool,

    environment_media_queries: Option<render_loop::WebEnvironmentMediaQueries>,
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
            exit_requested: Rc::new(Cell::new(false)),
            exiting: false,
            event_loop_proxy: None,
            dispatcher,
            app_window: AppWindowId::default(),
            window: None,
            window_id: None,
            window_state: None,
            pending_gfx: Rc::new(RefCell::new(None)),
            gfx: None,
            renderer_caps: None,
            gpu_ready_called: false,
            scene: Scene::default(),
            pending_events: Vec::new(),
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            uploaded_images: HashMap::new(),
            streaming_uploads: StreamingUploadQueue::default(),
            nv12_gpu: None,
            platform: fret_runner_winit::WinitPlatform::default(),
            web_cursor: None,
            web_services: WebPlatformServices::default(),
            diag_clipboard_force_unavailable: false,
            environment_media_queries: None,
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
        available.shell.share_sheet = false;
        available.shell.incoming_open = false;
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
        caps.shell.share_sheet &= available.shell.share_sheet;
        caps.shell.incoming_open &= available.shell.incoming_open;
        caps.gfx.native_gpu &= available.gfx.native_gpu;
        caps.gfx.webgpu &= available.gfx.webgpu;
        caps
    }
}
