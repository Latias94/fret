use super::*;

pub(super) fn has_copyable_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &CommandAvailabilityCx<'_, H>,
) -> bool {
    canvas
        .view_state
        .read_ref(cx.app, |state| {
            !state.selected_nodes.is_empty() || !state.selected_groups.is_empty()
        })
        .ok()
        .unwrap_or(false)
}

pub(super) fn has_any_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &CommandAvailabilityCx<'_, H>,
) -> bool {
    canvas
        .view_state
        .read_ref(cx.app, |state| {
            !state.selected_nodes.is_empty()
                || !state.selected_edges.is_empty()
                || !state.selected_groups.is_empty()
        })
        .ok()
        .unwrap_or(false)
}

pub(super) fn has_any_content<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &CommandAvailabilityCx<'_, H>,
) -> bool {
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            !graph.nodes.is_empty() || !graph.groups.is_empty()
        })
        .ok()
        .unwrap_or(false)
}
