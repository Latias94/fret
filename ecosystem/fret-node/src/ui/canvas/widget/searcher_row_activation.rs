use super::*;

pub(super) fn try_activate_searcher_row<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    row_ix: usize,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.take() else {
        return false;
    };

    let Some(row) = searcher.rows.get(row_ix).cloned() else {
        canvas.interaction.searcher = Some(searcher);
        return false;
    };
    let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
        canvas.interaction.searcher = Some(searcher);
        return false;
    };
    if !row.enabled {
        canvas.interaction.searcher = Some(searcher);
        return false;
    }

    let item = NodeGraphContextMenuItem {
        label: row.label,
        enabled: true,
        action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
    };
    canvas.activate_context_menu_item(
        cx,
        &searcher.target,
        searcher.invoked_at,
        item,
        &searcher.candidates,
    );
    true
}
