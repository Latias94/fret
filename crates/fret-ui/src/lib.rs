pub mod declarative;
#[allow(dead_code)]
pub(crate) mod dock;
pub mod element;
pub mod elements;
pub mod focus_visible;
pub mod host;
pub mod overlay_placement;
pub mod paint;
pub type ItemKey = u64;
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
#[allow(dead_code)]
pub(crate) mod text_input;
mod text_input_style;
pub mod theme;
pub mod tree;
pub mod virtual_list;
#[allow(dead_code)]
pub(crate) mod widget;

pub use elements::{ElementCx, ElementRuntime, GlobalElementId};
pub use host::UiHost;
pub use scroll::{ScrollHandle, ScrollStrategy, VirtualListScrollHandle};
pub use svg_source::SvgSource;
pub use text_input_style::TextInputStyle;
pub use theme::{Theme, ThemeConfig, ThemeSnapshot};
pub use tree::{
    PaintCachePolicy, UiDebugFrameStats, UiDebugHitTest, UiDebugLayerInfo, UiLayerId, UiTree,
};
pub use widget::Invalidation;
