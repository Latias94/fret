use super::*;

mod activate;
mod background_execution;
mod connection_execution;
mod connection_execution_conversion;
mod connection_execution_insert;
mod edge_execution;
mod input;
pub(in crate::ui::canvas::widget) mod item_builders;
mod key_navigation;
pub(in crate::ui::canvas::widget) mod opening;
mod pointer;
mod selection_activation;
mod target_hit;
mod target_selection;
mod ui;

pub(super) use ui::ContextMenuHoverEdgePolicy;

pub(in crate::ui::canvas::widget) trait ContextMenuCx<H, M: NodeGraphCanvasMiddleware>:
    key_navigation::ContextMenuKeyDownCx<H, M> + selection_activation::ContextMenuPointerDownCx<H, M>
{
}

impl<H, M, T> ContextMenuCx<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: key_navigation::ContextMenuKeyDownCx<H, M>
        + selection_activation::ContextMenuPointerDownCx<H, M>,
{
}

pub(super) fn apply_context_menu_open_state(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
    hover_edge_policy: ContextMenuHoverEdgePolicy,
) {
    ui::apply_context_menu_open_state(interaction, menu, hover_edge_policy);
}

pub(super) fn clear_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    ui::clear_context_menu(interaction)
}

pub(super) fn take_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> Option<ContextMenuState> {
    ui::take_context_menu(interaction)
}

pub(super) fn restore_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
) {
    ui::restore_context_menu(interaction, menu);
}

pub(super) fn handle_context_menu_escape<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuCx<H, M>,
) -> bool {
    input::handle_context_menu_escape(canvas, cx)
}

pub(super) fn handle_context_menu_key_down<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuCx<H, M>,
    key: fret_core::KeyCode,
) -> bool {
    input::handle_context_menu_key_down(canvas, cx, key)
}

pub(super) fn handle_context_menu_pointer_down<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    pointer::handle_context_menu_pointer_down(canvas, cx, position, button, zoom)
}

pub(super) fn handle_context_menu_pointer_move<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl ContextMenuCx<H, M>,
    position: Point,
    zoom: f32,
) -> bool {
    pointer::handle_context_menu_pointer_move(canvas, cx, position, zoom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Graph, GraphId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::canvas::widget::widget_tail::{
        WidgetHandledCx, WidgetPaintInvalidationCx, WidgetRedrawCx,
    };
    use fret_core::{KeyCode, Px};
    use fret_runtime::{CommandId, ModelStore};
    use selection_activation::ContextMenuSelectionActivationCx;
    use std::sync::Arc;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        stopped: bool,
        redraws: usize,
        paint_invalidations: usize,
        activation_calls: usize,
        activated_item_label: Option<String>,
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

    impl ContextMenuSelectionActivationCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
        fn activate_context_menu_item(
            &mut self,
            _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
            _target: &ContextMenuTarget,
            _invoked_at: Point,
            item: NodeGraphContextMenuItem,
            _menu_candidates: &[InsertNodeCandidate],
        ) {
            self.activation_calls += 1;
            self.activated_item_label = Some(item.label.to_string());
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

    fn menu(items: Vec<NodeGraphContextMenuItem>, active_item: usize) -> ContextMenuState {
        ContextMenuState {
            origin: Point::new(Px(0.0), Px(0.0)),
            invoked_at: Point::new(Px(4.0), Px(8.0)),
            target: ContextMenuTarget::Background,
            items,
            candidates: Vec::new(),
            hovered_item: None,
            active_item,
            typeahead: String::new(),
        }
    }

    fn item(label: &str, enabled: bool) -> NodeGraphContextMenuItem {
        NodeGraphContextMenuItem {
            label: Arc::<str>::from(label),
            enabled,
            action: NodeGraphContextMenuAction::Command(CommandId::from("demo.command")),
        }
    }

    fn row_position(
        canvas: &NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        row_ix: usize,
    ) -> Point {
        let menu = canvas
            .interaction
            .context_menu
            .as_ref()
            .expect("context menu should be installed");
        let pad = canvas.style.paint.context_menu_padding;
        let item_h = canvas.style.paint.context_menu_item_height;
        Point::new(
            Px(menu.origin.x.0 + pad + 1.0),
            Px(menu.origin.y.0 + pad + (row_ix as f32 + 0.5) * item_h),
        )
    }

    fn assert_finished(cx: &StubCx) {
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn context_menu_top_level_escape_dismisses_and_finishes() {
        let mut canvas = test_canvas();
        canvas.interaction.context_menu = Some(menu(vec![item("Alpha", true)], 0));
        let mut cx = StubCx::default();

        let handled = handle_context_menu_escape::<StubHost, _>(&mut canvas, &mut cx);

        assert!(handled);
        assert!(canvas.interaction.context_menu.is_none());
        assert_eq!(cx.activation_calls, 0);
        assert_finished(&cx);
    }

    #[test]
    fn context_menu_top_level_key_down_delegates_to_activation_seam() {
        let mut canvas = test_canvas();
        canvas.interaction.context_menu =
            Some(menu(vec![item("Alpha", true), item("Beta", true)], 1));
        let mut cx = StubCx::default();

        let handled =
            handle_context_menu_key_down::<StubHost, _>(&mut canvas, &mut cx, KeyCode::Enter);

        assert!(handled);
        assert!(canvas.interaction.context_menu.is_none());
        assert_eq!(cx.activation_calls, 1);
        assert_eq!(cx.activated_item_label.as_deref(), Some("Beta"));
        assert_finished(&cx);
    }

    #[test]
    fn context_menu_top_level_pointer_down_delegates_to_selection_activation() {
        let mut canvas = test_canvas();
        canvas.interaction.context_menu =
            Some(menu(vec![item("Alpha", true), item("Beta", true)], 0));
        let position = row_position(&canvas, 1);
        let mut cx = StubCx::default();

        let handled = handle_context_menu_pointer_down::<StubHost, _>(
            &mut canvas,
            &mut cx,
            position,
            MouseButton::Left,
            1.0,
        );

        assert!(handled);
        assert!(canvas.interaction.context_menu.is_none());
        assert_eq!(cx.activation_calls, 1);
        assert_eq!(cx.activated_item_label.as_deref(), Some("Beta"));
        assert_finished(&cx);
    }

    #[test]
    fn context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint() {
        let mut canvas = test_canvas();
        canvas.interaction.context_menu =
            Some(menu(vec![item("Alpha", true), item("Beta", true)], 0));
        let position = row_position(&canvas, 1);
        let mut cx = StubCx::default();

        let handled =
            handle_context_menu_pointer_move::<StubHost, _>(&mut canvas, &mut cx, position, 1.0);

        assert!(handled);
        let menu = canvas.interaction.context_menu.as_ref().unwrap();
        assert_eq!(menu.hovered_item, Some(1));
        assert_eq!(menu.active_item, 1);
        assert_eq!(cx.activation_calls, 0);
        assert!(!cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
