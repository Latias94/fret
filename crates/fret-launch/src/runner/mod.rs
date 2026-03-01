//! Runner module namespace for `fret-launch`.
//!
//! This module keeps long-lived runner paths stable while selecting platform-specific
//! implementations via `cfg`.

pub mod common;
pub use common::*;

#[doc(hidden)]
pub mod font_catalog;
pub mod imported_viewport_target;
pub mod native_external_import;
#[doc(hidden)]
pub mod streaming_upload;
pub mod viewport_overlay_immediate_3d;
pub mod viewport_target;
#[doc(hidden)]
pub mod yuv;
#[doc(hidden)]
pub mod yuv_gpu;

pub use imported_viewport_target::*;
pub use native_external_import::*;
pub use streaming_upload::*;
pub use viewport_overlay_immediate_3d::*;
pub use viewport_target::*;

#[cfg(not(target_arch = "wasm32"))]
mod shared_allocation;
#[cfg(not(target_arch = "wasm32"))]
pub use shared_allocation::*;

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub mod windows_mf_video;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "ios")
))]
pub mod apple_avfoundation_video;

#[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
pub mod android_mediacodec_video;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{
    RunnerUserEvent, WinitAppBuilder, WinitRunner, run_app, run_app_with_event_loop,
};

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub use desktop::windows_msg_hook;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::{WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle};
#[cfg(target_arch = "wasm32")]
pub use web::{WinitRunner, run_app, run_app_with_event_loop};
