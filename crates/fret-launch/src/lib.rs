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
//! - Specialized interop/media helpers live under dedicated submodules such as
//!   [`imported_viewport_target`], [`native_external_import`], [`shared_allocation`], and [`media`].
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

/// Imported viewport render-target helpers for external texture / foreign-surface integrations.
pub mod imported_viewport_target {
    pub use crate::runner::imported_viewport_target::{
        ImportedViewportFallbackUpdate, ImportedViewportFallbacks, ImportedViewportRenderTarget,
        NativeExternalImportOutcome,
    };
}

/// Native external texture import contracts and frame wrappers.
pub mod native_external_import {
    pub use crate::runner::native_external_import::{
        NativeExternalImportError, NativeExternalImportedFrame, NativeExternalTextureFrame,
        OwnedWgpuTextureFrame,
    };
}

/// Shared-allocation interop helpers for renderer-owned textures.
#[cfg(not(target_arch = "wasm32"))]
pub mod shared_allocation {
    pub use crate::runner::SharedAllocationExportError;

    #[cfg(target_os = "windows")]
    pub use crate::runner::dx12;
}

/// Platform-specific media import helpers.
pub mod media {
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    pub use crate::runner::windows_mf_video;

    #[cfg(all(
        not(target_arch = "wasm32"),
        any(target_os = "macos", target_os = "ios")
    ))]
    pub use crate::runner::apple_avfoundation_video;

    #[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
    pub use crate::runner::android_mediacodec_video;
}

pub use runner::{
    EngineFrameKeepalive, EngineFrameUpdate, FnDriver, FnDriverHooks, RenderTargetUpdate,
    ViewportOverlay3dHooks, ViewportOverlay3dHooksService, ViewportOverlay3dImmediateService,
    ViewportRenderTarget, ViewportRenderTargetWithDepth, WgpuInit, WindowCreateSpec,
    WindowLogicalSize, WindowPhysicalPosition, WindowPosition, WinitAppDriver, WinitCommandContext,
    WinitEventContext, WinitGlobalContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext, install_viewport_overlay_3d_immediate,
    record_viewport_overlay_3d, run_app, run_app_with_event_loop,
    upload_viewport_overlay_3d_immediate,
};

#[cfg(not(target_arch = "wasm32"))]
pub use runner::WinitAppBuilder;

#[cfg(target_arch = "wasm32")]
pub use runner::{WebRunnerHandle, run_app_with_handle};
