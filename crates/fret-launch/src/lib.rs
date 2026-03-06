//! Launch/runner facade for Fret apps and demos.
//!
//! This crate provides the higher-level entrypoints that wire together runner, platform, UI
//! runtime, and renderer. Application code should generally depend on this facade rather than
//! reaching into lower-level runner/platform crates directly.
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-launch/README.md`.

#[cfg(all(feature = "dev-state", not(target_arch = "wasm32")))]
pub mod dev_state;
mod error;
/// Compatibility module for older `fret_launch::runner::*` imports.
///
/// Prefer the curated root-level re-exports from `fret_launch::*` for long-lived public entry
/// points when possible.
pub mod runner;
mod stacksafe_config;

pub use error::RunnerError;
pub use stacksafe_config::configure_stacksafe_from_env;

#[cfg(all(feature = "dev-state", not(target_arch = "wasm32")))]
pub use dev_state::DevStateService;
#[cfg(all(feature = "dev-state", not(target_arch = "wasm32")))]
pub use dev_state::DevStateWindowKeyRegistry;
#[cfg(all(feature = "dev-state", not(target_arch = "wasm32")))]
pub use dev_state::{DevStateExport, DevStateHook, DevStateHooks};

pub use runner::{
    EngineFrameKeepalive, EngineFrameUpdate, FnDriver, FnDriverHooks,
    ImportedViewportFallbackUpdate, ImportedViewportFallbacks, ImportedViewportRenderTarget,
    NativeExternalImportError, NativeExternalImportOutcome, NativeExternalImportedFrame,
    NativeExternalTextureFrame, OwnedWgpuTextureFrame, RenderTargetUpdate, ViewportOverlay3dHooks,
    ViewportOverlay3dHooksService, ViewportOverlay3dImmediateService, ViewportRenderTarget,
    ViewportRenderTargetWithDepth, WgpuInit, WindowCreateSpec, WindowLogicalSize,
    WindowPhysicalPosition, WindowPosition, WinitAppDriver, WinitCommandContext, WinitEventContext,
    WinitGlobalContext, WinitHotReloadContext, WinitRenderContext, WinitRunner, WinitRunnerConfig,
    WinitWindowContext, install_viewport_overlay_3d_immediate, record_viewport_overlay_3d, run_app,
    run_app_with_event_loop, upload_viewport_overlay_3d_immediate,
};

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub use runner::dx12;

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub use runner::windows_mf_video;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "ios")
))]
pub use runner::apple_avfoundation_video;

#[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
pub use runner::android_mediacodec_video;

#[cfg(not(target_arch = "wasm32"))]
pub use runner::{RunnerUserEvent, WinitAppBuilder};

#[cfg(not(target_arch = "wasm32"))]
pub use runner::SharedAllocationExportError;

#[cfg(target_arch = "wasm32")]
pub use runner::{WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle};
