use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::*;

pub(super) fn handle_commit_release_chain<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    super::super::super::pointer_up_commit::handle_node_resize_release(canvas, cx)
        || super::super::super::pointer_up_commit::handle_group_resize_release(canvas, cx)
        || super::super::super::pointer_up_commit::handle_group_drag_release(canvas, cx)
        || super::super::super::pointer_up_commit::handle_node_drag_release(canvas, cx, snapshot)
}
