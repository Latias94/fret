use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::EdgeId;
use crate::ui::canvas::state::{EdgeDrag, PendingEdgeInsertDrag};
use crate::ui::canvas::widget::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, paint_invalidation::invalidate_paint,
};

pub(super) fn arm_edge_hit_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    edge_insert_on_alt_drag: bool,
    modifiers: Modifiers,
    edge: EdgeId,
    position: Point,
) {
    if should_arm_pending_edge_insert_drag(edge_insert_on_alt_drag, modifiers) {
        canvas.interaction.pending_edge_insert_drag = Some(PendingEdgeInsertDrag {
            edge,
            start_pos: position,
        });
        canvas.interaction.edge_insert_drag = None;
        canvas.interaction.edge_drag = None;
    } else {
        canvas.interaction.pending_edge_insert_drag = None;
        canvas.interaction.edge_insert_drag = None;
        canvas.interaction.edge_drag = Some(EdgeDrag {
            edge,
            start_pos: position,
        });
    }
    cx.capture_pointer(cx.node);
    invalidate_paint(cx);
}

fn should_arm_pending_edge_insert_drag(
    edge_insert_on_alt_drag: bool,
    modifiers: Modifiers,
) -> bool {
    edge_insert_on_alt_drag && (modifiers.alt || modifiers.alt_gr)
}

#[cfg(test)]
mod tests;
