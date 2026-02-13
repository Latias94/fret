mod common;
mod font_catalog;
mod imported_viewport_target;
mod streaming_upload;
mod viewport_overlay_immediate_3d;
mod viewport_target;
mod yuv;
mod yuv_gpu;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

pub use common::*;
pub use imported_viewport_target::*;
pub use viewport_overlay_immediate_3d::*;
pub use viewport_target::*;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{
    RunnerUserEvent, WinitAppBuilder, WinitRunner, run_app, run_app_with_event_loop,
};
#[cfg(target_arch = "wasm32")]
pub use web::{WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle};
#[cfg(target_arch = "wasm32")]
pub use web::{WinitRunner, run_app, run_app_with_event_loop};
