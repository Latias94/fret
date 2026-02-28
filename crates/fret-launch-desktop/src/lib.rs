//! Native launcher implementation for Fret (desktop-first).
//!
//! This crate contains the desktop-first `winit` + `wgpu` runner and native-only wiring.
//! Application code should typically depend on `fret-launch` unless it explicitly wants to
//! opt into a smaller, native-only surface.

#[cfg(target_arch = "wasm32")]
compile_error!(
    "`fret-launch-desktop` does not support wasm32; depend on `fret-launch` (web) instead."
);

#[cfg(feature = "dev-state")]
pub mod dev_state;
mod stacksafe_config;

pub mod runner;

pub use fret_launch_core::RunnerError;
pub use runner::desktop::{
    RunnerUserEvent, WinitAppBuilder, WinitRunner, run_app, run_app_with_event_loop,
};

pub use stacksafe_config::configure_stacksafe_from_env;

#[cfg(feature = "dev-state")]
pub use dev_state::DevStateService;
#[cfg(feature = "dev-state")]
pub use dev_state::DevStateWindowKeyRegistry;
#[cfg(feature = "dev-state")]
pub use dev_state::{DevStateExport, DevStateHook, DevStateHooks};
