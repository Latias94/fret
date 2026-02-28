//! Runner module namespace for `fret-launch` compatibility.

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

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::{WebRunnerHandle, WinitRunner, run_app, run_app_with_event_loop};
#[cfg(target_arch = "wasm32")]
pub use web::{run_app_with_event_loop_and_handle, run_app_with_handle};
