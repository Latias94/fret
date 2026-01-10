mod common;
mod streaming_upload;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(target_arch = "wasm32")]
mod web;

pub use common::*;

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::{
    RunnerUserEvent, WinitAppBuilder, WinitRunner, run_app, run_app_with_event_loop,
};
#[cfg(target_arch = "wasm32")]
pub use web::{WebRunnerHandle, run_app_with_event_loop_and_handle, run_app_with_handle};
#[cfg(target_arch = "wasm32")]
pub use web::{WinitRunner, run_app, run_app_with_event_loop};
