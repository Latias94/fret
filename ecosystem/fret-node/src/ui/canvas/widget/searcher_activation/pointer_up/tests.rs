use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
    searcher_activation::SearcherPointerHit,
    searcher_activation_state::SearcherReleaseCx,
    widget_tail::{
        PointerCaptureReleaseCx, WidgetHandledCx, WidgetPaintInvalidationCx, WidgetRedrawCx,
    },
};
use super::*;
use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use crate::ui::canvas::state::PendingInsertNodeDrag;
use fret_core::{MouseButton, Point, PointerId, Px};
use fret_runtime::{ModelStore, TickId};

struct StubHost;

#[derive(Default)]
struct StubCx {
    released: bool,
    stopped: bool,
    redraws: usize,
    paint_invalidations: usize,
    activated_row: Option<usize>,
}

impl WidgetRedrawCx<StubHost> for StubCx {
    fn request_redraw(&mut self) {
        self.redraws += 1;
    }
}

impl WidgetPaintInvalidationCx<StubHost> for StubCx {
    fn invalidate_paint(&mut self) {
        self.paint_invalidations += 1;
    }
}

impl WidgetHandledCx<StubHost> for StubCx {
    fn stop_propagation(&mut self) {
        self.stopped = true;
    }
}

impl PointerCaptureReleaseCx<StubHost> for StubCx {
    fn release_pointer_capture(&mut self) {
        self.released = true;
    }
}

impl SearcherReleaseCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
    fn try_activate_searcher_row(
        &mut self,
        _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        row_ix: usize,
    ) -> bool {
        self.activated_row = Some(row_ix);
        true
    }
}

fn candidate(label: &str) -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new(format!("math.{label}")),
        label: std::sync::Arc::<str>::from(label),
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
        candidates: vec![candidate("add")],
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

fn pending_drag() -> PendingInsertNodeDrag {
    PendingInsertNodeDrag {
        candidate: candidate("add"),
        start_pos: Point::new(Px(10.0), Px(20.0)),
        pointer_id: PointerId(7),
        start_tick: TickId(11),
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
fn searcher_pointer_up_ignores_non_left_button() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_up_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(10.0), Px(20.0)),
        MouseButton::Right,
        1.0,
    );

    assert!(!handled);
    assert!(canvas.interaction.searcher.is_some());
    assert!(canvas.interaction.pending_insert_node_drag.is_some());
    assert!(!cx.released);
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
    assert_eq!(cx.activated_row, None);
}

#[test]
fn searcher_pointer_up_without_searcher_clears_pending_drag_only() {
    let mut canvas = test_canvas();
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_up_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(10.0), Px(20.0)),
        MouseButton::Left,
        1.0,
    );

    assert!(!handled);
    assert!(canvas.interaction.searcher.is_none());
    assert!(canvas.interaction.pending_insert_node_drag.is_none());
    assert!(!cx.released);
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
    assert_eq!(cx.activated_row, None);
}

#[test]
fn searcher_pointer_up_on_row_activates_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_up_hit(
        &mut canvas,
        &mut cx,
        SearcherPointerHit {
            inside: true,
            row_ix: Some(0),
        },
    );

    assert!(handled);
    assert!(canvas.interaction.pending_insert_node_drag.is_none());
    assert!(cx.released);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
    assert_eq!(cx.activated_row, Some(0));
}

#[test]
fn searcher_pointer_up_outside_dismisses_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_up_hit(
        &mut canvas,
        &mut cx,
        SearcherPointerHit {
            inside: false,
            row_ix: None,
        },
    );

    assert!(handled);
    assert!(canvas.interaction.searcher.is_none());
    assert!(canvas.interaction.pending_insert_node_drag.is_none());
    assert!(cx.released);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
    assert_eq!(cx.activated_row, None);
}
