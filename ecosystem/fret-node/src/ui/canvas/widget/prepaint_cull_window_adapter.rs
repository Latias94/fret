//! Prepaint cull-window adapter contract.
//!
//! This module keeps cull-window route preparation behind a named seam. Concrete context bindings
//! live at the lifecycle entrypoint.

use fret_core::Rect;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) trait PrepaintCullWindowCx<H: UiHost> {
    fn prepaint_cull_window_host(&mut self) -> &mut H;
    fn prepaint_cull_window_bounds(&self) -> Rect;
    fn record_node_graph_cull_window_shift(&mut self, cull_window_key: u64);
}

pub(super) fn sync_prepaint_cull_window<H, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PrepaintCullWindowCx<H>,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
{
    let snapshot = canvas.sync_view_state(cx.prepaint_cull_window_host());
    if !super::retained_widget_cull_window_key::should_track_cull_window(canvas, &snapshot) {
        return;
    }

    let Some(next_key) = super::retained_widget_cull_window_key::build_cull_window_key(
        cx.prepaint_cull_window_bounds(),
        &snapshot,
    ) else {
        return;
    };

    super::retained_widget_cull_window_shift::apply_cull_window_key(canvas, cx, next_key);
}
