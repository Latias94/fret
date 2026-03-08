use super::searcher_activation::SearcherPointerHit;
use super::*;

pub(super) fn searcher_pointer_hit<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    position: Point,
    zoom: f32,
) -> SearcherPointerHit {
    let Some(searcher) = canvas.interaction.searcher.as_ref() else {
        return SearcherPointerHit::default();
    };

    let visible = super::searcher_visible_rows(searcher);
    let rect = super::searcher_rect_at(&canvas.style, searcher.origin, visible, zoom);
    SearcherPointerHit {
        inside: rect.contains(position),
        row_ix: super::hit_searcher_row(&canvas.style, searcher, position, zoom),
    }
}

pub(super) fn searcher_candidate_for_row(
    searcher: &SearcherState,
    row_ix: usize,
) -> Option<InsertNodeCandidate> {
    let row = searcher.rows.get(row_ix)?;
    if !row.enabled {
        return None;
    }
    let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
        return None;
    };
    searcher.candidates.get(candidate_ix).cloned()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_core::{Point, Px};

    use super::*;
    use crate::core::{CanvasPoint, NodeKindKey};
    use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};

    fn candidate(kind: &str, label: &str) -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new(kind),
            label: Arc::<str>::from(label),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        }
    }

    fn searcher_state(
        rows: Vec<SearcherRow>,
        candidates: Vec<InsertNodeCandidate>,
    ) -> SearcherState {
        SearcherState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(0.0), Px(0.0)),
            target: ContextMenuTarget::BackgroundInsertNodePicker {
                at: CanvasPoint::default(),
            },
            rows_mode: SearcherRowsMode::Catalog,
            query: String::new(),
            candidates,
            recent_kinds: Vec::new(),
            rows,
            hovered_row: None,
            active_row: 0,
            scroll: 0,
        }
    }

    #[test]
    fn searcher_candidate_for_row_returns_candidate_for_enabled_candidate_row() {
        let searcher = searcher_state(
            vec![SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: 0 },
                label: Arc::<str>::from("Add"),
                enabled: true,
            }],
            vec![candidate("math.add", "Add")],
        );

        assert_eq!(
            searcher_candidate_for_row(&searcher, 0).map(|candidate| candidate.kind),
            Some(NodeKindKey::new("math.add"))
        );
    }

    #[test]
    fn searcher_candidate_for_row_rejects_headers_and_disabled_rows() {
        let searcher = searcher_state(
            vec![
                SearcherRow {
                    kind: SearcherRowKind::Header,
                    label: Arc::<str>::from("Recent"),
                    enabled: false,
                },
                SearcherRow {
                    kind: SearcherRowKind::Candidate { candidate_ix: 0 },
                    label: Arc::<str>::from("Disabled"),
                    enabled: false,
                },
            ],
            vec![candidate("math.add", "Add")],
        );

        assert!(searcher_candidate_for_row(&searcher, 0).is_none());
        assert!(searcher_candidate_for_row(&searcher, 1).is_none());
    }
}
