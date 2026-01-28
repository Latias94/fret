//! Embedded viewport surfaces (render target + input forwarding).
//!
//! This module provides a small, reusable contract for:
//! - allocating and resizing an offscreen render target (`ViewportRenderTarget`)
//! - publishing its `RenderTargetId` as a per-window model
//! - handling `ViewportInputEvent` globally (the golden-path hook is app-wide)
//!
//! The intent is to make "foreign UI embedding" look like:
//! - store an `EmbeddedViewportSurface` in your window state
//! - call `.ensure_size_owned_view(...)` in `record_engine_frame`
//! - render into the returned `TextureView`
//! - use `viewport_surface_panel(...)` in UI
//! - install `handle_viewport_input` as the app-wide viewport input hook

use std::collections::HashMap;
use std::sync::Arc;

use crate::fret::render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_app::App;
use fret_core::{AppWindowId, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputKind};
use fret_launch::{EngineFrameUpdate, ViewportRenderTarget};
use fret_runtime::{FrameId, Model, TickId};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative as kit_decl;

/// Models published for an embedded viewport surface.
#[derive(Debug, Clone)]
pub struct EmbeddedViewportModels {
    pub clicks: Model<u32>,
    pub last_input: Model<Arc<str>>,
    pub target: Model<RenderTargetId>,
}

#[derive(Default)]
struct EmbeddedViewportService {
    by_window: HashMap<AppWindowId, EmbeddedViewportModels>,
    foreign_ui_by_window: HashMap<AppWindowId, Box<dyn EmbeddedViewportForeignUi>>,
}

impl EmbeddedViewportService {
    fn ensure_window(&mut self, app: &mut App, window: AppWindowId) -> EmbeddedViewportModels {
        if let Some(existing) = self.by_window.get(&window) {
            return existing.clone();
        }

        let clicks = app.models_mut().insert(0u32);
        let last_input = app.models_mut().insert(Arc::<str>::from(
            "Click inside the embedded viewport surface.",
        ));
        let target = app.models_mut().insert(RenderTargetId::default());

        let models = EmbeddedViewportModels {
            clicks,
            last_input,
            target,
        };
        self.by_window.insert(window, models.clone());
        models
    }

    fn models(&self, window: AppWindowId) -> Option<EmbeddedViewportModels> {
        self.by_window.get(&window).cloned()
    }

    fn set_foreign_ui(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        ui: Box<dyn EmbeddedViewportForeignUi>,
    ) {
        let _ = self.ensure_window(app, window);
        self.foreign_ui_by_window.insert(window, ui);
    }

    fn foreign_ui_mut(
        &mut self,
        window: AppWindowId,
    ) -> Option<&mut dyn EmbeddedViewportForeignUi> {
        self.foreign_ui_by_window.get_mut(&window).map(|v| &mut **v)
    }
    fn set_target(&mut self, app: &mut App, window: AppWindowId, id: RenderTargetId) {
        let models = self.ensure_window(app, window);
        let _ = app.models_mut().update(&models.target, |v| *v = id);
    }

    fn handle_viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        let Some(models) = self.models(event.window) else {
            return;
        };

        let target = app
            .models()
            .read(&models.target, |v| *v)
            .unwrap_or_default();
        if target == RenderTargetId::default() || event.target != target {
            return;
        }

        if matches!(event.kind, ViewportInputKind::PointerDown { .. }) {
            let _ = app
                .models_mut()
                .update(&models.clicks, |v| *v = v.saturating_add(1));
        }

        let msg: Arc<str> = Arc::from(format!(
            "kind={:?} uv=({:.3},{:.3}) target_px={:?}",
            event.kind, event.uv.0, event.uv.1, event.target_px
        ));
        let _ = app.models_mut().update(&models.last_input, |v| *v = msg);

        if let Some(ui) = self.foreign_ui_mut(event.window) {
            ui.on_viewport_input(app, &event);
        }
        app.request_redraw(event.window);
    }
}

