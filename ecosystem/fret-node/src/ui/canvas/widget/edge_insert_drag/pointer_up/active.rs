use fret_core::Point;
use fret_ui::UiHost;

use super::super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_release_cx::PointerUpReleaseCx,
};

pub(super) fn handle_active_edge_insert_left_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    position: Point,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpReleaseCx<H>,
{
    let Some(drag) = canvas.interaction.edge_insert_drag.take() else {
        return false;
    };

    if !super::super::super::menu_session::has_active_menu_session(&canvas.interaction) {
        let window = cx.window();
        canvas.open_edge_insert_node_picker(cx.host(), window, drag.edge, position);
    }
    canvas.interaction.hover_edge = None;
    super::super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}
