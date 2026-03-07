use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit, pointer_up_pending,
};
use crate::ui::canvas::state::ViewSnapshot;

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

    if pointer_up_pending::handle_pending_group_drag_release(canvas, cx) {
        return true;
    }

    if pointer_up_pending::handle_pending_group_resize_release(canvas, cx) {
        return true;
    }

    if pointer_up_pending::handle_pending_node_drag_release(canvas, cx, snapshot, position, zoom) {
        return true;
    }

    if pointer_up_pending::handle_pending_node_resize_release(canvas, cx) {
        return true;
    }

    if pointer_up_pending::handle_pending_wire_drag_release(canvas, cx, snapshot, position) {
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
