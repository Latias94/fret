use fret_core::Point;
use fret_ui::UiHost;

use super::threshold::exceeds_drag_threshold;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{NodeResize, ViewSnapshot};

pub(super) fn handle_pending_node_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    _cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.node_resize.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_node_resize.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.node_drag_threshold;
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen, zoom) {
        return true;
    }

    canvas.interaction.pending_node_resize = None;
    canvas.interaction.node_resize = Some(NodeResize {
        node: pending.node,
        handle: pending.handle,
        start_pos: pending.start_pos,
        start_node_pos: pending.start_node_pos,
        start_size: pending.start_size,
        start_size_opt: pending.start_size_opt,
        current_node_pos: pending.start_node_pos,
        current_size_opt: pending.start_size_opt,
        current_groups: Vec::new(),
        preview_rev: 0,
    });

    false
}
