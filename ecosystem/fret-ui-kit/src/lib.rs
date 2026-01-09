#![deny(deprecated)]
//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).
//!
//! Note: This crate is declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface.

pub mod declarative;
pub mod headless;
pub mod overlay;
pub mod overlay_controller;
pub mod primitives;
pub mod recipes;
pub mod tooltip_provider;
pub mod tree;
#[cfg(feature = "unstable-internals")]
pub mod window_overlays;
#[cfg(not(feature = "unstable-internals"))]
mod window_overlays;

mod sizing;
mod style;
mod styled;

pub use sizing::{Sizable, Size};
pub use style::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, MetricRef, OverflowRefinement,
    Radius, Space,
};
pub use styled::{RefineStyle, Stylable, Styled, StyledExt};

pub use overlay_controller::{
    OverlayController, OverlayKind, OverlayPresence, OverlayRequest, ToastLayerSpec,
};
pub use window_overlays::{
    DEFAULT_MAX_TOASTS, ToastAction, ToastId, ToastPosition, ToastRequest, ToastStore, ToastVariant,
};

pub use window_overlays::TOAST_VIEWPORT_FOCUS_COMMAND;

/// Attempts to handle a window-scoped command that targets `fret-ui-kit` overlay substrates.
///
/// This is intended to be called by app drivers after `UiTree::dispatch_command` returns `false`.
pub fn try_handle_window_overlays_command<H: fret_ui::UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    app: &mut H,
    window: fret_core::AppWindowId,
    command: &fret_runtime::CommandId,
) -> bool {
    window_overlays::try_handle_window_command(ui, app, window, command)
}

pub use tree::{
    TreeEntry, TreeItem, TreeItemId, TreeRowRenderer, TreeRowState, TreeState, flatten_tree,
};
