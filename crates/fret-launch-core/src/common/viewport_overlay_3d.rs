use std::sync::Arc;

use fret_app::App;
use fret_render::viewport_overlay::ViewportOverlay3dContext;

/// App-owned, engine-pass 3D viewport overlay hooks (gizmos, debug draw, selection outlines).
///
/// This hook is intentionally wgpu-facing and lives in the runner crate (ADR 0038 / ADR 0130):
/// - the engine owns the render pass topology and depth buffers,
/// - Fret provides a stable place to "draw after the scene" inside the viewport pass,
/// - tool policy remains app/ecosystem-owned (ADR 0027).
pub trait ViewportOverlay3dHooks: Send + Sync + 'static {
    fn record(
        &self,
        app: &mut App,
        window: fret_core::AppWindowId,
        target: fret_core::RenderTargetId,
        pass: &mut wgpu::RenderPass<'_>,
        ctx: &ViewportOverlay3dContext,
    );
}

/// Stores the optional app-owned `ViewportOverlay3dHooks` instance.
#[derive(Default)]
pub struct ViewportOverlay3dHooksService {
    hooks: Vec<Arc<dyn ViewportOverlay3dHooks>>,
}

impl ViewportOverlay3dHooksService {
    pub fn set(&mut self, hooks: Arc<dyn ViewportOverlay3dHooks>) {
        self.hooks.clear();
        self.hooks.push(hooks);
    }

    pub fn push(&mut self, hooks: Arc<dyn ViewportOverlay3dHooks>) {
        self.hooks.push(hooks);
    }

    pub fn clear(&mut self) {
        self.hooks.clear();
    }

    pub fn hooks(&self) -> Option<Arc<dyn ViewportOverlay3dHooks>> {
        self.hooks.last().cloned()
    }

    pub fn hooks_all(&self) -> &[Arc<dyn ViewportOverlay3dHooks>] {
        &self.hooks
    }
}

/// Records app-owned engine-pass viewport overlays into an existing render pass.
///
/// This is a convenience helper over `ViewportOverlay3dHooksService` to keep engine integrations
/// and demos free of boilerplate.
pub fn record_viewport_overlay_3d(
    app: &mut App,
    window: fret_core::AppWindowId,
    target: fret_core::RenderTargetId,
    pass: &mut wgpu::RenderPass<'_>,
    ctx: &ViewportOverlay3dContext,
) {
    let hooks = app
        .global::<ViewportOverlay3dHooksService>()
        .map(|svc| svc.hooks_all().to_vec());
    for hook in hooks.into_iter().flatten() {
        hook.record(app, window, target, pass, ctx);
    }
}
