pub mod action;
pub mod declarative;
pub mod drag_route;
pub mod element;
pub mod elements;
pub mod focus_visible;
mod frame_cx;
pub mod host;
pub mod input_modality;
pub mod overlay_placement;
pub mod paint;
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

pub use drag_route::InternalDragRouteService;
pub use elements::{ElementContext, ElementRuntime, GlobalElementId};
pub use frame_cx::{UiFrameContext, UiFrameCx};
pub use host::UiHost;
pub use resizable_panel_group::ResizablePanelGroupStyle;
pub use scroll::{ScrollHandle, ScrollStrategy, VirtualListScrollHandle};
pub use svg_source::SvgSource;
pub use text_area::TextAreaStyle;
pub use text_input_style::TextInputStyle;
pub use theme::{Theme, ThemeConfig, ThemeSnapshot};
pub use theme_keys::{ThemeColorKey, ThemeMetricKey};
pub use tree::{
    PaintCachePolicy, UiDebugFrameStats, UiDebugHitTest, UiDebugLayerInfo, UiLayerId, UiTree,
};
pub use widget::Invalidation;
