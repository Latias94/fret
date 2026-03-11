use fret_core::{Point, Px, Rect};

use crate::core::CanvasPoint;
use crate::ui::canvas::state::{NodeDrag, ViewSnapshot};
use crate::ui::canvas::widget::*;

pub(super) fn auto_pan_delta<M: NodeGraphCanvasMiddleware>(
    snapshot: &ViewSnapshot,
    position: Point,
    bounds: Rect,
) -> CanvasPoint {
    snapshot
        .interaction
        .auto_pan
        .on_node_drag
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, bounds))
        .unwrap_or_default()
}

pub(super) fn planned_drag_delta<M: NodeGraphCanvasMiddleware>(
    snapshot: &ViewSnapshot,
    drag: &NodeDrag,
    position: Point,
    auto_pan_delta: CanvasPoint,
) -> CanvasPoint {
    let start_anchor = Point::new(
        Px(drag.start_pos.x.0 - drag.grab_offset.x.0),
        Px(drag.start_pos.y.0 - drag.grab_offset.y.0),
    );
    let new_anchor = Point::new(
        Px(position.x.0 - drag.grab_offset.x.0 - auto_pan_delta.x),
        Px(position.y.0 - drag.grab_offset.y.0 - auto_pan_delta.y),
    );
    let delta = CanvasPoint {
        x: new_anchor.x.0 - start_anchor.x.0,
        y: new_anchor.y.0 - start_anchor.y.0,
    };

    if snapshot.interaction.snap_to_grid {
        snap_delta_to_grid::<M>(drag, delta, snapshot.interaction.snap_grid)
    } else {
        delta
    }
}

fn snap_delta_to_grid<M: NodeGraphCanvasMiddleware>(
    drag: &NodeDrag,
    delta: CanvasPoint,
    snap_grid: crate::core::CanvasSize,
) -> CanvasPoint {
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
    CanvasPoint {
        x: snapped.x - primary_start.x,
        y: snapped.y - primary_start.y,
    }
}
