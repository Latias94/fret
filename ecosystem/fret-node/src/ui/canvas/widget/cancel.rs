use fret_ui::UiHost;

use super::NodeGraphCanvas;

pub(super) fn handle_escape_cancel<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    let mut canceled = false;
    if canvas.interaction.wire_drag.take().is_some() {
        canvas.interaction.click_connect = false;
        canceled = true;
    }
    if canvas.interaction.edge_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.node_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_node_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.group_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_group_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.group_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_group_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.node_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_node_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_wire_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.marquee.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_marquee.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.panning {
        canvas.interaction.panning = false;
        canceled = true;
    }
    if canvas.interaction.sticky_wire || canvas.interaction.sticky_wire_ignore_next_up {
        canvas.interaction.sticky_wire = false;
        canvas.interaction.sticky_wire_ignore_next_up = false;
        canceled = true;
    }
    if canvas.interaction.snap_guides.take().is_some() {
        canceled = true;
    }
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_edge = None;

    if canceled {
        canvas.stop_auto_pan_timer(cx.app);
        cx.release_pointer_capture();
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}
