use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceAcquireError {
    #[error("wgpu surface acquire timed out")]
    Timeout,

    #[error("wgpu surface is outdated")]
    Outdated,

    #[error("wgpu surface was lost")]
    Lost,

    #[error("wgpu surface ran out of memory")]
    OutOfMemory,

    #[error("wgpu surface acquire failed with an unspecified error")]
    Other,
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("invalid FRET_WGPU_BACKEND override: {raw}")]
    InvalidWgpuBackendOverride { raw: String },

    #[error("wgpu init failed after {attempt_count} attempt(s): {last_error}")]
    WgpuInitFailed {
        attempt_count: usize,
        #[source]
        last_error: Box<RenderError>,
        attempts: Vec<crate::WgpuInitAttemptSnapshot>,
    },

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

    #[error(
        "wgpu adapter rejected by Fret policy: missing downlevel flags (required={required_flags} actual={actual_flags})"
    )]
    AdapterMissingRequiredDownlevelFlags {
        required_flags: String,
        actual_flags: String,
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
        source: SurfaceAcquireError,
    },
}
