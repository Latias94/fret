use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
    low_level_adapter::{CanvasHandledCx, CanvasPaintInvalidationCx, CanvasRedrawCx},
};
use super::super::SearcherInputCx;
use super::*;
use crate::core::{Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use fret_core::{KeyCode, Modifiers, Point, Px};
use fret_runtime::ModelStore;

struct StubHost;

#[derive(Default)]
struct StubCx {
    stopped: bool,
    redraws: usize,
    paint_invalidations: usize,
    activated_row: Option<usize>,
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

impl CanvasHandledCx<StubHost> for StubCx {
    fn stop_propagation(&mut self) {
        self.stopped = true;
    }
}

impl SearcherInputCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
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

fn row(label: &str, candidate_ix: usize, enabled: bool) -> SearcherRow {
    SearcherRow {
        kind: SearcherRowKind::Candidate { candidate_ix },
        label: std::sync::Arc::<str>::from(label),
        enabled,
    }
}

fn searcher_state() -> SearcherState {
    SearcherState {
        origin: Point::new(Px(0.0), Px(0.0)),
        invoked_at: Point::new(Px(0.0), Px(0.0)),
        target: ContextMenuTarget::Background,
        rows_mode: SearcherRowsMode::Flat,
        query: String::new(),
        candidates: vec![candidate("add"), candidate("mul")],
        recent_kinds: Vec::new(),
        rows: vec![row("add", 0, true), row("mul", 1, true)],
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
fn searcher_enter_activates_active_row_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();

    let handled = handle_searcher_key_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        KeyCode::Enter,
        Modifiers::default(),
    );

    assert!(handled);
    assert_eq!(cx.activated_row, Some(0));
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_arrow_down_steps_active_row_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();

    let handled = handle_searcher_key_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    assert!(handled);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().active_row, 1);
    assert_eq!(cx.activated_row, None);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_text_key_updates_query_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();

    let handled = handle_searcher_key_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        KeyCode::KeyM,
        Modifiers::default(),
    );

    assert!(handled);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().query, "m");
    assert_eq!(cx.activated_row, None);
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_ctrl_text_key_is_not_handled() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();
    let modifiers = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };

    let handled = handle_searcher_key_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        KeyCode::KeyM,
        modifiers,
    );

    assert!(!handled);
    assert_eq!(canvas.interaction.searcher.as_ref().unwrap().query, "");
    assert_eq!(cx.activated_row, None);
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}

#[test]
fn searcher_key_without_searcher_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled = handle_searcher_key_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    assert!(!handled);
    assert_eq!(cx.activated_row, None);
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}
