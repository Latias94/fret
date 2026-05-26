use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
    low_level_adapter::{
        CanvasHandledCx, CanvasPaintInvalidationCx, CanvasPointerCaptureReleaseCx, CanvasRedrawCx,
    },
    searcher_activation::SearcherPointerHit,
    searcher_activation_state::SearcherArmCx,
};
use super::*;
use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use fret_core::{MouseButton, Point, PointerId, Px};
use fret_runtime::{ModelStore, TickId};

#[derive(Default)]
struct StubCx {
    pointer_id: Option<PointerId>,
    tick_id: TickId,
    captures: usize,
    released: bool,
    stopped: bool,
    redraws: usize,
    paint_invalidations: usize,
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

impl CanvasRedrawCx<()> for StubCx {
    fn request_redraw(&mut self) {
        self.redraws += 1;
    }
}

impl CanvasPaintInvalidationCx<()> for StubCx {
    fn invalidate_paint(&mut self) {
        self.paint_invalidations += 1;
    }
}

impl CanvasHandledCx<()> for StubCx {
    fn stop_propagation(&mut self) {
        self.stopped = true;
    }
}

impl CanvasPointerCaptureReleaseCx<()> for StubCx {
    fn release_pointer_capture(&mut self) {
        self.released = true;
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
        candidates: vec![candidate("add", true)],
        recent_kinds: Vec::new(),
        rows: vec![SearcherRow {
            kind: SearcherRowKind::Candidate { candidate_ix: 0 },
            label: std::sync::Arc::<str>::from("add"),
            enabled: true,
        }],
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
fn searcher_pointer_down_without_searcher_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_down_event(
        &mut canvas,
        &mut cx,
        Point::new(Px(1.0), Px(1.0)),
        MouseButton::Left,
        1.0,
    );

    assert!(!handled);
    assert!(!cx.stopped);
    assert!(!cx.released);
    assert_eq!(cx.captures, 0);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}

#[test]
fn searcher_left_pointer_down_on_row_arms_drag_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx {
        pointer_id: Some(PointerId(7)),
        tick_id: TickId(11),
        ..StubCx::default()
    };
    let start = Point::new(Px(10.0), Px(20.0));

    let handled = handle_searcher_pointer_down_hit(
        &mut canvas,
        &mut cx,
        start,
        MouseButton::Left,
        SearcherPointerHit {
            inside: true,
            row_ix: Some(0),
        },
    );

    assert!(handled);
    let pending = canvas
        .interaction
        .pending_insert_node_drag
        .as_ref()
        .expect("row pointer-down should arm pending insert-node drag");
    assert_eq!(pending.start_pos, start);
    assert_eq!(pending.pointer_id, PointerId(7));
    assert_eq!(pending.start_tick, TickId(11));
    assert_eq!(pending.candidate.label.as_ref(), "add");
    assert_eq!(cx.captures, 1);
    assert!(!cx.released);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_left_pointer_down_outside_dismisses_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_down_hit(
        &mut canvas,
        &mut cx,
        Point::new(Px(10.0), Px(20.0)),
        MouseButton::Left,
        SearcherPointerHit {
            inside: false,
            row_ix: None,
        },
    );

    assert!(handled);
    assert!(canvas.interaction.searcher.is_none());
    assert_eq!(cx.captures, 0);
    assert!(cx.released);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_secondary_pointer_down_dismisses_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_down_hit(
        &mut canvas,
        &mut cx,
        Point::new(Px(10.0), Px(20.0)),
        MouseButton::Right,
        SearcherPointerHit {
            inside: true,
            row_ix: Some(0),
        },
    );

    assert!(handled);
    assert!(canvas.interaction.searcher.is_none());
    assert_eq!(cx.captures, 0);
    assert!(cx.released);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}
