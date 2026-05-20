mod delta;
mod tail;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    group_preview_move_cx::GroupPreviewMoveCx,
};

pub(super) fn handle_group_drag_move<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: GroupPreviewMoveCx<H>,
{
    let Some(mut drag) = canvas.interaction.group_drag.clone() else {
        return false;
    };

    let auto_pan_delta = delta::auto_pan_delta::<M>(snapshot, position, cx.bounds());
    let delta =
        delta::planned_drag_delta::<M>(snapshot, &drag, position, modifiers, auto_pan_delta);
    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.host(), |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }
    tail::finish_group_drag_move(canvas, cx, &mut drag, delta);
    true
}
