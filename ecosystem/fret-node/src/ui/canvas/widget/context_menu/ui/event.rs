use super::super::*;
use super::{ContextMenuHoverEdgePolicy, overlay};
use crate::ui::canvas::widget::widget_tail::{WidgetHandledCx, WidgetPaintInvalidationCx};

pub(in crate::ui::canvas::widget::context_menu) trait ContextMenuFocusCx<H>:
    WidgetHandledCx<H>
{
    fn request_context_menu_focus(&mut self);
}

pub(super) fn invalidate_context_menu_paint<H>(cx: &mut impl WidgetPaintInvalidationCx<H>) {
    super::super::widget_tail::invalidate_widget_paint(cx);
}

pub(super) fn finish_context_menu_event<H>(cx: &mut impl WidgetHandledCx<H>) -> bool {
    super::super::widget_tail::finish_widget_handled(cx);
    true
}

pub(super) fn open_context_menu_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuFocusCx<H>,
    menu: ContextMenuState,
    hover_edge_policy: ContextMenuHoverEdgePolicy,
) -> bool {
    overlay::apply_context_menu_open_state(&mut canvas.interaction, menu, hover_edge_policy);
    cx.request_context_menu_focus();
    finish_context_menu_event(cx)
}

pub(super) fn restore_context_menu_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
    menu: ContextMenuState,
) -> bool {
    overlay::restore_context_menu(&mut canvas.interaction, menu);
    finish_context_menu_event(cx)
}

pub(super) fn dismiss_context_menu_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
) -> bool {
    if !overlay::clear_context_menu(&mut canvas.interaction) {
        return false;
    }

    finish_context_menu_event(cx)
}

pub(super) fn handle_context_menu_escape_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetHandledCx<H>,
) -> bool {
    dismiss_context_menu_event(canvas, cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Graph, GraphId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::canvas::widget::{
        NoopNodeGraphCanvasMiddleware,
        widget_tail::{WidgetHandledCx, WidgetRedrawCx},
    };
    use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};
    use fret_core::{Point, Px};
    use fret_runtime::ModelStore;
    use std::sync::Arc;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        focus_requests: usize,
        stopped: bool,
        redraws: usize,
        paint_invalidations: usize,
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

    impl ContextMenuFocusCx<StubHost> for StubCx {
        fn request_context_menu_focus(&mut self) {
            self.focus_requests += 1;
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

    fn test_menu(label: &str) -> ContextMenuState {
        ContextMenuState {
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(10.0), Px(20.0)),
            target: ContextMenuTarget::Background,
            items: vec![NodeGraphContextMenuItem {
                label: Arc::<str>::from(label),
                enabled: true,
                action: NodeGraphContextMenuAction::Custom(1),
            }],
            candidates: Vec::new(),
            hovered_item: None,
            active_item: 0,
            typeahead: String::new(),
        }
    }

    #[test]
    fn open_context_menu_event_installs_menu_focuses_and_finishes() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();

        let handled = open_context_menu_event::<StubHost, _>(
            &mut canvas,
            &mut cx,
            test_menu("Open"),
            ContextMenuHoverEdgePolicy::Clear,
        );

        assert!(handled);
        assert_eq!(
            canvas
                .interaction
                .context_menu
                .as_ref()
                .expect("menu should be installed")
                .items[0]
                .label
                .as_ref(),
            "Open"
        );
        assert_eq!(cx.focus_requests, 1);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn restore_context_menu_event_restores_menu_and_finishes_without_focus() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();

        let handled =
            restore_context_menu_event::<StubHost, _>(&mut canvas, &mut cx, test_menu("Restore"));

        assert!(handled);
        assert_eq!(
            canvas
                .interaction
                .context_menu
                .as_ref()
                .expect("menu should be restored")
                .items[0]
                .label
                .as_ref(),
            "Restore"
        );
        assert_eq!(cx.focus_requests, 0);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn dismiss_context_menu_event_clears_menu_and_finishes() {
        let mut canvas = test_canvas();
        canvas.interaction.context_menu = Some(test_menu("Dismiss"));
        let mut cx = StubCx::default();

        let handled = dismiss_context_menu_event::<StubHost, _>(&mut canvas, &mut cx);

        assert!(handled);
        assert!(canvas.interaction.context_menu.is_none());
        assert_eq!(cx.focus_requests, 0);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn dismiss_context_menu_event_without_menu_is_side_effect_free() {
        let mut canvas = test_canvas();
        let mut cx = StubCx::default();

        let handled = dismiss_context_menu_event::<StubHost, _>(&mut canvas, &mut cx);

        assert!(!handled);
        assert_eq!(cx.focus_requests, 0);
        assert!(!cx.stopped);
        assert_eq!(cx.redraws, 0);
        assert_eq!(cx.paint_invalidations, 0);
    }

    #[test]
    fn invalidate_context_menu_paint_requests_redraw_and_paint_invalidation() {
        let mut cx = StubCx::default();

        invalidate_context_menu_paint::<StubHost>(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.stopped);
    }
}
