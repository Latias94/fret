use fret_core::{CursorIcon, Point};
use fret_ui::UiHost;

use super::super::state::ViewSnapshot;
use super::NodeGraphCanvas;

pub(super) fn update_close_button_cursor<H: UiHost>(
    canvas: &NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if canvas.close_command.is_none()
        || canvas.interaction.node_drag.is_some()
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
