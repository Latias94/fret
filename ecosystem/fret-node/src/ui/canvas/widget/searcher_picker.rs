use super::*;

pub(super) fn open_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    at: CanvasPoint,
) {
    let menu_candidates = canvas.list_background_insert_candidates(host);
    open_searcher_picker(
        canvas,
        host,
        Point::new(Px(at.x), Px(at.y)),
        ContextMenuTarget::BackgroundInsertNodePicker { at },
        menu_candidates,
    );
}

pub(super) fn open_connection_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    at: CanvasPoint,
) {
    let menu_candidates = canvas.list_connection_insert_candidates(host, from);
    open_searcher_picker(
        canvas,
        host,
        Point::new(Px(at.x), Px(at.y)),
        ContextMenuTarget::ConnectionInsertNodePicker { from, at },
        menu_candidates,
    );
}

pub(super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    edge: EdgeId,
    invoked_at: Point,
) {
    super::edge_insert::open_edge_insert_node_picker(canvas, host, window, edge, invoked_at);
}

fn open_searcher_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    invoked_at: Point,
    target: ContextMenuTarget,
    candidates: Vec<InsertNodeCandidate>,
) {
    let snapshot = canvas.sync_view_state(host);
    let bounds = canvas.interaction.last_bounds.unwrap_or_default();
    canvas.open_searcher_overlay(
        invoked_at,
        bounds,
        &snapshot,
        target,
        candidates,
        SearcherRowsMode::Catalog,
    );
}
