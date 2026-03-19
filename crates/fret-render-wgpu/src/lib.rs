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
pub use renderer::{ClearColor, RenderSceneParams, Renderer};
pub use renderer::{EffectDegradationCounters, EffectDegradationSnapshot};
pub use renderer::{IntermediatePerfSnapshot, RenderPerfSnapshot, SvgPerfSnapshot};
pub use surface::SurfaceState;
pub use svg::{
    SvgAlphaMask, SvgRgbaImage, UploadedAlphaMask, UploadedRgbaImage, upload_alpha_mask,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Summary of a single backend/adapter initialization attempt.
pub struct WgpuInitAttemptSnapshot {
    pub backends: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_webgpu_compliant: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downlevel_flags: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Diagnostics collected while initializing a `WgpuContext`.
pub struct WgpuInitDiagnosticsSnapshot {
    pub allow_fallback: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_backend: Option<String>,
    pub requested_backend_is_override: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<WgpuInitAttemptSnapshot>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Selected adapter metadata captured for diagnostics and repro bundles.
pub struct WgpuAdapterSelectionSnapshot {
    pub schema_version: u32,
    pub allow_fallback: bool,
    pub required_downlevel_flags: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_backend: Option<String>,
    pub requested_backend_is_override: bool,
    pub selected_backend: String,
    pub adapter_name: String,
    pub driver: String,
    pub driver_info: String,
    pub vendor: u32,
    pub device: u32,
    pub is_webgpu_compliant: bool,
    pub downlevel_flags: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub init_attempts: Vec<WgpuInitAttemptSnapshot>,
}

impl WgpuAdapterSelectionSnapshot {
    pub fn from_context(context: &WgpuContext) -> Self {
        let requested_backend = context.init_diagnostics.requested_backend.clone();
        let info = context.adapter.get_info();
        let downlevel = context.adapter.get_downlevel_capabilities();

        Self {
            schema_version: 2,
            allow_fallback: context.init_diagnostics.allow_fallback,
            required_downlevel_flags: format!("{:?}", fret_required_downlevel_flags()),
            requested_backend_is_override: context.init_diagnostics.requested_backend_is_override,
            requested_backend,
            selected_backend: format!("{:?}", info.backend),
            adapter_name: info.name,
            driver: info.driver,
            driver_info: info.driver_info,
            vendor: info.vendor,
            device: info.device,
            is_webgpu_compliant: downlevel.is_webgpu_compliant(),
            downlevel_flags: format!("{:?}", downlevel.flags),
            init_attempts: context.init_diagnostics.attempts.clone(),
        }
    }

    pub fn from_adapter(adapter: &wgpu::Adapter, requested_backend: Option<String>) -> Self {
        let info = adapter.get_info();
        let downlevel = adapter.get_downlevel_capabilities();

        Self {
            schema_version: 2,
            allow_fallback: false,
            required_downlevel_flags: format!("{:?}", fret_required_downlevel_flags()),
            requested_backend_is_override: requested_backend.is_some(),
            requested_backend,
            selected_backend: format!("{:?}", info.backend),
            adapter_name: info.name,
            driver: info.driver,
            driver_info: info.driver_info,
            vendor: info.vendor,
            device: info.device,
            is_webgpu_compliant: downlevel.is_webgpu_compliant(),
            downlevel_flags: format!("{:?}", downlevel.flags),
            init_attempts: Vec::new(),
        }
    }
}

fn parse_wgpu_backends(raw: &str) -> Option<wgpu::Backends> {
    let mut backends = wgpu::Backends::empty();

    for part in raw.split([',', '|', '+', ' ']) {
        let token = part.trim().to_ascii_lowercase();
        if token.is_empty() {
            continue;
        }

        match token.as_str() {
            "dx12" | "d3d12" => backends |= wgpu::Backends::DX12,
            "vulkan" | "vk" => backends |= wgpu::Backends::VULKAN,
            "metal" => backends |= wgpu::Backends::METAL,
            "gl" | "opengl" => backends |= wgpu::Backends::GL,
            "all" => return Some(wgpu::Backends::all()),
            _ => {}
        }
    }

    (!backends.is_empty()).then_some(backends)
}

fn env_var_trimmed(name: &str) -> Option<String> {
    let raw = std::env::var(name).ok()?;
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn parse_env_bool(name: &str) -> bool {
    let Some(raw) = env_var_trimmed(name) else {
        return false;
    };

    match raw.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => true,
        "0" | "false" | "no" | "n" | "off" => false,
        _ => true,
    }
}

fn allow_fallback_from_env() -> bool {
    cfg!(debug_assertions) && parse_env_bool("FRET_WGPU_ALLOW_FALLBACK")
}

fn backend_override_from_env() -> Result<Option<(String, wgpu::Backends)>, RenderError> {
    let Some(raw) = env_var_trimmed("FRET_WGPU_BACKEND") else {
        return Ok(None);
    };
    let Some(backends) = parse_wgpu_backends(&raw) else {
        return Err(RenderError::InvalidWgpuBackendOverride { raw });
    };
    Ok(Some((raw, backends)))
}

fn default_wgpu_backends_for_target() -> wgpu::Backends {
    #[cfg(target_os = "android")]
    {
        return wgpu::Backends::VULKAN;
    }
    #[cfg(target_os = "ios")]
    {
        return wgpu::Backends::METAL;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        wgpu::Backends::PRIMARY
    }
}

fn create_wgpu_instance_with_backends(backends: wgpu::Backends) -> wgpu::Instance {
    // On Windows/DX12, wgpu defaults to `DxgiFromHwnd`, which does not support transparent
    // windows (and therefore prevents OS-backed background materials like Mica/Acrylic from
    // being visible behind transparent clears).
    //
    // Prefer `DxgiFromVisual` so frameless/utility windows can be composited correctly.
    // Developers can override via `WGPU_DX12_PRESENTATION_SYSTEM=hwnd` when needed (e.g. RenderDoc).
    let mut backend_options = wgpu::BackendOptions::default();
    #[cfg(target_os = "windows")]
    {
        backend_options.dx12 = wgpu::Dx12BackendOptions {
            presentation_system: wgpu::Dx12SwapchainKind::DxgiFromVisual,
            ..Default::default()
        };
    }
    backend_options = backend_options.with_env();

    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        backend_options,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    })
}

fn format_backends(backends: wgpu::Backends) -> String {
    let mut parts = Vec::new();
    if backends.contains(wgpu::Backends::VULKAN) {
        parts.push("vulkan");
    }
    if backends.contains(wgpu::Backends::METAL) {
        parts.push("metal");
    }
    if backends.contains(wgpu::Backends::DX12) {
        parts.push("dx12");
    }
    if backends.contains(wgpu::Backends::GL) {
        parts.push("gl");
    }
    if parts.is_empty() {
        return "none".to_string();
    }
    parts.join("|")
}

fn fret_required_downlevel_flags() -> wgpu::DownlevelFlags {
    // The renderer uses storage buffers in vertex shaders (e.g. quad instance data).
    wgpu::DownlevelFlags::VERTEX_STORAGE
}

fn validate_adapter(adapter: &wgpu::Adapter) -> Result<(), RenderError> {
    let required = fret_required_downlevel_flags();
    let actual = adapter.get_downlevel_capabilities().flags;
    if !actual.contains(required) {
        return Err(RenderError::AdapterMissingRequiredDownlevelFlags {
            required_flags: format!("{:?}", required),
            actual_flags: format!("{:?}", actual),
        });
    }
    Ok(())
}

fn memory_hints_from_env() -> wgpu::MemoryHints {
    let Some(v) = std::env::var_os("FRET_WGPU_MEMORY_HINTS").filter(|v| !v.is_empty()) else {
        return wgpu::MemoryHints::default();
    };

    match v.to_string_lossy().trim().to_ascii_lowercase().as_str() {
        "performance" | "perf" => wgpu::MemoryHints::Performance,
        "memory" | "memoryusage" | "memory_usage" => wgpu::MemoryHints::MemoryUsage,
        _ => wgpu::MemoryHints::default(),
    }
}

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub init_diagnostics: WgpuInitDiagnosticsSnapshot,
}

