use super::super::*;

pub(in super::super) fn apply_searcher_wheel_delta<M: NodeGraphCanvasMiddleware>(
    searcher: &mut SearcherState,
    delta_y: f32,
) -> bool {
    let n = searcher.rows.len();
    if n == 0 {
        return false;
    }

    let visible = super::super::SEARCHER_MAX_VISIBLE_ROWS.min(n);
    let max_scroll = n.saturating_sub(visible);
    let next_scroll = if delta_y > 0.0 {
        searcher.scroll.saturating_sub(1)
    } else if delta_y < 0.0 {
        (searcher.scroll + 1).min(max_scroll)
    } else {
        searcher.scroll
    };

    if next_scroll == searcher.scroll {
        return false;
    }

    searcher.scroll = next_scroll;
    NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    true
}

#[cfg(test)]
mod tests;
