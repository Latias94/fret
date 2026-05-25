use fret_ui::UiHost;

use super::super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
    low_level_adapter::CanvasPointerCaptureReleaseCx,
};

pub(super) fn handle_pending_edge_insert_left_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: CanvasPointerCaptureReleaseCx<H>,
{
    if canvas.interaction.pending_edge_insert_drag.take().is_some() {
        super::super::super::pointer_up_finish::finish_pointer_up(cx);
        return true;
    }

    false
}
