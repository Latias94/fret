//! Runner module namespace for `fret-launch` compatibility.
//!
//! This module intentionally mirrors `fret-launch::runner` so that `fret-launch` can remain a
//! thin facade while keeping long-lived paths stable.

pub use fret_launch_core::common;
pub use fret_launch_core::common::*;

pub use fret_launch_core::font_catalog;
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

mod shared_allocation;

#[cfg(target_os = "windows")]
pub mod windows_mf_video;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod apple_avfoundation_video;

#[cfg(target_os = "android")]
pub mod android_mediacodec_video;

pub mod desktop;

pub use shared_allocation::*;
