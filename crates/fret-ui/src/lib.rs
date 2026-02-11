//! UI runtime contract and mechanisms for the Fret workspace.
//!
//! This crate focuses on the **mechanisms** needed for editor-grade UI (tree, layout, input
//! routing, paint orchestration) rather than policy-heavy components. Radix/shadcn-style
//! interaction policies live in the ecosystem layer (`fret-ui-kit`, `fret-ui-shadcn`) instead.
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-ui/README.md`.

#![cfg_attr(test, allow(clippy::arc_with_non_send_sync))]

pub mod action;
pub mod cache_key;
pub mod canvas;
pub mod declarative;
mod drag_route;
pub mod element;
pub mod elements;
#[cfg(any(test, feature = "compat-retained-widgets"))]
pub mod fixed_split;
pub mod focus_visible;
mod frame_cx;
pub mod frame_pipeline;
pub mod host;
pub mod input_modality;
pub mod internal_drag;
mod layout;
pub use layout::constraints as layout_constraints;
pub use layout::engine as layout_engine;
pub use layout::pass as layout_pass;
pub mod overlay_placement;
pub mod paint;
pub mod pending_shortcut;
pub mod pixel_snap;
mod pointer_motion;
#[cfg(feature = "unstable-retained-bridge")]
pub mod retained_bridge;
pub type ItemKey = u64;
#[allow(dead_code)]
pub(crate) mod resizable_panel_group;
#[allow(dead_code)]
pub(crate) mod resizable_split;
#[allow(dead_code)]
pub(crate) mod resize_handle;
pub mod scroll;
mod svg_source;
#[cfg(test)]
mod test_host;
mod text;
pub(crate) use text::area as text_area;
pub(crate) use text::edit as text_edit;
pub(crate) use text::input as text_input;
pub(crate) use text::props as text_props;
pub(crate) use text::surface as text_surface;
pub mod theme;
pub use theme::keys as theme_keys;
pub(crate) use theme::registry as theme_registry;
pub mod tree;
pub mod virtual_list;
#[allow(dead_code)]
pub(crate) mod widget;
pub(crate) mod windowed_surface_host;

#[cfg(feature = "compat-policy-shortcuts")]
compile_error!(
    "Feature `compat-policy-shortcuts` has been removed. \
Use component-owned action hooks (ADR 0074) via `ElementCx::{pressable_*, dismissible_*, roving_*}` \
or `fret-ui-kit::declarative::action_hooks::ActionHooksExt`."
);

pub use elements::{ElementContext, ElementRuntime, GlobalElementId};
#[cfg(any(test, feature = "compat-retained-widgets"))]
pub use fixed_split::FixedSplit;
pub use frame_cx::{UiFrameContext, UiFrameCx};
pub use host::UiHost;
pub use pending_shortcut::PendingShortcutOverlayState;
pub use resizable_panel_group::ResizablePanelGroupStyle;
pub use scroll::{ScrollHandle, ScrollStrategy, VirtualListScrollHandle};
pub use svg_source::SvgSource;
pub use text::{TextAreaStyle, TextInputStyle};
pub use theme::{Theme, ThemeConfig, ThemeSnapshot};
pub use theme_keys::{ThemeColorKey, ThemeMetricKey};
pub use tree::{
    PaintCachePolicy, UiDebugFrameStats, UiDebugHitTest,
    UiDebugHoverDeclarativeInvalidationHotspot, UiDebugLayerInfo, UiLayerId, UiTree,
};
pub use widget::CommandAvailability;
pub use widget::Invalidation;
