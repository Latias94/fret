//! Docking UI and interaction policy built on top of `fret-ui` substrate.
//!
//! This crate follows ADR 0075 (Docking Layering, B route):
//! - dock graph/ops/persistence remain in `fret-core`
//! - `fret-ui` stays mechanism-only
//! - docking UI and policy live here

mod dock;
mod facade;
mod invalidation;
mod runtime;

#[cfg(feature = "imui")]
pub mod imui;

#[cfg(feature = "imui")]
pub use dock::imui_dock_space_element;
pub use dock::{
    DockPanel, DockPanelElementRegistry, DockViewportLayout, DockViewportOverlayHooks,
    DockingPolicy, ViewportPanel,
};
pub use facade::{
    DockHostOptions, DockSurface, DockSurfaceChange, DockSurfacePanelError,
    DockSurfacePanelLocation, DockSurfacePanelOutcome, DockSurfacePanelPlacement,
    DockSurfacePanelSnapshot, DockSurfaceSnapshot, DockSurfaceViewportCloseOutcome,
    DockSurfaceViewportError, DockSurfaceViewportOpenOutcome, DockSurfaceViewportOpenStatus,
    DockSurfaceViewportSession,
};

/// Explicit low-level docking access for framework tests and advanced first-party integrations.
///
/// Ordinary apps should use [`DockSurface`]. This module is intentionally narrower than the old
/// crate root: service globals, free runtime handlers, and direct host constructors are not part of
/// the common public surface.
pub mod advanced {
    pub use crate::dock::{
        ActivatePanelOptions, DockManager, DockPanelCatalog, DockPanelCatalogError, DockWorkspace,
    };
    pub use crate::facade::DockSurfaceDriver;
    pub use crate::runtime::{
        DockRuntimeCommand, recenter_in_window_floatings, request_dock_invalidation,
    };
}

#[cfg(test)]
mod test_host;
