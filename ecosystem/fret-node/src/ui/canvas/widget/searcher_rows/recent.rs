use super::super::*;

pub(super) fn record_recent_kind<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    kind: &NodeKindKey,
    max_recent_searcher_kinds: usize,
) {
    canvas.interaction.recent_kinds.retain(|item| item != kind);
    canvas.interaction.recent_kinds.insert(0, kind.clone());
    if canvas.interaction.recent_kinds.len() > max_recent_searcher_kinds {
        canvas
            .interaction
            .recent_kinds
            .truncate(max_recent_searcher_kinds);
    }
}
