use fret_core::{Modifiers, MouseButton, Point};

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
    searcher_activation::{self, SearcherPointerDownCx},
    searcher_activation_state::SearcherReleaseCx,
    searcher_input::{self, SearcherInputCx},
    searcher_pointer, searcher_ui,
};

pub(in super::super) trait SearcherCx<H, M: NodeGraphCanvasMiddleware>:
    SearcherPointerDownCx<H> + SearcherReleaseCx<H, M> + SearcherInputCx<H, M>
{
}

impl<H, M, T> SearcherCx<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: SearcherPointerDownCx<H> + SearcherReleaseCx<H, M> + SearcherInputCx<H, M>,
{
}

pub(super) fn handle_searcher_escape<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherCx<H, M>,
) -> bool {
    searcher_ui::handle_searcher_escape_event(canvas, cx)
}

pub(super) fn handle_searcher_key_down<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherCx<H, M>,
    key: fret_core::KeyCode,
    modifiers: Modifiers,
) -> bool {
    searcher_input::handle_searcher_key_down_event(canvas, cx, key, modifiers)
}

pub(super) fn handle_searcher_pointer_down<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    searcher_activation::handle_searcher_pointer_down_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_searcher_pointer_up<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    searcher_activation::handle_searcher_pointer_up_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_searcher_pointer_move<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherCx<H, M>,
    position: Point,
    zoom: f32,
) -> bool {
    searcher_pointer::handle_searcher_pointer_move_event(canvas, cx, position, zoom)
}

pub(super) fn handle_searcher_wheel<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherCx<H, M>,
    delta: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    searcher_pointer::handle_searcher_wheel_event(canvas, cx, delta, modifiers, zoom)
}

#[cfg(test)]
mod tests {
    use super::super::{
        ContextMenuTarget, InsertNodeCandidate, NoopNodeGraphCanvasMiddleware, SearcherRowsMode,
        SearcherState,
        searcher_activation_state::{SearcherArmCx, SearcherReleaseCx},
        searcher_input::SearcherInputCx,
        widget_tail::{
            PointerCaptureReleaseCx, WidgetHandledCx, WidgetPaintInvalidationCx, WidgetRedrawCx,
        },
    };
    use super::*;
    use crate::core::{CanvasPoint, Graph, GraphId, NodeKindKey};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::canvas::searcher::{SearcherRow, SearcherRowKind};
    use fret_core::{KeyCode, PointerId, Px};
    use fret_runtime::{ModelStore, TickId};

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        pointer_id: Option<PointerId>,
        tick_id: TickId,
        captures: usize,
        released: bool,
        stopped: bool,
        redraws: usize,
        paint_invalidations: usize,
        activated_by_key: Option<usize>,
        activated_by_release: Option<usize>,
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

    impl SearcherInputCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
        fn try_activate_searcher_row(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            row_ix: usize,
        ) -> bool {
            self.activated_by_key = Some(row_ix);
            true
        }
    }

    impl SearcherReleaseCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
        fn try_activate_searcher_row(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            row_ix: usize,
        ) -> bool {
            self.activated_by_release = Some(row_ix);
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
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(10.0), Px(20.0)),
            target: ContextMenuTarget::BackgroundInsertNodePicker {
                at: CanvasPoint::default(),
            },
            rows_mode: SearcherRowsMode::Catalog,
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
    fn searcher_top_level_escape_dismisses_and_finishes() {
        let mut canvas = test_canvas();
        canvas.interaction.searcher = Some(searcher_state());
        let mut cx = StubCx::default();

        let handled = handle_searcher_escape::<StubHost, _>(&mut canvas, &mut cx);

        assert!(handled);
        assert!(canvas.interaction.searcher.is_none());
        assert!(cx.released);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn searcher_top_level_key_down_delegates_to_activation_seam() {
        let mut canvas = test_canvas();
        canvas.interaction.searcher = Some(searcher_state());
        let mut cx = StubCx::default();

        let handled = handle_searcher_key_down::<StubHost, _>(
            &mut canvas,
            &mut cx,
            KeyCode::Enter,
            Modifiers::default(),
        );

        assert!(handled);
        assert_eq!(cx.activated_by_key, Some(0));
        assert_eq!(cx.activated_by_release, None);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn searcher_top_level_pointer_down_arms_row_drag_without_retained_cx() {
        let mut canvas = test_canvas();
        canvas.interaction.searcher = Some(searcher_state());
        let mut cx = StubCx {
            pointer_id: Some(PointerId(7)),
            tick_id: TickId(11),
            ..StubCx::default()
        };
        let start = row_position(&canvas, 0);

        let handled = handle_searcher_pointer_down::<StubHost, _>(
            &mut canvas,
            &mut cx,
            start,
            MouseButton::Left,
            1.0,
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
}
