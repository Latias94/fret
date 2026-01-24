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
pub mod layout_constraints;
pub mod layout_engine;
pub mod layout_pass;
pub mod overlay_placement;
pub mod paint;
pub mod pending_shortcut;
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
#[allow(dead_code)]
pub(crate) mod text_area;
mod text_edit;
#[allow(dead_code)]
pub(crate) mod text_input;
mod text_input_style;
mod text_surface;
pub mod theme;
pub mod theme_keys;
pub(crate) mod theme_registry;
pub mod tree;
pub mod virtual_list;
#[allow(dead_code)]
pub(crate) mod widget;

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
pub use text_area::TextAreaStyle;
pub use text_input_style::TextInputStyle;
pub use theme::{Theme, ThemeConfig, ThemeSnapshot};
pub use theme_keys::{ThemeColorKey, ThemeMetricKey};
pub use tree::{
    PaintCachePolicy, UiDebugFrameStats, UiDebugHitTest,
    UiDebugHoverDeclarativeInvalidationHotspot, UiDebugLayerInfo, UiLayerId, UiTree,
};
pub use widget::CommandAvailability;
pub use widget::Invalidation;
