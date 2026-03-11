use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(super) fn dispatch_auto_pan_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) {
    if canvas.interaction.wire_drag.is_some() {
        let _ = wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom);
    } else if canvas.interaction.node_drag.is_some() {
        let _ = node_drag::handle_node_drag_move(canvas, cx, snapshot, position, modifiers, zoom);
    } else if canvas.interaction.group_drag.is_some() {
        let _ = group_drag::handle_group_drag_move(canvas, cx, snapshot, position, modifiers, zoom);
    } else if canvas.interaction.group_resize.is_some() {
        let _ =
            group_resize::handle_group_resize_move(canvas, cx, snapshot, position, modifiers, zoom);
    }
}
