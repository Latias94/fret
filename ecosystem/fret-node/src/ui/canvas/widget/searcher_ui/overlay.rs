use super::super::*;

pub(super) fn install_searcher_overlay<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    searcher: SearcherState,
) {
    super::super::context_menu::clear_context_menu(&mut canvas.interaction);
    canvas.interaction.searcher = Some(searcher);
}

pub(super) fn open_searcher_overlay<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    invoked_at: Point,
    bounds: Rect,
    snapshot: &ViewSnapshot,
    target: ContextMenuTarget,
    candidates: Vec<InsertNodeCandidate>,
    rows_mode: SearcherRowsMode,
) {
    let searcher = build_searcher_state(
        canvas,
        invoked_at,
        bounds,
        snapshot,
        target,
        candidates,
        canvas.interaction.recent_kinds.clone(),
        rows_mode,
    );
    install_searcher_overlay(canvas, searcher);
}
