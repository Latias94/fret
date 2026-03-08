use fret_ui::UiHost;

use crate::runtime::callbacks::NodeDragEndOutcome;
use crate::ui::canvas::state::{NodeDrag, ViewSnapshot};

use super::pointer_up_finish::finish_pointer_up;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_node_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let Some(drag) = canvas.interaction.node_drag.take() else {
        return false;
    };

    let end_positions = super::pointer_up_node_drag_ops::end_positions(&drag);
    let group_overrides = super::pointer_up_node_drag_ops::group_overrides(&drag);
    let parent_changes = super::pointer_up_node_drag_parent::parent_changes(
        canvas,
        cx.app,
        snapshot,
        &drag,
        &end_positions,
        &group_overrides,
    );
    let drag_outcome = super::pointer_up_node_drag_ops::commit_release_ops(
        canvas,
        cx.app,
        cx.window,
        &drag,
        &end_positions,
        &group_overrides,
        &parent_changes,
    );
    finish_node_drag_release(canvas, cx, drag, drag_outcome);
    true
}

fn finish_node_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    drag: NodeDrag,
    drag_outcome: NodeDragEndOutcome,
) {
    canvas.emit_node_drag_end(drag.primary, &drag.node_ids, drag_outcome);
    canvas.interaction.pending_node_drag = None;
    canvas.interaction.snap_guides = None;
    finish_pointer_up(cx);
}
