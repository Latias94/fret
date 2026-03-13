use fret_core::{Modifiers, Point, Px, Rect};

use crate::core::CanvasPoint;
use crate::ui::canvas::state::{GroupDrag, ViewSnapshot};
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
    drag: &GroupDrag,
    position: Point,
    modifiers: Modifiers,
    auto_pan_delta: CanvasPoint,
) -> CanvasPoint {
    let position = Point::new(
        Px(position.x.0 - auto_pan_delta.x),
        Px(position.y.0 - auto_pan_delta.y),
    );

    let delta = CanvasPoint {
        x: position.x.0 - drag.start_pos.x.0,
        y: position.y.0 - drag.start_pos.y.0,
    };

    let allow_snap = !modifiers.alt && !modifiers.alt_gr;
    if allow_snap && snapshot.interaction.snap_to_grid {
        snap_delta_to_grid::<M>(drag, delta, snapshot.interaction.snap_grid)
    } else {
        delta
    }
}

fn snap_delta_to_grid<M: NodeGraphCanvasMiddleware>(
    drag: &GroupDrag,
    delta: CanvasPoint,
    snap_grid: crate::core::CanvasSize,
) -> CanvasPoint {
    let origin0 = drag.start_rect.origin;
    let target = CanvasPoint {
        x: origin0.x + delta.x,
        y: origin0.y + delta.y,
    };
    let snapped = NodeGraphCanvasWith::<M>::snap_canvas_point(target, snap_grid);
    CanvasPoint {
        x: snapped.x - origin0.x,
        y: snapped.y - origin0.y,
    }
}
