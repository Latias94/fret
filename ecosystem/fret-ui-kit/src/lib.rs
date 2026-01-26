#![deny(deprecated)]
//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).
//!
//! Note: This crate is declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface.

pub mod command;
mod corners4;
pub mod declarative;
#[cfg(feature = "dnd")]
pub mod dnd;
mod edges4;
pub mod headless;
pub mod overlay;
pub mod overlay_controller;
pub mod primitives;
pub mod recipes;
pub mod theme_tokens;
pub mod tooltip_provider;
pub mod tree;
pub mod ui;
pub mod ui_builder;
pub mod viewport_tooling;
#[cfg(feature = "unstable-internals")]
pub mod window_overlays;
#[cfg(not(feature = "unstable-internals"))]
mod window_overlays;

mod ui_builder_impls;

mod sizing;
mod style;
mod styled;

pub use corners4::Corners4;
pub use edges4::{Edges4, MarginEdge};
pub use sizing::{Sizable, Size};
pub use style::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, LengthRefinement,
    MetricRef, OverflowRefinement, Radius, ShadowPreset, SignedMetricRef, Space, WidgetState,
    WidgetStateProperty, WidgetStates, merge_override_slot, resolve_override_slot,
    resolve_override_slot_opt,
};
pub use styled::{RefineStyle, Stylable, Styled, StyledExt};
pub use ui_builder::{
    UiBuilder, UiExt, UiIntoElement, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
};

pub use overlay_controller::{
    OverlayArbitrationSnapshot, OverlayController, OverlayKind, OverlayPresence, OverlayRequest,
    OverlayStackEntryKind, ToastLayerSpec, WindowOverlayStackEntry, WindowOverlayStackSnapshot,
};
pub use window_overlays::{
    DEFAULT_MAX_TOASTS, ToastAction, ToastButtonStyle, ToastIconButtonStyle, ToastId,
    ToastLayerStyle, ToastPosition, ToastRequest, ToastStore, ToastTextStyle, ToastVariant,
    ToastVariantColors, ToastVariantPalette,
};

pub use window_overlays::TOAST_VIEWPORT_FOCUS_COMMAND;

// Diagnostics-only exports: used by `fret-bootstrap` to export bundle.json fields.
#[doc(hidden)]
pub use window_overlays::{
    OverlaySynthesisEvent, OverlaySynthesisKind, OverlaySynthesisOutcome, OverlaySynthesisSource,
    WindowOverlaySynthesisDiagnosticsStore,
};

/// Common imports for component/app code using `fret-ui-kit`.
///
/// Recommended: `use fret_ui_kit::prelude::*;`
pub mod prelude {
    pub use crate::command::ElementCommandGatingExt as _;
    pub use crate::declarative::prelude::*;
    pub use crate::declarative::{CachedSubtreeExt, CachedSubtreeProps};
    pub use crate::declarative::{stack, style};
    pub use crate::ui;

    #[cfg(feature = "icons")]
    pub use crate::declarative::icon;
    #[cfg(feature = "icons")]
    pub use fret_icons::IconId;

    pub use crate::{
        ChromeRefinement, ColorFallback, ColorRef, Corners4, Edges4, LayoutRefinement, MarginEdge,
        MetricRef, Radius, ShadowPreset, SignedMetricRef, Size, Space, StyledExt, UiExt,
        WidgetState, WidgetStateProperty, WidgetStates, merge_override_slot, resolve_override_slot,
        resolve_override_slot_opt,
    };
    pub use crate::{OverlayArbitrationSnapshot, OverlayController, OverlayKind, OverlayPresence};
    pub use crate::{OverlayRequest, OverlayStackEntryKind};
    pub use crate::{WindowOverlayStackEntry, WindowOverlayStackSnapshot};

    pub use fret_core::{AppWindowId, Px, TextOverflow, TextWrap, UiServices};
    pub use fret_runtime::{CommandId, Model};
    pub use fret_ui::element::{AnyElement, AnyElementIterExt as _, TextProps};
    pub use fret_ui::{ElementContext, Invalidation, Theme, UiHost, UiTree};
}

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
