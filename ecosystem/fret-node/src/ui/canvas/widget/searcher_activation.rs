use fret_core::PointerId;

use super::*;
use crate::ui::canvas::state::PendingInsertNodeDrag;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SearcherPointerHit {
    pub(super) inside: bool,
    pub(super) row_ix: Option<usize>,
}

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

fn searcher_candidate_for_row(
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

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn sync_searcher_active_row_if_selectable(&mut self, row_ix: usize) -> bool {
        let Some(searcher) = self.interaction.searcher.as_mut() else {
            return false;
        };
        if !searcher
            .rows
            .get(row_ix)
            .is_some_and(Self::searcher_is_selectable_row)
        {
            return false;
        }

        searcher.active_row = row_ix;
        Self::ensure_searcher_active_visible(searcher);
        true
    }

    pub(super) fn arm_searcher_row_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        row_ix: usize,
        position: Point,
    ) -> bool {
        if !self.sync_searcher_active_row_if_selectable(row_ix) {
            return false;
        }

        let Some(candidate) = self
            .interaction
            .searcher
            .as_ref()
            .and_then(|searcher| searcher_candidate_for_row(searcher, row_ix))
        else {
            return false;
        };

        self.interaction.pending_insert_node_drag = Some(PendingInsertNodeDrag {
            candidate,
            start_pos: position,
            pointer_id: cx.pointer_id.unwrap_or(PointerId(0)),
            start_tick: cx.app.tick_id(),
        });
        cx.capture_pointer(cx.node);
        true
    }

    pub(super) fn activate_searcher_hit_or_dismiss<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        hit: SearcherPointerHit,
    ) {
        if let Some(row_ix) = hit.row_ix {
            let _ = self.try_activate_searcher_row(cx, row_ix);
        } else if !hit.inside {
            self.interaction.searcher = None;
        }
    }
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
            vec![candidate("math.add", "Math/Add")],
        );

        let candidate = searcher_candidate_for_row(&searcher, 0).expect("expected candidate row");
        assert_eq!(candidate.kind, NodeKindKey::new("math.add"));
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
                    label: Arc::<str>::from("Add"),
                    enabled: false,
                },
            ],
            vec![candidate("math.add", "Math/Add")],
        );

        assert!(searcher_candidate_for_row(&searcher, 0).is_none());
        assert!(searcher_candidate_for_row(&searcher, 1).is_none());
        assert!(searcher_candidate_for_row(&searcher, 99).is_none());
    }
}
