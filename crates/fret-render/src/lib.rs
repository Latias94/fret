//! Compatibility facade for Fret's default renderer backend.
//!
//! Today the default renderer is wgpu-based (`fret-render-wgpu`). This crate exists to keep the
//! historical `fret-render` crate name stable while we split backend implementations into
//! explicit crates.
//!
//! Supported integration topologies:
//!
//! - Editor-hosted convenience path:
//!   create a [`WgpuContext`] with [`WgpuContext::new`] or [`WgpuContext::new_with_surface`], then
//!   build [`Renderer`] and [`SurfaceState`] from that context's adapter/device.
//! - Engine-hosted direct path:
//!   keep the engine-owned `wgpu::Instance` / `Adapter` / `Device` / `Queue`, derive
//!   [`RendererCapabilities`] with [`RendererCapabilities::from_adapter_device`],
//!   then call [`Renderer::new`], [`SurfaceState::new`], and [`Renderer::render_scene`] directly
//!   without routing through [`WgpuContext`].
//!
//! Stable v1 facade buckets:
//!
//! - Core runtime/bootstrap entrypoints:
//!   [`Renderer`], [`RenderSceneParams`], [`SurfaceState`], [`WgpuContext`], [`ClearColor`],
//!   [`RenderError`]
//! - Capability and adapter snapshots:
//!   [`RendererCapabilities`], [`AdapterCapabilities`], [`StreamingImageCapabilities`],
//!   [`WgpuAdapterSelectionSnapshot`], [`WgpuInitDiagnosticsSnapshot`],
//!   [`WgpuInitAttemptSnapshot`]
//! - Render-target / ingest contracts:
//!   [`RenderTargetDescriptor`] and the `RenderTarget*` metadata/value enums re-exported here
//! - Diagnostics/report stores used by first-party runners and tooling:
//!   [`RendererPerfFrameStore`], [`WgpuHubReportFrameStore`], [`WgpuAllocatorReportFrameStore`],
//!   plus the related sample/summary snapshot types
//! - External image/SVG upload helpers and viewport overlay support:
//!   [`ImageDescriptor`], [`UploadedRgba8Image`], [`create_rgba8_image_storage`],
//!   [`upload_rgba8_image`], [`write_rgba8_texture_region`], [`SvgAlphaMask`],
//!   [`SvgRgbaImage`], [`upload_alpha_mask`], [`upload_rgba_image`], [`viewport_overlay`]
//!
//! Depend on `fret-render-wgpu` directly only when you need backend-specific diagnostics or helper
//! surfaces that are intentionally not part of this default facade.

#[cfg(feature = "backend-wgpu")]
pub use fret_render_wgpu::{
    AdapterCapabilities, BlurQualityCounters, BlurQualitySnapshot, ClearColor,
    EffectDegradationCounters, EffectDegradationSnapshot, ImageColorSpace, ImageDescriptor,
    IntermediatePerfSnapshot, RenderError, RenderPerfSnapshot, RenderSceneParams,
    RenderTargetAlphaMode, RenderTargetColorEncoding, RenderTargetColorPrimaries,
    RenderTargetColorRange, RenderTargetColorSpace, RenderTargetDescriptor,
    RenderTargetIngestStrategy, RenderTargetMatrixCoefficients, RenderTargetMetadata,
    RenderTargetOrientation, RenderTargetRotation, RenderTargetTransferFunction, Renderer,
    RendererCapabilities, RendererPerfFrameSample, RendererPerfFrameStore,
    StreamingImageCapabilities, SurfaceState, SvgAlphaMask, SvgPerfSnapshot, SvgRgbaImage,
    SystemFontRescanResult, SystemFontRescanSeed, TextFontFamilyConfig, UploadedRgba8Image,
    WgpuAdapterSelectionSnapshot, WgpuAllocatorReportFrameSample, WgpuAllocatorReportFrameStore,
    WgpuAllocatorReportSummary, WgpuAllocatorReportTopAllocation, WgpuContext, WgpuHubReportCounts,
    WgpuHubReportFrameSample, WgpuHubReportFrameStore, WgpuInitAttemptSnapshot,
    WgpuInitDiagnosticsSnapshot, create_rgba8_image_storage, upload_alpha_mask, upload_rgba_image,
    upload_rgba8_image, viewport_overlay, write_rgba8_texture_region,
};

#[cfg(not(any(feature = "backend-wgpu")))]
compile_error!("fret-render requires at least one backend feature enabled (e.g. `backend-wgpu`).");

#[cfg(all(test, feature = "backend-wgpu"))]
mod tests {
    use super::*;

    #[test]
    fn facade_reexports_default_renderer_surface() {
        let _ = std::mem::size_of::<Renderer>();
        let _ = std::mem::size_of::<RendererCapabilities>();
        let _ = std::mem::size_of::<SurfaceState>();
        let _ = std::mem::size_of::<RenderSceneParams>();
        let _ = std::mem::size_of::<RenderError>();
        let _ = std::mem::size_of::<TextFontFamilyConfig>();
        let _ = std::mem::size_of::<RenderTargetColorSpace>();
        let _ = std::mem::size_of::<WgpuContext>();
        let _ = std::mem::size_of::<WgpuAdapterSelectionSnapshot>();
        let _ = std::mem::size_of::<WgpuInitDiagnosticsSnapshot>();
    }
}
