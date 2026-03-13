use fret_core::{CursorIcon, Point};

use crate::core::EdgeId;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::*;

pub(super) fn resolve_edge_anchor_cursor<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    _position: Point,
) -> Option<CursorIcon> {
    let edge_id =
        target_edge_for_anchor(canvas.interaction.focused_edge, &snapshot.selected_edges)?;
    canvas
        .interaction
        .hover_edge_anchor
        .is_some_and(|(id, _)| id == edge_id)
        .then_some(CursorIcon::Pointer)
}

fn target_edge_for_anchor(
    focused_edge: Option<EdgeId>,
    selected_edges: &[EdgeId],
) -> Option<EdgeId> {
    focused_edge.or_else(|| (selected_edges.len() == 1).then(|| selected_edges[0]))
}

#[cfg(test)]
mod tests;
