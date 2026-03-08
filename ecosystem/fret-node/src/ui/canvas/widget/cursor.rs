use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn update_cursors<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    update_close_button_cursor(canvas, cx, snapshot, position, zoom);
    update_resize_handle_cursor(canvas, cx, snapshot, position, zoom);
    update_edge_anchor_cursor(canvas, cx, snapshot, position);
}

fn update_close_button_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if !super::cursor_gate::allow_close_button_cursor(
        canvas.close_command.is_some(),
        &canvas.interaction,
    ) {
        return;
    }

    let rect = NodeGraphCanvasWith::<M>::close_button_rect(snapshot.pan, zoom);
    if NodeGraphCanvasWith::<M>::rect_contains(rect, position) {
        cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
    }
}

fn update_resize_handle_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if !super::cursor_gate::allow_canvas_detail_cursor(&canvas.interaction)
        || snapshot.selected_nodes.is_empty()
    {
        return;
    }

    let icon = super::cursor_resolve::resolve_resize_handle_cursor(
        canvas, cx.app, snapshot, position, zoom,
    );
    if let Some(icon) = icon {
        cx.set_cursor_icon(icon);
    }
}

fn update_edge_anchor_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) {
    if !super::cursor_gate::allow_canvas_detail_cursor(&canvas.interaction) {
        return;
    }

    let icon = super::cursor_resolve::resolve_edge_anchor_cursor(canvas, snapshot, position);
    if let Some(icon) = icon {
        cx.set_cursor_icon(icon);
    }
}
