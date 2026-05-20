use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
};
use super::*;
use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use fret_core::{Point, PointerId, Px};
use fret_runtime::{ModelStore, TickId};

#[derive(Default)]
struct StubCx {
    pointer_id: Option<PointerId>,
    tick_id: TickId,
    captures: usize,
}

impl SearcherArmCx for StubCx {
    fn pointer_id(&self) -> Option<PointerId> {
        self.pointer_id
    }

    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn capture_pointer(&mut self) {
        self.captures += 1;
    }
}

fn candidate(label: &str, enabled: bool) -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new(format!("math.{label}")),
        label: std::sync::Arc::<str>::from(label),
        enabled,
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
        candidates: vec![candidate("add", true), candidate("disabled", false)],
        recent_kinds: Vec::new(),
        rows: vec![
            SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: 0 },
                label: std::sync::Arc::<str>::from("add"),
                enabled: true,
            },
            SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: 1 },
                label: std::sync::Arc::<str>::from("disabled"),
                enabled: false,
            },
        ],
        hovered_row: None,
        active_row: 0,
        scroll: 0,
    }
}

fn test_canvas() -> NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    let mut models = ModelStore::default();
    let graph = models.insert(Graph::new(GraphId::new()));
    let view = models.insert(NodeGraphViewState::default());
    let editor_config = models.insert(NodeGraphEditorConfig::default());
    NodeGraphCanvasWith::new_with_middleware(
        graph,
        view,
        editor_config,
        NoopNodeGraphCanvasMiddleware,
    )
}

#[test]
fn arm_searcher_row_drag_rejects_unselectable_row_without_side_effects() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx {
        pointer_id: Some(PointerId(7)),
        tick_id: TickId(11),
        captures: 0,
    };

    let armed = arm_searcher_row_drag(&mut canvas, &mut cx, 1, Point::new(Px(10.0), Px(20.0)));

    assert!(!armed);
    assert!(canvas.interaction.pending_insert_node_drag.is_none());
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().active_row, 0);
    assert_eq!(cx.captures, 0);
}

#[test]
fn arm_searcher_row_drag_records_pending_drag_and_captures_pointer() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx {
        pointer_id: Some(PointerId(7)),
        tick_id: TickId(11),
        captures: 0,
    };
    let start = Point::new(Px(10.0), Px(20.0));

    let armed = arm_searcher_row_drag(&mut canvas, &mut cx, 0, start);

    assert!(armed);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().active_row, 0);
    let pending = canvas
        .interaction
        .pending_insert_node_drag
        .as_ref()
        .expect("searcher arm should record pending insert-node drag");
    assert_eq!(pending.start_pos, start);
    assert_eq!(pending.pointer_id, PointerId(7));
    assert_eq!(pending.start_tick, TickId(11));
    assert_eq!(pending.candidate.label.as_ref(), "add");
    assert_eq!(cx.captures, 1);
}
