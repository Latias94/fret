//! wgpu-based renderer implementation for the Fret workspace.
//!
//! This crate provides the default GPU renderer used by the native runner stack and exposes
//! diagnostics snapshots useful for tooling and issue reports.
//!
//! Most apps should not depend on this crate directly; prefer the higher-level facades
//! (`fret-framework` or the ecosystem `fret` crate) unless you are assembling a custom stack.
//!
//! Supported integration topologies:
//!
//! - Editor-hosted convenience path:
//!   [`WgpuContext`] bootstraps `Instance` / `Adapter` / `Device` / `Queue` and remains the short
//!   path for tools and first-party runners that let Fret own GPU initialization.
//! - Engine-hosted direct path:
//!   callers can keep host-owned GPU objects and use
//!   [`RendererCapabilities::from_adapter_device`], [`Renderer::new`], [`SurfaceState::new`], and
//!   [`Renderer::render_scene`] directly without constructing a [`WgpuContext`].
//!
//! See `tests/host_provided_gpu_topology_smoke.rs` for the smallest in-tree engine-hosted seam
//! exercise.

#![allow(clippy::too_many_arguments)]

mod capabilities;
mod context;
mod error;
mod images;
mod perf_store;
mod renderer;
mod surface;
mod svg;
mod targets;
mod text;
mod upload_counters;
pub mod viewport_overlay;
mod wgpu_report_store;

pub use capabilities::{AdapterCapabilities, RendererCapabilities, StreamingImageCapabilities};
pub use context::{
    WgpuAdapterSelectionSnapshot, WgpuContext, WgpuInitAttemptSnapshot, WgpuInitDiagnosticsSnapshot,
};
pub use error::{RenderError, SurfaceAcquireError};
pub use fret_core::ImageColorSpace;
pub use fret_render_core::{
    RenderTargetAlphaMode, RenderTargetColorEncoding, RenderTargetColorPrimaries,
    RenderTargetColorRange, RenderTargetColorSpace, RenderTargetIngestStrategy,
    RenderTargetMatrixCoefficients, RenderTargetMetadata, RenderTargetOrientation,
    RenderTargetRotation, RenderTargetTransferFunction,
};
pub use images::{
    ImageDescriptor, UploadedRgba8Image, create_rgba8_image_storage, upload_rgba8_image,
    write_rgba8_texture_region,
};
pub use perf_store::{RendererPerfFrameSample, RendererPerfFrameStore};
pub use renderer::{BlurQualityCounters, BlurQualitySnapshot};
pub use renderer::{ClearColor, RenderSceneParams, RenderSceneSource, Renderer};
pub use renderer::{EffectDegradationCounters, EffectDegradationSnapshot};
pub use renderer::{
    GeometryUploadPerfSnapshot, IntermediatePerfSnapshot, RenderPerfSnapshot,
    SceneEncodingCacheMissHistogramSnapshot, SvgPerfSnapshot,
};
pub use surface::SurfaceState;
pub use svg::{
    SvgAlphaMask, SvgRgbaImage, SvgTextBridgeDiagnosticsSnapshot,
    SvgTextFontFallbackRecordSnapshot, SvgTextFontSelectionMissSnapshot,
    SvgTextMissingGlyphRecordSnapshot, UploadedAlphaMask, UploadedRgbaImage, upload_alpha_mask,
    upload_rgba_image,
};
pub use targets::RenderTargetDescriptor;
pub use text::FontCatalogEntryMetadata;
pub use text::SystemFontRescanResult;
pub use text::SystemFontRescanSeed;
pub use text::TextFontFamilyConfig;
pub use wgpu_report_store::{
    WgpuAllocatorReportFrameSample, WgpuAllocatorReportFrameStore, WgpuAllocatorReportSummary,
    WgpuAllocatorReportTopAllocation, WgpuHubReportCounts, WgpuHubReportFrameSample,
    WgpuHubReportFrameStore,
};
