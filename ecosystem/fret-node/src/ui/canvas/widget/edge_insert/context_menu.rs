use super::prelude::*;

pub(in super::super) fn open_edge_insert_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge: EdgeId,
    invoked_at: Point,
) {
    let menu_candidates = canvas.list_edge_insert_candidates(cx.app, edge);
    let items = super::super::build_insert_candidate_menu_items(&menu_candidates);

    let snapshot = canvas.sync_view_state(cx.app);
    let menu = super::super::build_context_menu_state(
        canvas,
        invoked_at,
        cx.bounds,
        &snapshot,
        ContextMenuTarget::EdgeInsertNodePicker(edge),
        items,
        menu_candidates,
    );
    super::super::context_menu::apply_context_menu_open_state(
        &mut canvas.interaction,
        menu,
        super::super::context_menu::ContextMenuHoverEdgePolicy::Preserve,
    );
}
