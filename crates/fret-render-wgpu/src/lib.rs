#![allow(clippy::too_many_arguments)]

mod capabilities;
mod error;
mod images;
mod perf_store;
mod renderer;
mod surface;
mod svg;
mod svg_cache;
mod targets;
mod text;
mod upload_counters;
pub mod viewport_overlay;

pub use capabilities::{AdapterCapabilities, RendererCapabilities, StreamingImageCapabilities};
pub use error::RenderError;
pub use fret_core::ImageColorSpace;
pub use fret_render_core::RenderTargetColorSpace;
pub use images::{
    ImageDescriptor, ImageRegistry, UploadedRgba8Image, create_rgba8_image_storage,
    upload_rgba8_image, write_rgba8_texture_region,
};
pub use perf_store::{RendererPerfFrameSample, RendererPerfFrameStore};
pub use renderer::{ClearColor, RenderSceneParams, Renderer};
pub use renderer::{IntermediatePerfSnapshot, RenderPerfSnapshot, SvgPerfSnapshot};
pub use surface::SurfaceState;
pub use svg::{
    SMOOTH_SVG_SCALE_FACTOR, SvgAlphaMask, SvgRenderer, SvgRgbaImage, UploadedAlphaMask,
    UploadedRgbaImage, upload_alpha_mask, upload_rgba_image,
};
pub use svg_cache::{CachedSvgImage, SvgImageCache, SvgRasterKind};
pub use targets::{RenderTargetDescriptor, RenderTargetRegistry};
pub use text::FontCatalogEntryMetadata;
pub use text::TextFontFamilyConfig;

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

fn parse_wgpu_backends_from_env() -> Option<wgpu::Backends> {
    let raw = std::env::var("FRET_WGPU_BACKEND").ok()?;
    parse_wgpu_backends(&raw)
}

fn create_wgpu_instance() -> wgpu::Instance {
    let backends = parse_wgpu_backends_from_env().unwrap_or(wgpu::Backends::PRIMARY);
    wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    })
}

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuContext {
    pub async fn new() -> Result<Self, RenderError> {
        let instance = create_wgpu_instance();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|source| RenderError::RequestAdapterFailed { source })?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("fret wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|source| RenderError::RequestDeviceFailed { source })?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub async fn new_with_surface<'window>(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> Result<(Self, wgpu::Surface<'window>), RenderError> {
        let instance = create_wgpu_instance();
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

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("fret wgpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|source| RenderError::RequestDeviceFailed { source })?;

        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
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
