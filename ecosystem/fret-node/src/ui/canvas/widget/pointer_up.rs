use fret_canvas::scale::canvas_units_from_screen_px;
use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit};
use crate::ui::canvas::state::{PendingNodeSelectAction, ViewSnapshot, WireDrag, WireDragKind};

pub(super) fn handle_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    click_count: u8,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.last_canvas_pos = Some(crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    });

    if button == MouseButton::Left
        && canvas.interaction.sticky_wire_ignore_next_up
        && canvas.interaction.wire_drag.is_some()
    {
        canvas.interaction.sticky_wire_ignore_next_up = false;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.panning && canvas.interaction.panning_button == Some(button) {
        canvas.interaction.panning = false;
        canvas.interaction.panning_button = None;
        canvas.interaction.pan_last_screen_pos = None;
        canvas.interaction.pan_last_sample_at = None;
        canvas.stop_auto_pan_timer(cx.app);
        let started_inertia = canvas.maybe_start_pan_inertia_timer(cx.app, cx.window, snapshot);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanDrag,
            ViewportMoveEndOutcome::Ended,
        );
        if started_inertia {
            canvas.emit_move_start(snapshot, ViewportMoveKind::PanInertia);
        }
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if button != MouseButton::Left {
        return false;
    }

    canvas.stop_auto_pan_timer(cx.app);

    if click_count == 2
        && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
        && let Some(edge_drag) = canvas.interaction.edge_drag.take()
    {
        canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_drag.edge, position);

        canvas.interaction.hover_edge = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if super::marquee::handle_left_up(canvas, cx) {
        return true;
    }

    if pointer_up_commit::handle_node_resize_release(canvas, cx) {
        return true;
    }

    if pointer_up_commit::handle_group_resize_release(canvas, cx) {
        return true;
    }

    if pointer_up_commit::handle_group_drag_release(canvas, cx) {
        return true;
    }

    if pointer_up_commit::handle_node_drag_release(canvas, cx, snapshot) {
        return true;
    }

    if canvas.interaction.pending_group_drag.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_group_resize.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(pending) = canvas.interaction.pending_node_drag.take() {
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

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_node_resize.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(pending) = canvas.interaction.pending_wire_drag.take() {
        if snapshot.interaction.connect_on_click {
            if matches!(pending.kind, WireDragKind::New { .. }) {
                let kind = pending.kind.clone();
                canvas.interaction.wire_drag = Some(WireDrag {
                    kind: pending.kind,
                    pos: position,
                });
                canvas.interaction.click_connect = true;
                canvas.emit_connect_start(snapshot, &kind);
            }
        }

        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom) {
        return true;
    }

    if super::edge_insert_drag::handle_edge_insert_left_up(canvas, cx, position) {
        return true;
    }

    if super::edge_drag::handle_edge_left_up(canvas, cx) {
        return true;
    }

    false
}
