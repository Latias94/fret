use fret_core::Point;
use fret_ui::UiHost;

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit, pointer_up_pending,
};
use crate::ui::canvas::state::ViewSnapshot;

pub(in super::super) fn handle_left_release_chain<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    if super::super::marquee::handle_left_up(canvas, cx) {
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

    if super::super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom) {
        return true;
    }

    if super::super::edge_insert_drag::handle_edge_insert_left_up(canvas, cx, position) {
        return true;
    }

    if super::super::edge_drag::handle_edge_left_up(canvas, cx) {
        return true;
    }

    false
}
