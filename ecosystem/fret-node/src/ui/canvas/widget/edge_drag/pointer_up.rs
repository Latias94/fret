use fret_ui::UiHost;

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_release_cx::PointerUpReleaseCx,
};

pub(super) fn handle_edge_left_up<H: UiHost, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerUpReleaseCx<H>,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
{
    if canvas.interaction.edge_drag.take().is_some() {
        super::super::pointer_up_finish::finish_pointer_up(cx);
        return true;
    }

    false
}
