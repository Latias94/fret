use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_node_resize_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    let Some(resize) = canvas.interaction.node_resize.take() else {
        return false;
    };

    canvas.interaction.pending_node_resize = None;

    let ops = canvas
        .graph
        .read_ref(cx.app, |graph| {
            super::pointer_up_commit_resize::build_node_resize_ops(&resize, graph)
        })
        .ok()
        .unwrap_or_default();
    if !ops.is_empty() {
        let _ = canvas.commit_ops(cx.app, cx.window, Some("Resize Node"), ops);
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_group_resize_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    let Some(resize) = canvas.interaction.group_resize.take() else {
        return false;
    };

    canvas.interaction.pending_group_resize = None;

    let ops = super::pointer_up_commit_resize::build_group_resize_ops(&resize);
    if !ops.is_empty() {
        let _ = canvas.commit_ops(cx.app, cx.window, Some("Resize Group"), ops);
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_group_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    let Some(drag) = canvas.interaction.group_drag.take() else {
        return false;
    };

    canvas.interaction.pending_group_drag = None;

    let ops = super::pointer_up_commit_group_drag::build_group_drag_ops(&drag);
    if !ops.is_empty() {
        let _ = canvas.commit_ops(cx.app, cx.window, Some("Move Group"), ops);
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_node_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    super::pointer_up_node_drag::handle_node_drag_release(canvas, cx, snapshot)
}
