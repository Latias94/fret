use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
    searcher_activation::SearcherPointerHit,
    widget_tail::{
        PointerCaptureReleaseCx, WidgetHandledCx, WidgetPaintInvalidationCx, WidgetRedrawCx,
    },
};
use super::*;
use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use crate::ui::canvas::state::PendingInsertNodeDrag;
use fret_core::{Point, PointerId, Px};
use fret_runtime::{ModelStore, TickId};

struct StubHost;

#[derive(Default)]
struct StubCx {
    stopped: bool,
    released: bool,
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
fn searcher_release_without_pending_drag_is_side_effect_free() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();

    let handled = finish_searcher_row_drag_release(
        &mut canvas,
        &mut cx,
        SearcherPointerHit {
            inside: true,
            row_ix: Some(0),
        },
    );

    assert!(!handled);
    assert!(canvas.interaction.searcher.is_some());
    assert!(!cx.released);
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
    assert_eq!(cx.activated_row, None);
}

#[test]
fn searcher_release_on_row_activates_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    let handled = finish_searcher_row_drag_release(
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
fn searcher_release_outside_dismisses_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    let handled = finish_searcher_row_drag_release(
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
