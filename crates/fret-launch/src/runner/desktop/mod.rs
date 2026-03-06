//! Desktop launcher implementation (winit + wgpu).

mod runner;

pub use runner::{WinitAppBuilder, run_app, run_app_with_event_loop};
