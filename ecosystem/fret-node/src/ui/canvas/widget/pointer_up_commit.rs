use fret_ui::UiHost;

use crate::ops::GraphOp;

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

    let end_pos = resize.current_node_pos;
    let end_size = resize.current_size_opt;

    let mut ops: Vec<GraphOp> = Vec::new();
    if resize.start_node_pos != end_pos {
        ops.push(GraphOp::SetNodePos {
            id: resize.node,
            from: resize.start_node_pos,
            to: end_pos,
        });
    }
    if resize.start_size_opt != end_size {
        ops.push(GraphOp::SetNodeSize {
            id: resize.node,
            from: resize.start_size_opt,
            to: end_size,
        });
    }

    let group_rect_ops: Vec<GraphOp> = canvas
        .graph
        .read_ref(cx.app, |graph| {
            resize
                .current_groups
                .iter()
                .filter_map(|(id, to)| {
                    let from = graph.groups.get(id).map(|g| g.rect)?;
                    (from != *to).then_some(GraphOp::SetGroupRect {
                        id: *id,
                        from,
                        to: *to,
                    })
                })
                .collect()
        })
        .ok()
        .unwrap_or_default();
    ops.extend(group_rect_ops);

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

    let end = resize.current_rect;
    if end != resize.start_rect {
        let _ = canvas.commit_ops(
            cx.app,
            cx.window,
            Some("Resize Group"),
            vec![GraphOp::SetGroupRect {
                id: resize.group,
                from: resize.start_rect,
                to: end,
            }],
        );
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

    let mut ops: Vec<GraphOp> = Vec::new();
    let end_rect = drag.current_rect;
    if end_rect != drag.start_rect {
        ops.push(GraphOp::SetGroupRect {
            id: drag.group,
            from: drag.start_rect,
            to: end_rect,
        });
    }

    for (id, start) in &drag.nodes {
        let end = drag
            .current_nodes
            .iter()
            .find(|(node_id, _)| node_id == id)
            .map(|(_, p)| *p)
            .unwrap_or(*start);
        if end != *start {
            ops.push(GraphOp::SetNodePos {
                id: *id,
                from: *start,
                to: end,
            });
        }
    }

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
