use super::super::super::{
    ContextMenuTarget, InsertNodeCandidate, NodeGraphCanvasWith, NoopNodeGraphCanvasMiddleware,
    SearcherRowsMode, SearcherState,
    low_level_adapter::{CanvasPaintInvalidationCx, CanvasRedrawCx},
};
use super::*;
use crate::core::{Graph, GraphId, NodeKindKey};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
use fret_core::{Point, Px};
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

fn searcher_state() -> SearcherState {
    SearcherState {
        origin: Point::new(Px(10.0), Px(20.0)),
        invoked_at: Point::new(Px(10.0), Px(20.0)),
        target: ContextMenuTarget::Background,
        rows_mode: SearcherRowsMode::Flat,
        query: String::new(),
        candidates: vec![candidate("add"), candidate("mul")],
        recent_kinds: Vec::new(),
        rows: vec![
            SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: 0 },
                label: std::sync::Arc::<str>::from("add"),
                enabled: true,
            },
            SearcherRow {
                kind: SearcherRowKind::Candidate { candidate_ix: 1 },
                label: std::sync::Arc::<str>::from("mul"),
                enabled: true,
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

fn row_position(
    canvas: &NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
    row_ix: usize,
) -> Point {
    let searcher = canvas
        .interaction
        .searcher
        .as_ref()
        .expect("searcher should be installed");
    let pad = canvas.style.paint.context_menu_padding;
    let item_h = canvas.style.paint.context_menu_item_height;
    Point::new(
        Px(searcher.origin.x.0 + pad + 1.0),
        Px(searcher.origin.y.0 + pad + item_h + pad + (row_ix as f32 + 0.5) * item_h),
    )
}

#[test]
fn searcher_pointer_move_without_searcher_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled = handle_searcher_pointer_move_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(0.0)),
        1.0,
    );

    assert!(!handled);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}

#[test]
fn searcher_pointer_move_updates_hover_and_invalidates_paint() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();
    let position = row_position(&canvas, 1);

    let handled =
        handle_searcher_pointer_move_event::<StubHost, _>(&mut canvas, &mut cx, position, 1.0);

    assert!(handled);
    let searcher = canvas.interaction.searcher.as_ref().unwrap();
    assert_eq!(searcher.hovered_row, Some(1));
    assert_eq!(searcher.active_row, 1);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn searcher_pointer_move_same_hover_does_not_invalidate_paint_again() {
    let mut canvas = test_canvas();
    canvas.interaction.searcher = Some(searcher_state());
    let mut cx = StubCx::default();
    let position = row_position(&canvas, 0);

    assert!(handle_searcher_pointer_move_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        position,
        1.0
    ));
    assert!(handle_searcher_pointer_move_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        position,
        1.0
    ));

    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}