/// A small, object-safe contract for "foreign UI" embedded in an [`EmbeddedViewportSurface`].
///
/// This is intentionally minimal:
/// - input comes in as [`ViewportInputEvent`] in surface UV space
/// - rendering is a wgpu command recording into the offscreen target
///
/// This keeps the interop boundary explicit and makes it easy to host other ecosystems that
/// can render to a `wgpu::TextureView` (or can be adapted to do so).
pub trait EmbeddedViewportForeignUi: 'static {
    /// Handle an input event targeted at this embedded surface.
    fn on_viewport_input(&mut self, _app: &mut App, _event: &ViewportInputEvent) {}

    /// Record rendering commands for the current frame.
    fn record_foreign_frame(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        context: &WgpuContext,
        renderer: &mut Renderer,
        scale_factor: f32,
        tick_id: TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    );

    /// Whether to request another redraw after recording this frame.
    fn request_redraw_after_record(&self) -> bool {
        true
    }
}

/// Install (or replace) a foreign UI instance for the given window.
pub fn set_foreign_ui(app: &mut App, window: AppWindowId, ui: impl EmbeddedViewportForeignUi) {
    app.with_global_mut(EmbeddedViewportService::default, |svc, app| {
        svc.set_foreign_ui(app, window, Box::new(ui));
    });
}

/// Ensure models exist for the given window.
pub fn ensure_models(app: &mut App, window: AppWindowId) -> EmbeddedViewportModels {
    app.with_global_mut(EmbeddedViewportService::default, |svc, app| {
        svc.ensure_window(app, window)
    })
}

/// Read models for the given window (if present).
pub fn models(app: &App, window: AppWindowId) -> Option<EmbeddedViewportModels> {
    app.global::<EmbeddedViewportService>()
        .and_then(|svc| svc.models(window))
}

/// Global hook for `UiAppDriver::viewport_input(...)`.
pub fn handle_viewport_input(app: &mut App, event: ViewportInputEvent) {
    app.with_global_mut(EmbeddedViewportService::default, |svc, app| {
        svc.handle_viewport_input(app, event);
    });
}

/// A per-window offscreen surface owned by the app state.
#[derive(Debug)]
pub struct EmbeddedViewportSurface {
    target: ViewportRenderTarget,
    target_px_size: (u32, u32),
    last_id: RenderTargetId,
}

impl EmbeddedViewportSurface {
    pub fn new(
        format: wgpu::TextureFormat,
        color_space: RenderTargetColorSpace,
        initial_px_size: (u32, u32),
    ) -> Self {
        Self {
            target: ViewportRenderTarget::new(format, color_space),
            target_px_size: initial_px_size,
            last_id: RenderTargetId::default(),
        }
    }

    pub fn target_id(&self) -> RenderTargetId {
        self.last_id
    }

    pub fn target_px_size(&self) -> (u32, u32) {
        self.target_px_size
    }

    pub fn set_target_px_size(&mut self, px_size: (u32, u32)) {
        self.target_px_size = px_size;
    }

    /// Ensure the target exists at the current size and publish its id to the per-window models.
    pub fn ensure_size_owned_view(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        context: &WgpuContext,
        renderer: &mut Renderer,
        label: Option<&str>,
    ) -> (RenderTargetId, wgpu::TextureView) {
        let (id, view) =
            self.target
                .ensure_size_owned_view(context, renderer, self.target_px_size, label);
        self.last_id = id;

        app.with_global_mut(EmbeddedViewportService::default, |svc, app| {
            svc.set_target(app, window, id);
        });

        (id, view)
    }

    /// Create a UI panel that presents this surface as an isolated viewport.
    pub fn panel<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        props: EmbeddedViewportPanelProps,
    ) -> AnyElement {
        let props = kit_decl::viewport_surface::ViewportSurfacePanelProps {
            target: self.last_id,
            target_px_size: self.target_px_size,
            fit: props.fit,
            opacity: props.opacity,
            forward_input: props.forward_input,
        };
        kit_decl::viewport_surface::viewport_surface_panel(cx, props)
    }
}

/// UI props for [`EmbeddedViewportSurface::panel`].
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedViewportPanelProps {
    pub fit: ViewportFit,
    pub opacity: f32,
    pub forward_input: bool,
}

