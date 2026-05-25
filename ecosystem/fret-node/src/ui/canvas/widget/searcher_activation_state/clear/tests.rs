use super::super::super::low_level_adapter::{
    CanvasPaintInvalidationCx, CanvasPointerCaptureReleaseCx, CanvasRedrawCx,
};
use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
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
    released: bool,
    redraws: usize,
    paint_invalidations: usize,
}

impl CanvasRedrawCx<StubHost> for StubCx {
    fn request_redraw(&mut self) {
        self.redraws += 1;
    }
}

impl CanvasPaintInvalidationCx<StubHost> for StubCx {
    fn invalidate_paint(&mut self) {
        self.paint_invalidations += 1;
    }
}

impl CanvasPointerCaptureReleaseCx<StubHost> for StubCx {
    fn release_pointer_capture(&mut self) {
        self.released = true;
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

#[test]
fn dismiss_searcher_overlay_clears_state_and_releases_capture_without_painting() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    canvas.interaction.pending_insert_node_drag = Some(pending_drag());
    let mut cx = StubCx::default();

    dismiss_searcher_overlay::<StubHost, NoopNodeGraphCanvasMiddleware>(&mut canvas, &mut cx);

    assert!(canvas.interaction.searcher.is_none());
    assert!(canvas.interaction.pending_insert_node_drag.is_none());
    assert!(cx.released);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}
