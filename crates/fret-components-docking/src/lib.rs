//! Docking UI and interaction policy built on top of `fret-ui` substrate.
//!
//! This crate follows ADR 0075 (Docking Layering, B route):
//! - dock graph/ops/persistence remain in `fret-core`
//! - `fret-ui` stays mechanism-only
//! - docking UI and policy live here

pub mod dock;

pub use dock::{ActivatePanelOptions, DockManager, DockPanel, DockSpace, ViewportPanel};

#[cfg(test)]
mod test_host;
