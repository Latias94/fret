use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit, pointer_up_pending,
};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_left_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    canvas.stop_auto_pan_timer(cx.app);

    if should_open_edge_insert_picker(click_count, modifiers)
        && let Some(edge_drag) = canvas.interaction.edge_drag.take()
    {
        canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_drag.edge, position);

        canvas.interaction.hover_edge = None;
        cx.release_pointer_capture();
        super::paint_invalidation::invalidate_paint(cx);
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

fn should_open_edge_insert_picker(click_count: u8, modifiers: Modifiers) -> bool {
    click_count == 2 && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_open_edge_insert_picker_requires_plain_double_click() {
        assert!(should_open_edge_insert_picker(2, Modifiers::default()));
        assert!(!should_open_edge_insert_picker(1, Modifiers::default()));
        assert!(!should_open_edge_insert_picker(
            2,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            }
        ));
        assert!(!should_open_edge_insert_picker(
            2,
            Modifiers {
                alt_gr: true,
                ..Modifiers::default()
            }
        ));
    }
}
