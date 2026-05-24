use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::super::super::{LeftClickCx, capture_pointer_and_invalidate_paint};
use crate::core::EdgeId;
use crate::ui::canvas::state::{EdgeDrag, PendingEdgeInsertDrag};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn arm_edge_hit_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl LeftClickCx<H>,
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
    capture_pointer_and_invalidate_paint(cx);
}

fn should_arm_pending_edge_insert_drag(
    edge_insert_on_alt_drag: bool,
    modifiers: Modifiers,
) -> bool {
    edge_insert_on_alt_drag && (modifiers.alt || modifiers.alt_gr)
}

#[cfg(test)]
mod tests;
