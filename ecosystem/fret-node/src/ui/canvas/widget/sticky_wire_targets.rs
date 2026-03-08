use fret_core::Point;
use fret_ui::UiHost;

use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_sticky_wire_non_port_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &crate::ui::canvas::CanvasGeometry,
    index: &crate::ui::canvas::CanvasSpatialDerived,
    from_port: crate::core::PortId,
    position: Point,
    zoom: f32,
) -> bool {
    let at = canvas.interaction.last_canvas_pos.unwrap_or_default();
    let (on_node, hit_edge) =
        inspect_non_port_target(canvas, cx.app, snapshot, geom, index, position, zoom);

    reset_sticky_wire_state(canvas);
    cx.release_pointer_capture();

    if on_node {
        return false;
    }
    if let Some(edge_id) = hit_edge {
        canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, position);
        finish_sticky_wire_target_picker(cx);
        return true;
    }

    canvas.open_connection_insert_node_picker(cx.app, from_port, at);
    finish_sticky_wire_target_picker(cx);
    true
}

pub(super) fn reset_sticky_wire_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.interaction.sticky_wire = false;
    canvas.interaction.sticky_wire_ignore_next_up = false;
}

fn inspect_non_port_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    geom: &crate::ui::canvas::CanvasGeometry,
    index: &crate::ui::canvas::CanvasSpatialDerived,
    position: Point,
    zoom: f32,
) -> (bool, Option<crate::core::EdgeId>) {
    canvas
        .graph
        .read_ref(host, |graph| {
            let on_node = geom.order.iter().rev().any(|id| {
                geom.nodes
                    .get(id)
                    .is_some_and(|node| node.rect.contains(position))
            });
            if on_node {
                return (true, None);
            }
            let mut scratch = HitTestScratch::default();
            let mut hit_test = HitTestCtx::new(geom, index, zoom, &mut scratch);
            let hit_edge = canvas.hit_edge(graph, snapshot, &mut hit_test, position);
            (false, hit_edge)
        })
        .ok()
        .unwrap_or((false, None))
}

fn finish_sticky_wire_target_picker<H: UiHost>(cx: &mut fret_ui::retained_bridge::EventCx<'_, H>) {
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}
