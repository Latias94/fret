use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use crate::ops::GraphOp;

use super::super::state::ViewSnapshot;
use super::NodeGraphCanvas;

pub(super) fn handle_pointer_up<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_canvas_pos = Some(NodeGraphCanvas::screen_to_canvas(
        cx.bounds,
        position,
        snapshot.pan,
        zoom,
    ));

    if button == MouseButton::Left
        && canvas.interaction.sticky_wire_ignore_next_up
        && canvas.interaction.wire_drag.is_some()
    {
        canvas.interaction.sticky_wire_ignore_next_up = false;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if button == MouseButton::Middle && canvas.interaction.panning {
        canvas.interaction.panning = false;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if button != MouseButton::Left {
        return false;
    }

    if super::marquee::handle_left_up(canvas, cx) {
        return true;
    }

    if let Some(drag) = canvas.interaction.node_drag.take() {
        let ops = canvas
            .graph
            .read_ref(cx.app, |g| {
                drag.nodes
                    .iter()
                    .filter_map(|(id, start)| {
                        let end = g.nodes.get(id).map(|n| n.pos)?;
                        (end != *start).then_some(GraphOp::SetNodePos {
                            id: *id,
                            from: *start,
                            to: end,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .ok()
            .unwrap_or_default();
        if !ops.is_empty() {
            let label = if ops.len() == 1 {
                "Move Node"
            } else {
                "Move Nodes"
            };
            canvas.history.record(crate::ops::GraphTransaction {
                label: Some(label.to_string()),
                ops,
            });
        }
        canvas.interaction.pending_node_drag = None;
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if canvas.interaction.pending_node_drag.take().is_some() {
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom) {
        return true;
    }

    if super::edge_drag::handle_edge_left_up(canvas, cx) {
        return true;
    }

    false
}
