pub mod declarative;
pub mod dock;
pub mod element;
pub mod elements;
pub mod focus_visible;
pub mod host;
pub mod overlay_placement;
pub mod paint;
pub type ItemKey = u64;
pub mod resize_handle;
pub mod resizable_split;
pub mod scroll;
mod svg_source;
#[cfg(test)]
mod test_host;
pub mod text_area;
pub mod text_input;
mod text_input_style;
pub mod theme;
pub mod tree;
pub mod virtual_list;
pub mod widget;

pub use dock::{DockManager, DockPanel, DockPanelContentService, DockSpace, ViewportPanel};
pub use elements::{ElementCx, ElementRuntime, GlobalElementId};
pub use host::UiHost;
pub use resize_handle::ResizeHandle;
pub use resizable_split::ResizableSplit;
pub use scroll::{ScrollHandle, ScrollStrategy, VirtualListScrollHandle};
pub use svg_source::SvgSource;
pub use text_area::{BoundTextArea, TextArea, TextAreaStyle};
pub use text_input::{BoundTextInput, TextInput};
pub use text_input_style::TextInputStyle;
pub use theme::{Theme, ThemeConfig, ThemeSnapshot};
pub use tree::{
    PaintCachePolicy, UiDebugFrameStats, UiDebugHitTest, UiDebugLayerInfo, UiLayerId, UiTree,
};
pub use widget::{CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, Widget};
