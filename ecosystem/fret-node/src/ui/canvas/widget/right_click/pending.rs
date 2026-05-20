use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use super::super::context_menu::opening;
use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use super::RightClickCx;
use crate::ui::canvas::state::ViewSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingRightClickPointerUpPlan {
    Ignored,
    ReleaseOnly,
    ReleaseAndOpenMenu,
}

fn plan_pending_right_click_pointer_up<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> PendingRightClickPointerUpPlan {
    if button != MouseButton::Right || !snapshot.interaction.pan_on_drag.right {
        return PendingRightClickPointerUpPlan::Ignored;
    }

    let Some(pending) = canvas.interaction.pending_right_click.take() else {
        return PendingRightClickPointerUpPlan::Ignored;
    };

    if super::threshold::pending_right_click_is_click(
        pending,
        position,
        snapshot.interaction.pane_click_distance,
        zoom,
    ) {
        PendingRightClickPointerUpPlan::ReleaseAndOpenMenu
    } else {
        PendingRightClickPointerUpPlan::ReleaseOnly
    }
}

pub(in super::super) fn handle_right_click_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl RightClickCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    opening::handle_right_click_context_menu_event(canvas, cx, snapshot, position, zoom)
}

pub(in super::super) fn handle_pending_right_click_pointer_up<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl RightClickCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    match plan_pending_right_click_pointer_up(canvas, snapshot, position, button, zoom) {
        PendingRightClickPointerUpPlan::Ignored => false,
        PendingRightClickPointerUpPlan::ReleaseOnly => {
            cx.release_pointer_capture();
            true
        }
        PendingRightClickPointerUpPlan::ReleaseAndOpenMenu => {
            cx.release_pointer_capture();
            let _ = handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px};

    use super::*;
    use crate::core::{Graph, GraphId};
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::canvas::state::PendingRightClick;
    use crate::ui::canvas::widget::NoopNodeGraphCanvasMiddleware;
    use fret_runtime::ModelStore;

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

    fn snapshot() -> ViewSnapshot {
        let mut interaction = crate::io::NodeGraphInteractionState::default();
        interaction.pan_on_drag.right = true;
        interaction.pane_click_distance = 2.0;
        ViewSnapshot {
            pan: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
            zoom: 1.0,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            selected_groups: Vec::new(),
            draw_order: Vec::new(),
            group_draw_order: Vec::new(),
            interaction,
        }
    }

    #[test]
    fn pending_right_click_pointer_up_ignores_non_right_button() {
        let mut canvas = test_canvas();
        canvas.interaction.pending_right_click = Some(PendingRightClick {
            start_pos: Point::new(Px(10.0), Px(10.0)),
        });

        let plan = plan_pending_right_click_pointer_up(
            &mut canvas,
            &snapshot(),
            Point::new(Px(10.0), Px(10.0)),
            MouseButton::Left,
            1.0,
        );

        assert_eq!(plan, PendingRightClickPointerUpPlan::Ignored);
        assert!(canvas.interaction.pending_right_click.is_some());
    }

    #[test]
    fn pending_right_click_pointer_up_without_pending_state_is_side_effect_free() {
        let mut canvas = test_canvas();

        let plan = plan_pending_right_click_pointer_up(
            &mut canvas,
            &snapshot(),
            Point::new(Px(10.0), Px(10.0)),
            MouseButton::Right,
            1.0,
        );

        assert_eq!(plan, PendingRightClickPointerUpPlan::Ignored);
        assert!(canvas.interaction.pending_right_click.is_none());
    }

    #[test]
    fn pending_right_click_drag_release_clears_pending_and_releases_capture() {
        let mut canvas = test_canvas();
        canvas.interaction.pending_right_click = Some(PendingRightClick {
            start_pos: Point::new(Px(10.0), Px(10.0)),
        });

        let plan = plan_pending_right_click_pointer_up(
            &mut canvas,
            &snapshot(),
            Point::new(Px(30.0), Px(10.0)),
            MouseButton::Right,
            1.0,
        );

        assert_eq!(plan, PendingRightClickPointerUpPlan::ReleaseOnly);
        assert!(canvas.interaction.pending_right_click.is_none());
        assert!(canvas.interaction.context_menu.is_none());
    }

    #[test]
    fn pending_right_click_click_release_requests_menu_open() {
        let mut canvas = test_canvas();
        canvas.interaction.pending_right_click = Some(PendingRightClick {
            start_pos: Point::new(Px(10.0), Px(10.0)),
        });

        let plan = plan_pending_right_click_pointer_up(
            &mut canvas,
            &snapshot(),
            Point::new(Px(11.0), Px(10.0)),
            MouseButton::Right,
            1.0,
        );

        assert_eq!(plan, PendingRightClickPointerUpPlan::ReleaseAndOpenMenu);
        assert!(canvas.interaction.pending_right_click.is_none());
    }
}
