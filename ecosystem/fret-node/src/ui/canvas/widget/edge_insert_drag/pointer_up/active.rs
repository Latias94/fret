use fret_core::Point;
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_active_edge_insert_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
) -> bool {
    let Some(drag) = canvas.interaction.edge_insert_drag.take() else {
        return false;
    };

    if canvas.interaction.searcher.is_none() && canvas.interaction.context_menu.is_none() {
        canvas.open_edge_insert_node_picker(cx.app, cx.window, drag.edge, position);
    }
    canvas.interaction.hover_edge = None;
    super::super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}