impl WgpuContext {
    pub async fn new() -> Result<Self, RenderError> {
        let allow_fallback = allow_fallback_from_env();
        let override_env = backend_override_from_env()?;
        let requested_backend = override_env.as_ref().map(|(raw, _)| raw.clone());
        let requested_backend_is_override = override_env.is_some();

        let primary_backends = override_env
            .as_ref()
            .map(|(_, backends)| *backends)
            .unwrap_or_else(default_wgpu_backends_for_target);

        #[allow(unused_mut)]
        let mut candidates = vec![primary_backends];
        if allow_fallback {
            #[cfg(target_os = "android")]
            {
                if primary_backends != wgpu::Backends::GL {
                    candidates.push(wgpu::Backends::GL);
                }
            }
        }

        let mut attempts = Vec::new();
        let mut last_error: Option<RenderError> = None;

        for backends in candidates {
            let instance = create_wgpu_instance_with_backends(backends);

            match instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
            {
                Ok(adapter) => {
                    let info = adapter.get_info();
                    let downlevel = adapter.get_downlevel_capabilities();

                    if let Err(err) = validate_adapter(&adapter) {
                        attempts.push(WgpuInitAttemptSnapshot {
                            backends: format_backends(backends),
                            ok: false,
                            error: Some(format!("{err:?}")),
                            selected_backend: Some(format!("{:?}", info.backend)),
                            adapter_name: Some(info.name),
                            is_webgpu_compliant: Some(downlevel.is_webgpu_compliant()),
                            downlevel_flags: Some(format!("{:?}", downlevel.flags)),
                        });
                        last_error = Some(err);
                        continue;
                    }

                    match adapter
                        .request_device(&wgpu::DeviceDescriptor {
                            label: Some("fret wgpu device"),
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::default(),
                            experimental_features: wgpu::ExperimentalFeatures::default(),
                            memory_hints: memory_hints_from_env(),
                            trace: wgpu::Trace::default(),
                        })
                        .await
                    {
                        Ok((device, queue)) => {
                            attempts.push(WgpuInitAttemptSnapshot {
                                backends: format_backends(backends),
                                ok: true,
                                error: None,
                                selected_backend: Some(format!("{:?}", info.backend)),
                                adapter_name: Some(info.name),
                                is_webgpu_compliant: Some(downlevel.is_webgpu_compliant()),
                                downlevel_flags: Some(format!("{:?}", downlevel.flags)),
                            });

                            return Ok(Self {
                                instance,
                                adapter,
                                device,
                                queue,
                                init_diagnostics: WgpuInitDiagnosticsSnapshot {
                                    allow_fallback,
                                    requested_backend,
                                    requested_backend_is_override,
                                    attempts,
                                },
                            });
                        }
                        Err(source) => {
                            let err = RenderError::RequestDeviceFailed { source };
                            attempts.push(WgpuInitAttemptSnapshot {
                                backends: format_backends(backends),
                                ok: false,
                                error: Some(format!("{err:?}")),
                                selected_backend: Some(format!("{:?}", info.backend)),
                                adapter_name: Some(info.name),
                                is_webgpu_compliant: Some(downlevel.is_webgpu_compliant()),
                                downlevel_flags: Some(format!("{:?}", downlevel.flags)),
                            });
                            last_error = Some(err);
                            continue;
                        }
                    }
                }
                Err(source) => {
                    let err = RenderError::RequestAdapterFailed { source };
                    attempts.push(WgpuInitAttemptSnapshot {
                        backends: format_backends(backends),
                        ok: false,
                        error: Some(format!("{err:?}")),
                        selected_backend: None,
                        adapter_name: None,
                        is_webgpu_compliant: None,
                        downlevel_flags: None,
                    });
                    last_error = Some(err);
                    continue;
                }
            }
        }

        let last_error = last_error.expect("wgpu init attempts are non-empty");
        Err(RenderError::WgpuInitFailed {
            attempt_count: attempts.len(),
            last_error: Box::new(last_error),
            attempts,
        })
    }

