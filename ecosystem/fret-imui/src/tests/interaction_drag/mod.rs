use super::*;

use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options, render_cross_window_drag_preview_ghosts,
};

#[derive(Clone)]
struct TestDragPayload {
    label: Arc<str>,
}

mod collection_drag;
mod drag_core;
mod drag_preview;
mod multi_select;
mod sortable;
