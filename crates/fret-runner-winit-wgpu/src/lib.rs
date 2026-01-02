#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod runner;

#[cfg(target_arch = "wasm32")]
mod wasm_stub;

#[cfg(not(target_arch = "wasm32"))]
pub use error::RunnerError;
#[cfg(not(target_arch = "wasm32"))]
pub use runner::RunnerUserEvent;
#[cfg(not(target_arch = "wasm32"))]
pub use runner::{
    EngineFrameUpdate, RenderTargetUpdate, WgpuInit, WindowCreateSpec, WinitAppBuilder,
    WinitAppDriver, WinitAppDriverAdapter, WinitCommandContext, WinitDriver, WinitEventContext,
    WinitGlobalContext, WinitRenderContext, WinitRunner, WinitRunnerConfig, WinitWindowContext,
};
#[cfg(not(target_arch = "wasm32"))]
pub use runner::{run_app, run_app_with_event_loop};

#[cfg(target_arch = "wasm32")]
pub use wasm_stub::*;
