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

pub(super) fn update_searcher_hover_from_position<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    let hovered_row = super::hit_searcher_row(&canvas.style, searcher, position, zoom);
    sync_searcher_hovered_row::<M>(searcher, hovered_row)
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
}
