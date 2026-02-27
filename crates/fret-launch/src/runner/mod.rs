//! Runner module namespace for the `fret-launch` facade.
//!
//! This module is intentionally split into:
//! - portable/public runner helpers in `fret-launch-core`,
//! - native implementation in `fret-launch-desktop`,
//! - web implementation kept in-tree (to be split later).

pub use fret_launch_core::common;
pub use fret_launch_core::common::*;

pub use fret_launch_core::imported_viewport_target;
pub use fret_launch_core::native_external_import;
pub use fret_launch_core::streaming_upload;
pub use fret_launch_core::viewport_overlay_immediate_3d;
pub use fret_launch_core::viewport_target;
pub use fret_launch_core::yuv;
pub use fret_launch_core::yuv_gpu;

pub use fret_launch_core::imported_viewport_target::*;
pub use fret_launch_core::native_external_import::*;
pub use fret_launch_core::streaming_upload::*;
pub use fret_launch_core::viewport_overlay_immediate_3d::*;
pub use fret_launch_core::viewport_target::*;

#[cfg(not(target_arch = "wasm32"))]
pub use fret_launch_desktop::runner::SharedAllocationExportError;

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub use fret_launch_desktop::runner::dx12;

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub use fret_launch_desktop::runner::windows_mf_video;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "ios")
))]
pub use fret_launch_desktop::runner::apple_avfoundation_video;

#[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
pub use fret_launch_desktop::runner::android_mediacodec_video;

#[cfg(not(target_arch = "wasm32"))]
pub use fret_launch_desktop::runner::desktop;

#[cfg(not(target_arch = "wasm32"))]
pub use fret_launch_desktop::runner::desktop::{
    RunnerUserEvent, WinitAppBuilder, WinitRunner, run_app, run_app_with_event_loop,
};

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
pub use fret_launch_desktop::runner::desktop::windows_msg_hook;

#[cfg(target_arch = "wasm32")]
pub use fret_launch_web::runner::{
    WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle,
};
#[cfg(target_arch = "wasm32")]
pub use fret_launch_web::runner::{WinitRunner, run_app, run_app_with_event_loop};
