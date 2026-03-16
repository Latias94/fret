use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error(transparent)]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error(transparent)]
    AssetManifest(#[from] fret_assets::AssetManifestLoadError),

    #[error(transparent)]
    AssetStartup(#[from] crate::assets::AssetStartupPlanError),

    #[error("wgpu context is not initialized")]
    WgpuNotInitialized,

    #[error("failed to create OS window")]
    CreateWindowFailed {
        #[source]
        source: winit::error::RequestError,
    },

    #[error(transparent)]
    Render(#[from] fret_render::RenderError),
}