impl Default for EmbeddedViewportPanelProps {
    fn default() -> Self {
        Self {
            fit: ViewportFit::Contain,
            opacity: 1.0,
            forward_input: true,
        }
    }
}

/// A small contract for recording a frame into an embedded surface.
///
/// This is meant to remove the boilerplate of:
/// - ensuring the render target exists/resizes,
/// - creating an encoder,
/// - pushing the command buffer into `EngineFrameUpdate`,
/// while keeping the escape hatch: apps can still provide a custom `record_engine_frame` hook.
pub trait EmbeddedViewportRecord: 'static {
    /// Return the embedded surface stored in your window state.
    fn embedded_viewport_surface(&mut self) -> &mut EmbeddedViewportSurface;

    /// Optional label used for the render target and encoder.
    fn embedded_viewport_label(&self) -> Option<&'static str> {
        None
    }

    /// Record wgpu commands into `encoder` targeting `view`.
    fn record_embedded_viewport(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        context: &WgpuContext,
        renderer: &mut Renderer,
        scale_factor: f32,
        tick_id: TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    );

    /// Whether to request another redraw after recording this frame.
    fn request_redraw_after_record(&self) -> bool {
        true
    }
}

/// A smaller contract for recording into an embedded surface, while delegating the rendering
/// implementation to a registered [`EmbeddedViewportForeignUi`] instance.
pub trait EmbeddedViewportSurfaceOwner: 'static {
    /// Return the embedded surface stored in your window state.
    fn embedded_viewport_surface(&mut self) -> &mut EmbeddedViewportSurface;

    /// Optional label used for the render target and encoder.
    fn embedded_viewport_label(&self) -> Option<&'static str> {
        None
    }
}

/// Boilerplate-free `record_engine_frame` hook for types implementing [`EmbeddedViewportRecord`].
pub fn record_engine_frame<S: EmbeddedViewportRecord>(
    app: &mut App,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut S,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
    tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    ensure_models(app, window);

    let label = st.embedded_viewport_label().unwrap_or("embedded viewport");
    let surface = st.embedded_viewport_surface();
    let (_id, view) = surface.ensure_size_owned_view(app, window, context, renderer, Some(label));

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });

    st.record_embedded_viewport(
        app,
        window,
        context,
        renderer,
        scale_factor,
        tick_id,
        frame_id,
        &view,
        &mut encoder,
    );

    let mut update = EngineFrameUpdate::default();
    update.push_command_buffer(encoder.finish());

    if st.request_redraw_after_record() {
        app.request_redraw(window);
    }

    update
}

/// Boilerplate-free `record_engine_frame` hook for types implementing [`EmbeddedViewportSurfaceOwner`].
///
/// The actual rendering implementation comes from a foreign UI registered via [`set_foreign_ui`].
pub fn record_engine_frame_foreign<S: EmbeddedViewportSurfaceOwner>(
    app: &mut App,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut S,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
    tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    ensure_models(app, window);

    let label = st.embedded_viewport_label().unwrap_or("embedded viewport");
    let surface = st.embedded_viewport_surface();
    let (_id, view) = surface.ensure_size_owned_view(app, window, context, renderer, Some(label));

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });

    let request_redraw_after_record =
        app.with_global_mut(EmbeddedViewportService::default, |svc, app| {
            let Some(ui) = svc.foreign_ui_mut(window) else {
                return true;
            };

            ui.record_foreign_frame(
                app,
                window,
                context,
                renderer,
                scale_factor,
                tick_id,
                frame_id,
                &view,
                &mut encoder,
            );
            ui.request_redraw_after_record()
        });

    let mut update = EngineFrameUpdate::default();
    update.push_command_buffer(encoder.finish());

    if request_redraw_after_record {
        app.request_redraw(window);
    }

    update
}

