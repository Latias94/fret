mod delta;
mod tail;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn handle_group_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    let Some(mut drag) = canvas.interaction.group_drag.clone() else {
        return false;
    };

    let auto_pan_delta = delta::auto_pan_delta::<M>(snapshot, position, cx.bounds);
    let delta =
        delta::planned_drag_delta::<M>(snapshot, &drag, position, modifiers, auto_pan_delta);
    tail::finish_group_drag_move(canvas, cx, &mut drag, delta, auto_pan_delta);
    true
}
