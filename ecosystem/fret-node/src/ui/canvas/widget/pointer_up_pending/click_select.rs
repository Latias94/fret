use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::Point;
use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::core::NodeId;
use crate::io::NodeGraphViewState;
use crate::ui::canvas::state::{PendingNodeSelectAction, ViewSnapshot};

pub(in super::super) fn handle_pending_node_drag_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(pending) = canvas.interaction.pending_node_drag.take() else {
        return false;
    };

    super::super::pointer_up_session::clear_node_drag_release_state(&mut canvas.interaction);

    if pending.select_action != PendingNodeSelectAction::None
        && is_click_release(
            pending.start_pos,
            position,
            snapshot.interaction.node_click_distance,
            zoom,
        )
    {
        let node = pending.primary;
        let select_action = pending.select_action;
        canvas.update_view_state(cx.app, |view| {
            apply_pending_node_selection(view, node, select_action);
        });
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

fn is_click_release(
    start_pos: Point,
    position: Point,
    click_distance_screen: f32,
    zoom: f32,
) -> bool {
    let dx = position.x.0 - start_pos.x.0;
    let dy = position.y.0 - start_pos.y.0;
    let click_distance_screen = click_distance_screen.max(0.0);
    let click_distance_canvas = canvas_units_from_screen_px(click_distance_screen, zoom);

    click_distance_screen == 0.0
        || (dx * dx + dy * dy) <= click_distance_canvas * click_distance_canvas
}

fn apply_pending_node_selection(
    view: &mut NodeGraphViewState,
    node: NodeId,
    select_action: PendingNodeSelectAction,
) {
    match select_action {
        PendingNodeSelectAction::Toggle => {
            if let Some(index) = view.selected_nodes.iter().position(|id| *id == node) {
                view.selected_nodes.remove(index);
            } else {
                view.selected_nodes.push(node);
            }
        }
        PendingNodeSelectAction::None => {}
    }

    view.draw_order.retain(|id| *id != node);
    view.draw_order.push(node);
}

#[cfg(test)]
mod tests;
