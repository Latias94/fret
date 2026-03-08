use fret_core::{Modifiers, Point, Px};
use fret_ui::UiHost;

use crate::core::CanvasPoint;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot, node_drag_constraints,
    node_drag_preview, node_drag_snap,
};

pub(super) fn handle_node_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    let Some(mut drag) = canvas.interaction.node_drag.clone() else {
        return false;
    };
    let multi_drag = drag.nodes.len() > 1;

    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();
    let snap_to_grid = snapshot.interaction.snap_to_grid;
    let snap_grid = snapshot.interaction.snap_grid;
    let snaplines = snapshot.interaction.snaplines;
    let snaplines_threshold_screen = snapshot.interaction.snaplines_threshold;

    let start_anchor = Point::new(
        Px(drag.start_pos.x.0 - drag.grab_offset.x.0),
        Px(drag.start_pos.y.0 - drag.grab_offset.y.0),
    );
    let new_anchor = Point::new(
        Px(position.x.0 - drag.grab_offset.x.0 - auto_pan_delta.x),
        Px(position.y.0 - drag.grab_offset.y.0 - auto_pan_delta.y),
    );
    let mut delta = CanvasPoint {
        x: new_anchor.x.0 - start_anchor.x.0,
        y: new_anchor.y.0 - start_anchor.y.0,
    };

    if snap_to_grid {
        let primary_start = drag
            .nodes
            .iter()
            .find(|(id, _)| *id == drag.primary)
            .map(|(_, p)| *p)
            .unwrap_or_default();
        let primary_target = CanvasPoint {
            x: primary_start.x + delta.x,
            y: primary_start.y + delta.y,
        };
        let snapped = NodeGraphCanvasWith::<M>::snap_canvas_point(primary_target, snap_grid);
        delta = CanvasPoint {
            x: snapped.x - primary_start.x,
            y: snapped.y - primary_start.y,
        };
    }

    delta = node_drag_snap::apply_snaplines_delta(
        canvas,
        cx,
        snapshot,
        &drag,
        delta,
        snaplines,
        snaplines_threshold_screen,
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

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    canvas.emit_node_drag(drag.primary, &drag.node_ids);

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
