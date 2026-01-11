use fret_core::Point;
use fret_ui::UiHost;

use super::super::state::{ViewSnapshot, WireDrag, WireDragKind};
use super::NodeGraphCanvas;
use super::threshold::exceeds_drag_threshold;

pub(super) fn handle_edge_drag_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(drag) = canvas.interaction.edge_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold;
    if !exceeds_drag_threshold(drag.start_pos, position, threshold_screen) {
        return false;
    }

    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    let reconnect = canvas
        .graph
        .read_ref(cx.app, |graph| {
            canvas.pick_reconnect_endpoint(
                graph,
                geom.as_ref(),
                drag.edge,
                drag.start_pos,
                snapshot.interaction.reconnect_radius,
                zoom,
            )
        })
        .ok()
        .flatten();

    let Some((endpoint, fixed)) = reconnect else {
        return false;
    };
    let endpoint_allowed = canvas
        .graph
        .read_ref(cx.app, |graph| {
            NodeGraphCanvas::edge_endpoint_is_reconnectable(
                graph,
                &snapshot.interaction,
                drag.edge,
                endpoint,
            )
        })
        .ok()
        .unwrap_or(false);
    if !endpoint_allowed {
        canvas.interaction.edge_drag = None;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    canvas.interaction.edge_drag = None;
    canvas.interaction.hover_edge = None;
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::Reconnect {
            edge: drag.edge,
            endpoint,
            fixed,
        },
        pos: position,
    });

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_edge_left_up<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.edge_drag.take().is_some() {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    false
}
