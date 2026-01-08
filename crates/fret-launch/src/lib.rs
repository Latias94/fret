mod error;
pub mod runner;

pub use error::RunnerError;

pub use runner::{
    EngineFrameUpdate, FnDriver, FnDriverHooks, RenderTargetUpdate, WgpuInit, WindowCreateSpec,
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitGlobalContext,
    WinitHotReloadContext, WinitRenderContext, WinitRunner, WinitRunnerConfig, WinitWindowContext,
    run_app, run_app_with_event_loop,
};

#[cfg(not(target_arch = "wasm32"))]
pub use runner::{RunnerUserEvent, WinitAppBuilder};

#[cfg(target_arch = "wasm32")]
pub use runner::{WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle};
