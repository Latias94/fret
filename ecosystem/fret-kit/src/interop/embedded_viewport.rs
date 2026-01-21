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
use fret_launch::ViewportRenderTarget;
use fret_runtime::Model;
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

#[derive(Debug, Default)]
struct EmbeddedViewportService {
    by_window: HashMap<AppWindowId, EmbeddedViewportModels>,
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
        app.request_redraw(event.window);
    }
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
