use fret_core::RenderTargetId;
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer};

use super::EngineFrameUpdate;

/// Per-frame imported render target intended to be embedded into the UI via `SceneOp::ViewportSurface`.
///
/// Unlike [`super::ViewportRenderTarget`], this helper does **not** call `renderer.update_render_target(...)`
/// directly after initial registration. Instead, it records registry updates as explicit runner
/// deltas (`EngineFrameUpdate.target_updates`) as locked by ADR 0234.
///
/// This is the intended authoring shape for:
/// - platform-provided per-frame views (e.g. "external texture" on web),
/// - engine subsystems that want to keep registry mutation staged through the runner.
///
/// Notes:
/// - A stable `RenderTargetId` is still renderer-owned. The first registration needs `&mut Renderer`.
/// - Subsequent frames should push `RenderTargetUpdate::Update` through `EngineFrameUpdate`.
#[derive(Debug)]
pub struct ImportedViewportRenderTarget {
    id: RenderTargetId,
    format: wgpu::TextureFormat,
    color_space: RenderTargetColorSpace,
}

impl ImportedViewportRenderTarget {
    pub fn new(format: wgpu::TextureFormat, color_space: RenderTargetColorSpace) -> Self {
        Self {
            id: RenderTargetId::default(),
            format,
            color_space,
        }
    }

    pub fn id(&self) -> RenderTargetId {
        self.id
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn color_space(&self) -> RenderTargetColorSpace {
        self.color_space
    }

    pub fn is_registered(&self) -> bool {
        self.id != RenderTargetId::default()
    }

    /// Ensure this imported target has a stable id.
    ///
    /// This performs the initial `register_render_target(...)` call. Callers typically do this
    /// once during a `gpu_ready(...)` / `gpu_frame_prepare(...)`-style hook.
    pub fn ensure_registered(
        &mut self,
        renderer: &mut Renderer,
        view: wgpu::TextureView,
        size: (u32, u32),
    ) -> RenderTargetId {
        if self.is_registered() {
            return self.id;
        }

        let desc = RenderTargetDescriptor {
            view,
            size,
            format: self.format,
            color_space: self.color_space,
        };
        self.id = renderer.register_render_target(desc);
        self.id
    }

    /// Record an imported view update as a runner delta for the current frame.
    ///
    /// Panics if the target has not been registered yet.
    pub fn push_update(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
    ) {
        assert!(
            self.is_registered(),
            "ImportedViewportRenderTarget::push_update requires a registered RenderTargetId"
        );
        let desc = RenderTargetDescriptor {
            view,
            size,
            format: self.format,
            color_space: self.color_space,
        };
        update.update_render_target(self.id, desc);
    }

    /// Record an imported view update and a per-frame keepalive token.
    ///
    /// Use this when the imported view depends on an ephemeral external handle (e.g. a WebCodecs
    /// `VideoFrame`) whose lifetime must be extended until submission.
    pub fn push_update_with_keepalive<T: 'static>(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
        keepalive: T,
    ) {
        self.push_update(update, view, size);
        update.push_keepalive(keepalive);
    }

    /// Record an unregister request as a runner delta and clear the local id.
    pub fn push_unregister(&mut self, update: &mut EngineFrameUpdate) {
        if !self.is_registered() {
            return;
        }
        update.unregister_render_target(self.id);
        self.id = RenderTargetId::default();
    }
}
