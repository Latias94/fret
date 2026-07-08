//! Immediate-mode (`UiWriter`) adapters for `fret-docking`.
//!
//! This module provides a tiny glue layer that lets imui apps embed the declarative docking host.
//!
//! Notes:
//! - Docking remains policy-heavy and stateful; this module only provides embedding helpers.
//! - Window creation and dock ops are handled through `DockSurface` by the runner/driver.

use fret_authoring::UiWriter;
use fret_ui::UiHost;

pub use crate::DockHostOptions as DockSpaceElementOptions;
use crate::imui_dock_space_element;

/// Embed a declarative docking host into an imui output list.
///
/// Panel content is read from the registry installed through `DockSurface`; configure dock graph
/// state before calling this helper in the same immediate render pass.
#[track_caller]
pub fn dock_space_declarative_with<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    options: DockSpaceElementOptions,
) {
    imui_dock_space_element(ui, options);
}

/// Convenience wrapper for [`dock_space_declarative_with`].
#[track_caller]
pub fn dock_space_declarative<H: UiHost + 'static>(ui: &mut impl UiWriter<H>) {
    dock_space_declarative_with(ui, DockSpaceElementOptions::default());
}
