use super::*;
use crate::core::{CanvasPoint, NodeKindKey};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use crate::ui::canvas::state::PendingInsertNodeDrag;
use fret_core::{Point, PointerId, Px};
use fret_runtime::TickId;

fn candidate() -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new("math.add"),
        label: std::sync::Arc::<str>::from("Add"),
        enabled: true,
        template: None,
        payload: serde_json::Value::Null,
    }
}

fn searcher_state() -> SearcherState {
    SearcherState {
        origin: Point::new(Px(0.0), Px(0.0)),
        invoked_at: Point::new(Px(0.0), Px(0.0)),
        target: ContextMenuTarget::BackgroundInsertNodePicker {
            at: CanvasPoint::default(),
        },
        rows_mode: SearcherRowsMode::Catalog,
        query: String::new(),
        candidates: vec![candidate()],
        recent_kinds: Vec::new(),
        rows: vec![SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: 0 },
            label: std::sync::Arc::<str>::from("Add"),
            enabled: true,
        }],
        hovered_row: None,
        active_row: 0,
        scroll: 0,
    }
}

fn pending_drag() -> PendingInsertNodeDrag {
    PendingInsertNodeDrag {
        candidate: candidate(),
        start_pos: Point::new(Px(10.0), Px(20.0)),
        pointer_id: PointerId(7),
        start_tick: TickId(0),
    }
}

#[test]
fn clear_pending_searcher_row_drag_reports_and_clears_state() {
    let mut interaction = crate::ui::canvas::state::InteractionState::default();
    interaction.pending_insert_node_drag = Some(pending_drag());

    assert!(clear_pending_searcher_row_drag(&mut interaction));
    assert!(interaction.pending_insert_node_drag.is_none());
    assert!(!clear_pending_searcher_row_drag(&mut interaction));
}

#[test]
fn clear_searcher_overlay_clears_searcher_and_pending_drag() {
    let mut interaction = crate::ui::canvas::state::InteractionState::default();
    interaction.searcher = Some(searcher_state());
    interaction.pending_insert_node_drag = Some(pending_drag());

    assert!(clear_searcher_overlay(&mut interaction));
    assert!(interaction.searcher.is_none());
    assert!(interaction.pending_insert_node_drag.is_none());
}
