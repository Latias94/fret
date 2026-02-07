use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("wgpu request_adapter failed")]
    RequestAdapterFailed {
        #[source]
        source: wgpu::RequestAdapterError,
    },

    #[error("wgpu request_device failed")]
    RequestDeviceFailed {
        #[source]
        source: wgpu::RequestDeviceError,
    },

    #[error("wgpu create_surface failed")]
    CreateSurfaceFailed {
        #[source]
        source: wgpu::CreateSurfaceError,
    },

    #[error("surface reported no supported formats")]
    SurfaceNoFormats,

    #[error("surface reported no present modes")]
    SurfaceNoPresentModes,

    #[error("surface reported no alpha modes")]
    SurfaceNoAlphaModes,

    #[error("wgpu surface get_current_texture failed")]
    SurfaceAcquireFailed {
        #[source]
        source: wgpu::SurfaceError,
    },
}
