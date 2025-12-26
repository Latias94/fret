mod error;
mod images;
mod renderer;
mod surface;
mod svg;
mod targets;
mod text;

pub use error::RenderError;
pub use images::{ImageColorSpace, ImageDescriptor, ImageRegistry};
pub use renderer::{ClearColor, RenderSceneParams, Renderer};
pub use surface::SurfaceState;
pub use svg::{SMOOTH_SVG_SCALE_FACTOR, SvgAlphaMask, SvgRenderer};
pub use targets::{RenderTargetColorSpace, RenderTargetDescriptor, RenderTargetRegistry};

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuContext {
    pub async fn new() -> Result<Self, RenderError> {
        let instance = wgpu::Instance::default();
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
        let instance = wgpu::Instance::default();
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
