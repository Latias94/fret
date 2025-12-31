mod layout;
mod model;
mod style;
mod widget;

#[allow(unused_imports)]
pub use layout::ResizablePanelGroupLayout;
pub use style::ResizablePanelGroupStyle;
pub use widget::BoundResizablePanelGroup;

pub(crate) use layout::compute_resizable_panel_group_layout;
pub(crate) use model::{apply_handle_delta, fractions_from_sizes};

#[cfg(test)]
mod tests;
