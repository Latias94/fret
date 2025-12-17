use anyhow::Context as _;

mod renderer;
mod surface;

pub use renderer::{ClearColor, Renderer};
pub use surface::SurfaceState;

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuContext {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .context("request_adapter failed")?;

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
            .context("request_device failed")?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub async fn new_with_surface<'window>(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> anyhow::Result<(Self, wgpu::Surface<'window>)> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(target)
            .context("create_surface failed")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("request_adapter failed")?;

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
            .context("request_device failed")?;

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
    ) -> anyhow::Result<wgpu::Surface<'window>> {
        self.instance
            .create_surface(target)
            .context("create_surface failed")
    }
}