    pub async fn new_with_backends(backends: wgpu::Backends) -> Result<Self, RenderError> {
        let instance = create_wgpu_instance_with_backends(backends);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|source| RenderError::RequestAdapterFailed { source })?;

        validate_adapter(&adapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("fret wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: memory_hints_from_env(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|source| RenderError::RequestDeviceFailed { source })?;

        let info = adapter.get_info();
        let downlevel = adapter.get_downlevel_capabilities();
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            init_diagnostics: WgpuInitDiagnosticsSnapshot {
                allow_fallback: false,
                requested_backend: None,
                requested_backend_is_override: false,
                attempts: vec![WgpuInitAttemptSnapshot {
                    backends: format_backends(backends),
                    ok: true,
                    error: None,
                    selected_backend: Some(format!("{:?}", info.backend)),
                    adapter_name: Some(info.name),
                    is_webgpu_compliant: Some(downlevel.is_webgpu_compliant()),
                    downlevel_flags: Some(format!("{:?}", downlevel.flags)),
                }],
            },
        })
    }

    pub async fn new_with_surface<'window>(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> Result<(Self, wgpu::Surface<'window>), RenderError> {
        let allow_fallback = allow_fallback_from_env();
        let override_env = backend_override_from_env()?;
        let requested_backend = override_env.as_ref().map(|(raw, _)| raw.clone());
        let requested_backend_is_override = override_env.is_some();

        let used_backends = override_env
            .as_ref()
            .map(|(_, backends)| *backends)
            .unwrap_or_else(default_wgpu_backends_for_target);
        let instance = create_wgpu_instance_with_backends(used_backends);
        let surface = instance
            .create_surface(target)
            .map_err(|source| RenderError::CreateSurfaceFailed { source })?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|source| RenderError::RequestAdapterFailed { source })?;

        validate_adapter(&adapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("fret wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: memory_hints_from_env(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|source| RenderError::RequestDeviceFailed { source })?;

        let info = adapter.get_info();
        let downlevel = adapter.get_downlevel_capabilities();
        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
                init_diagnostics: WgpuInitDiagnosticsSnapshot {
                    allow_fallback,
                    requested_backend,
                    requested_backend_is_override,
                    attempts: vec![WgpuInitAttemptSnapshot {
                        backends: format_backends(used_backends),
                        ok: true,
                        error: None,
                        selected_backend: Some(format!("{:?}", info.backend)),
                        adapter_name: Some(info.name),
                        is_webgpu_compliant: Some(downlevel.is_webgpu_compliant()),
                        downlevel_flags: Some(format!("{:?}", downlevel.flags)),
                    }],
                },
            },
            surface,
        ))
    }

    pub fn create_surface<'window>(
        &self,
        target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> Result<wgpu::Surface<'window>, RenderError> {
        self.instance
            .create_surface(target)
            .map_err(|source| RenderError::CreateSurfaceFailed { source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wgpu_backends_reports_none_for_empty_or_unknown() {
        assert_eq!(parse_wgpu_backends(""), None);
        assert_eq!(parse_wgpu_backends("   "), None);
        assert_eq!(parse_wgpu_backends("unknown"), None);
        assert_eq!(parse_wgpu_backends("unknown,  ,wat"), None);
    }

    #[test]
    fn parse_wgpu_backends_supports_separators_and_synonyms() {
        assert_eq!(
            parse_wgpu_backends("dx12,vk|metal + gl opengl"),
            Some(
                wgpu::Backends::DX12
                    | wgpu::Backends::VULKAN
                    | wgpu::Backends::METAL
                    | wgpu::Backends::GL
            )
        );
    }

    #[test]
    fn parse_wgpu_backends_all_is_strict_override() {
        assert_eq!(parse_wgpu_backends("all"), Some(wgpu::Backends::all()));
        assert_eq!(
            parse_wgpu_backends("dx12, all, vk"),
            Some(wgpu::Backends::all())
        );
    }
}