/// Boilerplate-free `record_engine_frame` hook for MVU window states.
///
/// This lets MVU apps embed a surface by:
/// - implementing [`EmbeddedViewportRecord`] for their user state `S`,
/// - wiring `viewport_input` to [`handle_viewport_input`],
/// - wiring `record_engine_frame` to this function.
pub fn record_engine_frame_mvu<S: EmbeddedViewportRecord, M: 'static>(
    app: &mut App,
    window: AppWindowId,
    ui: &mut fret_ui::UiTree<App>,
    st: &mut crate::mvu::MvuWindowState<S, M>,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
    tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    record_engine_frame(
        app,
        window,
        ui,
        &mut st.user,
        context,
        renderer,
        scale_factor,
        tick_id,
        frame_id,
    )
}

/// Boilerplate-free `record_engine_frame` hook for MVU window states hosting a foreign UI.
pub fn record_engine_frame_mvu_foreign<S: EmbeddedViewportSurfaceOwner, M: 'static>(
    app: &mut App,
    window: AppWindowId,
    ui: &mut fret_ui::UiTree<App>,
    st: &mut crate::mvu::MvuWindowState<S, M>,
    context: &WgpuContext,
    renderer: &mut Renderer,
    scale_factor: f32,
    tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    record_engine_frame_foreign(
        app,
        window,
        ui,
        &mut st.user,
        context,
        renderer,
        scale_factor,
        tick_id,
        frame_id,
    )
}

/// Extension helpers for `UiAppDriver<S>` applications embedding an [`EmbeddedViewportSurface`].
pub trait EmbeddedViewportUiAppDriverExt: Sized {
    /// Install the global input hook and the per-window frame recorder.
    fn drive_embedded_viewport(self) -> Self;
}

impl<S> EmbeddedViewportUiAppDriverExt for crate::UiAppDriver<S>
where
    S: EmbeddedViewportRecord,
{
    fn drive_embedded_viewport(self) -> Self {
        self.viewport_input(handle_viewport_input)
            .record_engine_frame(record_engine_frame::<S>)
    }
}

/// Extension helpers for `UiAppDriver<S>` applications hosting a foreign UI in an embedded surface.
pub trait EmbeddedViewportForeignUiAppDriverExt: Sized {
    /// Install the global input hook and the per-window frame recorder.
    fn drive_embedded_viewport_foreign(self) -> Self;
}

impl<S> EmbeddedViewportForeignUiAppDriverExt for crate::UiAppDriver<S>
where
    S: EmbeddedViewportSurfaceOwner,
{
    fn drive_embedded_viewport_foreign(self) -> Self {
        self.viewport_input(handle_viewport_input)
            .record_engine_frame(record_engine_frame_foreign::<S>)
    }
}

/// Extension helpers for MVU drivers embedding an [`EmbeddedViewportSurface`].
pub trait EmbeddedViewportMvuUiAppDriverExt: Sized {
    /// Install the global input hook and the per-window frame recorder.
    fn drive_embedded_viewport(self) -> Self;
}

impl<S, M> EmbeddedViewportMvuUiAppDriverExt for crate::mvu::MvuUiAppDriver<S, M>
where
    S: EmbeddedViewportRecord,
    M: 'static,
{
    fn drive_embedded_viewport(self) -> Self {
        self.viewport_input(handle_viewport_input)
            .record_engine_frame(record_engine_frame_mvu::<S, M>)
    }
}

/// Extension helpers for MVU drivers hosting a foreign UI in an embedded surface.
pub trait EmbeddedViewportForeignMvuUiAppDriverExt: Sized {
    /// Install the global input hook and the per-window frame recorder.
    fn drive_embedded_viewport_foreign(self) -> Self;
}

impl<S, M> EmbeddedViewportForeignMvuUiAppDriverExt for crate::mvu::MvuUiAppDriver<S, M>
where
    S: EmbeddedViewportSurfaceOwner,
    M: 'static,
{
    fn drive_embedded_viewport_foreign(self) -> Self {
        self.viewport_input(handle_viewport_input)
            .record_engine_frame(record_engine_frame_mvu_foreign::<S, M>)
    }
}

/// Convenience: record a simple clear pass.
pub fn clear_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    label: Option<&str>,
    color: wgpu::Color,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}
