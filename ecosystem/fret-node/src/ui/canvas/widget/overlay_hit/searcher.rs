use super::super::*;

pub(in super::super) fn searcher_visible_rows(searcher: &SearcherState) -> usize {
    searcher
        .rows
        .len()
        .saturating_sub(searcher.scroll)
        .min(SEARCHER_MAX_VISIBLE_ROWS)
}

pub(in super::super) fn searcher_rect_at(
    style: &NodeGraphStyle,
    origin: Point,
    row_count: usize,
    zoom: f32,
) -> Rect {
    Rect::new(origin, searcher_size_at_zoom(style, row_count, zoom))
}

pub(in super::super) fn searcher_size_at_zoom(
    style: &NodeGraphStyle,
    row_count: usize,
    zoom: f32,
) -> Size {
    let w = style.paint.context_menu_width / zoom;
    let item_h = style.paint.context_menu_item_height / zoom;
    let pad = style.paint.context_menu_padding / zoom;

    let list_rows = row_count.max(1) as f32;
    let h = 3.0 * pad + item_h * (1.0 + list_rows);
    Size::new(Px(w), Px(h))
}

pub(in super::super) fn hit_searcher_row(
    style: &NodeGraphStyle,
    searcher: &SearcherState,
    pos: Point,
    zoom: f32,
) -> Option<usize> {
    let visible = searcher_visible_rows(searcher);
    let rect = searcher_rect_at(style, searcher.origin, visible, zoom);
    if !rect.contains(pos) {
        return None;
    }

    let pad = style.paint.context_menu_padding / zoom;
    let item_h = style.paint.context_menu_item_height / zoom;

    let list_top = rect.origin.y.0 + pad + item_h + pad;
    let y = pos.y.0 - list_top;
    if y < 0.0 {
        return None;
    }

    let slot = (y / item_h).floor() as isize;
    if slot < 0 {
        return None;
    }
    let slot = slot as usize;
    if slot >= visible {
        return None;
    }

    let row_ix = searcher.scroll + slot;
    (row_ix < searcher.rows.len()).then_some(row_ix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(label: &str) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: 0 },
            label: std::sync::Arc::<str>::from(label),
            enabled: true,
        }
    }

    fn test_searcher(row_count: usize, scroll: usize) -> SearcherState {
        SearcherState {
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(10.0), Px(20.0)),
            target: ContextMenuTarget::Background,
            rows_mode: SearcherRowsMode::Flat,
            query: String::new(),
            candidates: Vec::new(),
            recent_kinds: Vec::new(),
            rows: (0..row_count).map(|ix| row(&format!("Row {ix}"))).collect(),
            hovered_row: None,
            active_row: 0,
            scroll,
        }
    }

    #[test]
    fn searcher_visible_rows_caps_by_scroll_and_max_visible() {
        let searcher = test_searcher(20, 3);
        assert_eq!(searcher_visible_rows(&searcher), SEARCHER_MAX_VISIBLE_ROWS);
    }

    #[test]
    fn hit_searcher_row_includes_scroll_offset() {
        let style = NodeGraphStyle::default();
        let searcher = test_searcher(5, 1);
        let hit = hit_searcher_row(&style, &searcher, Point::new(Px(15.0), Px(71.0)), 1.0);
        assert_eq!(hit, Some(1));
    }
}
