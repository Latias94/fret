use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_pending_edge_insert_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if canvas.interaction.pending_edge_insert_drag.take().is_some() {
        super::super::super::pointer_up_finish::finish_pointer_up(cx);
        return true;
    }

    false
}
