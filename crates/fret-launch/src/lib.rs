//! Launch/runner facade for Fret apps and demos.
//!
//! This crate provides the higher-level entrypoints that wire together runner, platform, UI
//! runtime, and renderer. Application code should generally depend on this facade rather than
//! reaching into lower-level runner/platform crates directly.
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-launch/README.md`.

mod error;
pub mod runner;
mod stacksafe_config;

pub use error::RunnerError;
pub use stacksafe_config::configure_stacksafe_from_env;

pub use runner::{
    EngineFrameUpdate, FnDriver, FnDriverHooks, RenderTargetUpdate, ViewportOverlay3dHooks,
    ViewportOverlay3dHooksService, ViewportOverlay3dImmediateService, ViewportRenderTarget,
    ViewportRenderTargetWithDepth, WgpuInit, WindowCreateSpec, WinitAppDriver, WinitCommandContext,
    WinitEventContext, WinitGlobalContext, WinitHotReloadContext, WinitRenderContext, WinitRunner,
    WinitRunnerConfig, WinitWindowContext, install_viewport_overlay_3d_immediate,
    record_viewport_overlay_3d, run_app, run_app_with_event_loop,
    upload_viewport_overlay_3d_immediate,
};

#[cfg(not(target_arch = "wasm32"))]
pub use runner::{RunnerUserEvent, WinitAppBuilder};

#[cfg(target_arch = "wasm32")]
pub use runner::{WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle};
