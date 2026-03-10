use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(in super::super) fn handle_node_resize_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    let Some(resize) = super::super::pointer_up_session::take_active_release(
        &mut canvas.interaction.node_resize,
        &mut canvas.interaction.pending_node_resize,
    ) else {
        return false;
    };

    let ops = canvas
        .graph
        .read_ref(cx.app, |graph| {
            super::super::pointer_up_commit_resize::build_node_resize_ops(&resize, graph)
        })
        .ok()
        .unwrap_or_default();
    if !ops.is_empty() {
        let _ = canvas.commit_ops(cx.app, cx.window, Some("Resize Node"), ops);
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(in super::super) fn handle_group_resize_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    let Some(resize) = super::super::pointer_up_session::take_active_release(
        &mut canvas.interaction.group_resize,
        &mut canvas.interaction.pending_group_resize,
    ) else {
        return false;
    };

    let ops = super::super::pointer_up_commit_resize::build_group_resize_ops(&resize);
    if !ops.is_empty() {
        let _ = canvas.commit_ops(cx.app, cx.window, Some("Resize Group"), ops);
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}
