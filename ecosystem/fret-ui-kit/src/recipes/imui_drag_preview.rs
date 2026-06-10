//! Immediate drag preview helpers built on top of the typed `imui` drag source seam.
//!
//! This module intentionally lives in `recipes`, not in `imui` itself:
//! - `fret-ui-kit::imui` owns typed drag/drop publication and readout,
//! - this module owns source-side preview presentation policy,
//! - and app code still authors the actual preview content.

mod cross_window;
mod same_window;

pub use cross_window::{
    publish_cross_window_drag_preview_ghost, publish_cross_window_drag_preview_ghost_with_options,
    render_cross_window_drag_preview_ghosts,
};
pub use same_window::{
    DragPreviewGhostOptions, drag_preview_ghost, drag_preview_ghost_with_options,
};
