use fret_core::Modifiers;
use fret_ui::UiHost;

use super::searcher_ui::invalidate_searcher_paint;
use super::*;
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};

fn is_selectable_searcher_row(row: &SearcherRow) -> bool {
    matches!(row.kind, SearcherRowKind::Candidate { .. }) && row.enabled
}

pub(super) fn sync_searcher_hovered_row<M: NodeGraphCanvasMiddleware>(
    searcher: &mut SearcherState,
    hovered_row: Option<usize>,
) -> bool {
    if searcher.hovered_row == hovered_row {
        return false;
    }

    searcher.hovered_row = hovered_row;
    if let Some(ix) = hovered_row
        && searcher
            .rows
            .get(ix)
            .is_some_and(is_selectable_searcher_row)
    {
        searcher.active_row = ix;
        NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    }
    true
}

pub(super) fn apply_searcher_wheel_delta<M: NodeGraphCanvasMiddleware>(
    searcher: &mut SearcherState,
    delta_y: f32,
) -> bool {
    let n = searcher.rows.len();
    if n == 0 {
        return false;
    }

    let visible = super::SEARCHER_MAX_VISIBLE_ROWS.min(n);
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

pub(super) fn handle_searcher_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    if canvas.update_searcher_hover_from_position(position, zoom) {
        invalidate_searcher_paint(cx);
    }
    true
}

pub(super) fn handle_searcher_wheel_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    delta: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    if canvas.scroll_searcher_from_wheel(delta, modifiers) {
        invalidate_searcher_paint(cx);
        return true;
    }

    !modifiers.ctrl && !modifiers.meta
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn update_searcher_hover_from_position(
        &mut self,
        position: Point,
        zoom: f32,
    ) -> bool {
        let Some(searcher) = self.interaction.searcher.as_mut() else {
            return false;
        };
        let hovered_row = super::hit_searcher_row(&self.style, searcher, position, zoom);
        sync_searcher_hovered_row::<M>(searcher, hovered_row)
    }

    pub(super) fn scroll_searcher_from_wheel(
        &mut self,
        delta: Point,
        modifiers: Modifiers,
    ) -> bool {
        if modifiers.ctrl || modifiers.meta {
            return false;
        }

        let Some(searcher) = self.interaction.searcher.as_mut() else {
            return false;
        };
        apply_searcher_wheel_delta::<M>(searcher, delta.y.0)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_core::{Point, Px};

    use super::*;
    use crate::core::{CanvasPoint, NodeKindKey};

    fn header(label: &str) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Header,
            label: Arc::<str>::from(label),
            enabled: false,
        }
    }

    fn candidate(label: &str, enabled: bool, candidate_ix: usize) -> SearcherRow {
        SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix },
            label: Arc::<str>::from(label),
            enabled,
        }
    }

    fn candidate_item(kind: &str, label: &str) -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new(kind),
            label: Arc::<str>::from(label),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }
    }

    fn searcher_state(rows: Vec<SearcherRow>, active_row: usize, scroll: usize) -> SearcherState {
        SearcherState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(0.0), Px(0.0)),
            target: ContextMenuTarget::BackgroundInsertNodePicker {
                at: CanvasPoint::default(),
            },
            rows_mode: SearcherRowsMode::Catalog,
            query: String::new(),
            candidates: vec![candidate_item("math.add", "Math/Add")],
            recent_kinds: Vec::new(),
            rows,
            hovered_row: None,
            active_row,
            scroll,
        }
    }

    #[test]
    fn sync_searcher_hovered_row_promotes_selectable_hit_to_active_row() {
        let mut searcher = searcher_state(vec![header("Recent"), candidate("Add", true, 0)], 0, 0);

        assert!(sync_searcher_hovered_row::<NoopNodeGraphCanvasMiddleware>(
            &mut searcher,
            Some(1),
        ));
        assert_eq!(searcher.hovered_row, Some(1));
        assert_eq!(searcher.active_row, 1);
    }

    #[test]
    fn sync_searcher_hovered_row_keeps_active_row_for_non_selectable_hit() {
        let mut searcher = searcher_state(vec![header("Recent"), candidate("Add", true, 0)], 1, 0);

        assert!(sync_searcher_hovered_row::<NoopNodeGraphCanvasMiddleware>(
            &mut searcher,
            Some(0),
        ));
        assert_eq!(searcher.hovered_row, Some(0));
        assert_eq!(searcher.active_row, 1);
    }

    #[test]
    fn apply_searcher_wheel_delta_clamps_scroll_range() {
        let rows = (0..20)
            .map(|ix| candidate(&format!("Item {ix}"), true, 0))
            .collect();
        let mut searcher = searcher_state(rows, SEARCHER_MAX_VISIBLE_ROWS, 0);

        assert!(apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(
            &mut searcher,
            -1.0,
        ));
        assert_eq!(searcher.scroll, 1);

        for _ in 0..50 {
            let _ =
                apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(&mut searcher, -1.0);
        }
        assert_eq!(
            searcher.scroll,
            searcher
                .rows
                .len()
                .saturating_sub(SEARCHER_MAX_VISIBLE_ROWS)
        );

        for _ in 0..50 {
            let _ = apply_searcher_wheel_delta::<NoopNodeGraphCanvasMiddleware>(&mut searcher, 1.0);
        }
        assert_eq!(searcher.scroll, 1);
    }
}
