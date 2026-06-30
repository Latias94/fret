mod layout;
mod model;
mod style;
mod widget;

pub use style::ResizablePanelGroupStyle;

#[allow(unused_imports)]
pub(crate) use layout::ResizablePanelGroupLayout;
pub(crate) use layout::compute_resizable_panel_group_layout;
pub(crate) use model::{apply_handle_delta, fractions_from_sizes};
pub(crate) use widget::BoundResizablePanelGroup;

#[cfg(test)]
mod tests;
