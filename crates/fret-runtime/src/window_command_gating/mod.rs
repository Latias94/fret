mod helpers;
mod service;
mod snapshot;

#[cfg(test)]
mod tests;

pub use helpers::{
    best_effort_snapshot_for_window, best_effort_snapshot_for_window_with_input_ctx_fallback,
    command_is_enabled_for_window_with_input_ctx_fallback, snapshot_for_window,
    snapshot_for_window_with_input_ctx_fallback,
};
pub use service::{WindowCommandGatingHandle, WindowCommandGatingService};
pub use snapshot::WindowCommandGatingSnapshot;
