mod delta;
mod tail;

use fret_core::Modifiers;
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot, node_drag_constraints,
    node_drag_preview, node_drag_snap,
};

pub(super) fn handle_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: fret_core::Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(mut drag) = canvas.interaction.node_drag.clone() else {
        return false;
    };
    let multi_drag = drag.nodes.len() > 1;

    let auto_pan_delta = delta::auto_pan_delta::<M>(snapshot, position, cx.bounds);
    let mut delta = delta::planned_drag_delta::<M>(snapshot, &drag, position, auto_pan_delta);

    delta = node_drag_snap::apply_snaplines_delta(
        canvas,
        cx,
        snapshot,
        &drag,
        delta,
        snapshot.interaction.snaplines,
        snapshot.interaction.snaplines_threshold,
        modifiers,
        zoom,
    );

    delta = node_drag_constraints::apply_multi_drag_extent_delta(
        canvas,
        cx,
        snapshot,
        &drag.node_ids,
        delta,
        multi_drag,
    );
    let (next_nodes, next_groups) = node_drag_preview::compute_preview_positions(
        canvas, cx, snapshot, &drag, delta, multi_drag,
    );
    node_drag_preview::update_drag_preview_state(&mut drag, next_nodes, next_groups);
    canvas.interaction.node_drag = Some(drag.clone());

    tail::finish_node_drag_move(canvas, cx, &drag, auto_pan_delta);
    true
}
