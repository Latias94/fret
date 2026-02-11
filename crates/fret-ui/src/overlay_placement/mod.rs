mod solver;
mod types;
mod util;

pub use solver::{
    anchored_panel_bounds, anchored_panel_bounds_sized, anchored_panel_layout_ex,
    anchored_panel_layout_ex_with_trace, anchored_panel_layout_sized_ex,
    anchored_panel_layout_sized_ex_with_trace,
};
pub use types::{
    Align, AnchoredPanelLayout, AnchoredPanelLayoutTrace, AnchoredPanelOptions, ArrowLayout,
    ArrowOptions, CollisionOptions, LayoutDirection, Offset, ShiftOptions, Side, StickyMode,
};
pub use util::{inset_rect, intersect_rect};

#[cfg(test)]
mod tests;
