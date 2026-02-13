use thiserror::Error;

use crate::runner::EngineFrameKeepalive;

/// Error returned by a native external texture import attempt (ADR 0234 D1/D3).
#[derive(Debug, Error)]
pub enum NativeExternalImportError {
    /// The importer deterministically determined that this frame cannot be imported on the current
    /// backend/capabilities and callers should fall back to copy paths (ADR 0119).
    #[error("native external import is not supported (deterministic fallback required)")]
    Unsupported,

    /// Import failed due to a backend/platform error. Callers MUST fall back deterministically.
    #[error("native external import failed (deterministic fallback required): {reason}")]
    Failed { reason: &'static str },
}

/// Result of importing a platform-produced frame into a `wgpu::TextureView` suitable for
/// `RenderTargetId` sampling via `SceneOp::ViewportSurface` (ADR 0007 / ADR 0234).
#[derive(Debug)]
pub struct NativeExternalImportedFrame {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub metadata: fret_render::RenderTargetMetadata,
    pub keepalive: EngineFrameKeepalive,
}

/// A platform-produced GPU frame that can be imported into a `wgpu::TextureView` on the runner's
/// device without leaking backend handles into `fret-ui` (ADR 0123).
///
/// This trait is intentionally runner-facing: it lives in `fret-launch` and is consumed inside
/// `record_engine_frame(...)` hooks to produce `EngineFrameUpdate` deltas (ADR 0038 / ADR 0234).
///
/// Contract:
/// - Implementations MUST be capability-gated and return `Unsupported` when a fast path is not
///   available (ADR 0122).
/// - Implementations MUST provide deterministic fallback behavior via errors (ADR 0119).
/// - Implementations SHOULD move any ephemeral platform handles into the returned keepalive token
///   so the runner can keep them alive until submission (ADR 0234 D3).
pub trait NativeExternalTextureFrame: 'static {
    fn import(
        self: Box<Self>,
        ctx: &fret_render::WgpuContext,
        caps: &fret_render::RendererCapabilities,
    ) -> Result<NativeExternalImportedFrame, NativeExternalImportError>;
}
