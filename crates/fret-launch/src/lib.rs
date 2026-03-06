//! Launch/runner facade for Fret apps and advanced integrations.
//!
//! This crate provides the higher-level entrypoints that wire together runner, platform, UI
//! runtime, and renderer. Application code should generally depend on this facade rather than
//! reaching into lower-level runner/platform crates directly.
//!
//! ## Choosing the right layer
//!
//! - App authors should usually start with `fret`.
//! - Manual assembly with curated re-exports and without ecosystem defaults should usually start
//!   with `fret-framework` and then opt into `fret-bootstrap` as needed.
//! - Advanced host/runner integration belongs in `fret-launch`.
//!
//! ## Choosing a driver surface
//!
//! - Prefer [`FnDriver`] / [`FnDriverHooks`] for new advanced integrations.
//! - Treat [`WinitAppDriver`] as a compatibility surface for existing trait-based code.
//! - Use [`run_app`], [`run_app_with_event_loop`], and [`WinitAppBuilder`] for native entrypoint
//!   wiring after choosing a driver surface.
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-launch/README.md`.

#[cfg(all(feature = "dev-state", not(target_arch = "wasm32")))]
pub mod dev_state;
mod error;
mod runner;
mod stacksafe_config;

pub use error::RunnerError;

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
    WinitGlobalContext, WinitHotReloadContext, WinitRenderContext, WinitRunnerConfig,
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
pub use runner::WinitAppBuilder;

#[cfg(not(target_arch = "wasm32"))]
pub use runner::SharedAllocationExportError;

#[cfg(target_arch = "wasm32")]
pub use runner::{WebRunnerHandle, run_app_with_handle};
