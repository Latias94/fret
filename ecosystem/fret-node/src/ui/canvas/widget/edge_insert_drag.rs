use fret_core::Point;
use fret_ui::UiHost;

use super::super::state::{EdgeInsertDrag, ViewSnapshot};
use super::NodeGraphCanvas;
use super::threshold::exceeds_drag_threshold;

pub(super) fn handle_pending_edge_insert_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    if canvas.interaction.edge_insert_drag.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_edge_insert_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold.max(0.0);
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen) {
        return true;
    }

    canvas.interaction.pending_edge_insert_drag = None;
    canvas.interaction.edge_insert_drag = Some(EdgeInsertDrag {
        edge: pending.edge,
        pos: position,
    });
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_edge_insert_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
) -> bool {
    let Some(mut drag) = canvas.interaction.edge_insert_drag.clone() else {
        return false;
    };
    drag.pos = position;
    canvas.interaction.edge_insert_drag = Some(drag);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_edge_insert_left_up<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
) -> bool {
    if canvas.interaction.pending_edge_insert_drag.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(drag) = canvas.interaction.edge_insert_drag.take() {
        if canvas.interaction.searcher.is_none() && canvas.interaction.context_menu.is_none() {
            canvas.open_edge_insert_node_picker(cx.app, cx.window, drag.edge, position);
        }
        canvas.interaction.hover_edge = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    false
}
