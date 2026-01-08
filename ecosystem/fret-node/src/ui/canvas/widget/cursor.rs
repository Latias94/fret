use fret_core::{CursorIcon, Point};
use fret_ui::UiHost;

use super::super::state::ViewSnapshot;
use super::NodeGraphCanvas;

pub(super) fn update_cursors<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    update_close_button_cursor(canvas, cx, snapshot, position, zoom);
    update_resize_handle_cursor(canvas, cx, snapshot, position, zoom);
}

fn update_close_button_cursor<H: UiHost>(
    canvas: &NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if canvas.close_command.is_none()
        || canvas.interaction.node_drag.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.edge_drag.is_some()
        || canvas.interaction.panning
    {
        return;
    }

    let rect = NodeGraphCanvas::close_button_rect(snapshot.pan, zoom);
    if NodeGraphCanvas::rect_contains(rect, position) {
        cx.set_cursor_icon(CursorIcon::Pointer);
    }
}

fn update_resize_handle_cursor<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if canvas.interaction.node_drag.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.edge_drag.is_some()
        || canvas.interaction.panning
        || canvas.interaction.marquee.is_some()
        || canvas.interaction.context_menu.is_some()
        || canvas.interaction.searcher.is_some()
    {
        return;
    }

    if snapshot.selected_nodes.is_empty() {
        return;
    }

    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    for node_id in &snapshot.selected_nodes {
        let Some(node_geom) = geom.nodes.get(node_id) else {
            continue;
        };
        let handle = canvas.resize_handle_rect(node_geom.rect, zoom);
        if NodeGraphCanvas::rect_contains(handle, position) {
            cx.set_cursor_icon(CursorIcon::ColResize);
            return;
        }
    }
}
