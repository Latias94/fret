use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingNodeSelectAction, ViewSnapshot, WireDrag, WireDragKind};

pub(super) fn handle_pending_group_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.pending_group_drag.take().is_none() {
        return false;
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_pending_group_resize_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.pending_group_resize.take().is_none() {
        return false;
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_pending_node_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(pending) = canvas.interaction.pending_node_drag.take() else {
        return false;
    };

    canvas.interaction.pending_node_resize = None;
    canvas.interaction.snap_guides = None;

    if pending.select_action != PendingNodeSelectAction::None {
        let dx = position.x.0 - pending.start_pos.x.0;
        let dy = position.y.0 - pending.start_pos.y.0;
        let click_distance_screen = snapshot.interaction.node_click_distance.max(0.0);
        let click_distance_canvas = canvas_units_from_screen_px(click_distance_screen, zoom);
        let is_click = click_distance_screen == 0.0
            || (dx * dx + dy * dy) <= click_distance_canvas * click_distance_canvas;

        if is_click {
            let node = pending.primary;
            canvas.update_view_state(cx.app, |s| {
                match pending.select_action {
                    PendingNodeSelectAction::Toggle => {
                        if let Some(ix) = s.selected_nodes.iter().position(|id| *id == node) {
                            s.selected_nodes.remove(ix);
                        } else {
                            s.selected_nodes.push(node);
                        }
                    }
                    PendingNodeSelectAction::None => {}
                }

                s.draw_order.retain(|id| *id != node);
                s.draw_order.push(node);
            });
        }
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_pending_node_resize_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.pending_node_resize.take().is_none() {
        return false;
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn handle_pending_wire_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let Some(pending) = canvas.interaction.pending_wire_drag.take() else {
        return false;
    };

    if snapshot.interaction.connect_on_click && matches!(pending.kind, WireDragKind::New { .. }) {
        let kind = pending.kind.clone();
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: pending.kind,
            pos: position,
        });
        canvas.interaction.click_connect = true;
        canvas.emit_connect_start(snapshot, &kind);
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}
