//! Docking UI and interaction policy built on top of `fret-ui` substrate.
//!
//! This crate follows ADR 0075 (Docking Layering, B route):
//! - dock graph/ops/persistence remain in `fret-core`
//! - `fret-ui` stays mechanism-only
//! - docking UI and policy live here

pub mod dock;
pub mod runtime;

pub use dock::{
    ActivatePanelOptions, DockManager, DockPanel, DockSpace, DockViewportOverlayHooks,
    DockViewportOverlayHooksService, ViewportPanel, create_dock_space_node,
};
pub use runtime::{handle_dock_before_close_window, handle_dock_op, handle_dock_window_created};

#[cfg(test)]
mod test_host;
