use fret_ui::UiHost;

use crate::core::CanvasPoint;
use crate::ui::canvas::state::NodeDrag;
use crate::ui::canvas::widget::*;

pub(super) fn finish_node_drag_move<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    drag: &NodeDrag,
    auto_pan_delta: CanvasPoint,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: node_drag_move_tail_cx::NodeDragMoveTailCx<H>,
{
    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.host(), |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    canvas.emit_node_drag(drag.primary, &drag.node_ids);
    super::super::widget_tail::invalidate_widget_paint(cx);
}
