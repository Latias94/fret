//! Docking UI and interaction policy built on top of `fret-ui` substrate.
//!
//! This crate follows ADR 0075 (Docking Layering, B route):
//! - dock graph/ops/persistence remain in `fret-core`
//! - `fret-ui` stays mechanism-only
//! - docking UI and policy live here

pub mod dock;
mod facade;
mod invalidation;
pub mod runtime;

#[cfg(feature = "imui")]
pub mod imui;

#[cfg(feature = "imui")]
pub use dock::imui_dock_space_element;
pub use dock::{
    ActivatePanelOptions, DockManager, DockPanel, DockPanelElement, DockPanelElementRegistry,
    DockPanelElementRegistryService, DockSpaceElementOptions, DockViewportLayout,
    DockViewportOverlayHooks, DockViewportOverlayHooksService, DockingPolicy, DockingPolicyService,
    ViewportPanel, dock_panel_element, dock_space_element, dock_space_element_from_registry,
};
pub use facade::DockingRuntime;
pub use runtime::{handle_dock_before_close_window, handle_dock_op, handle_dock_window_created};

#[cfg(test)]
mod test_host;
