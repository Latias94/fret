use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
    low_level_adapter::{CanvasPaintInvalidationCx, CanvasRedrawCx},
};
use super::*;
use crate::core::{Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use fret_core::{Modifiers, Point, Px};
use fret_runtime::ModelStore;

struct StubHost;

#[derive(Default)]
struct StubCx {
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

fn candidate(label: &str) -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new(format!("math.{label}")),
        label: std::sync::Arc::<str>::from(label),
        enabled: true,
        template: None,
        payload: serde_json::Value::Null,
    }
}

fn row(ix: usize) -> SearcherRow {
    SearcherRow {
        kind: SearcherRowKind::Candidate { candidate_ix: ix },
        label: std::sync::Arc::<str>::from(format!("row-{ix}")),
        enabled: true,
    }
}

fn searcher_state_with_active(scroll: usize, active_row: usize) -> SearcherState {
    SearcherState {
        origin: Point::new(Px(10.0), Px(20.0)),
        invoked_at: Point::new(Px(10.0), Px(20.0)),
        target: ContextMenuTarget::Background,
        rows_mode: SearcherRowsMode::Flat,
        query: String::new(),
        candidates: vec![candidate("add")],
        recent_kinds: Vec::new(),
        rows: (0..20).map(row).collect(),
        hovered_row: None,
        active_row,
        scroll,
    }
}

fn searcher_state(scroll: usize) -> SearcherState {
    searcher_state_with_active(scroll, scroll)
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
fn searcher_wheel_without_searcher_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled = handle_searcher_wheel_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(-1.0)),
        Modifiers::default(),
    );

    assert!(!handled);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}

#[test]
fn searcher_wheel_scrolls_and_invalidates_paint() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state_with_active(0, 12));
    let mut cx = StubCx::default();

    let handled = handle_searcher_wheel_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(-1.0)),
        Modifiers::default(),
    );

    assert!(handled);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().scroll, 1);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_wheel_at_scroll_boundary_consumes_plain_wheel_without_paint() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state(0));
    let mut cx = StubCx::default();

    let handled = handle_searcher_wheel_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(1.0)),
        Modifiers::default(),
    );

    assert!(handled);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().scroll, 0);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}

#[test]
fn searcher_wheel_with_ctrl_does_not_consume_or_paint() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state(0));
    let mut cx = StubCx::default();
    let modifiers = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };

    let handled = handle_searcher_wheel_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(-1.0)),
        modifiers,
    );

    assert!(!handled);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().scroll, 0);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}
