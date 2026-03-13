use fret_core::Point;
use fret_ui::UiHost;

use crate::core::{CanvasRect, GroupId};
use crate::ui::canvas::state::{PendingGroupDrag, PendingGroupResize};
use crate::ui::canvas::widget::*;

pub(super) fn begin_group_resize<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    group: GroupId,
    rect: CanvasRect,
) {
    canvas.interaction.pending_group_resize = Some(PendingGroupResize {
        group,
        start_pos: position,
        start_rect: rect,
    });
    canvas.interaction.group_resize = None;
    finish_group_pointer_down(cx);
}

pub(super) fn begin_group_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    group: GroupId,
    rect: CanvasRect,
) {
    canvas.interaction.pending_group_drag = Some(PendingGroupDrag {
        group,
        start_pos: position,
        start_rect: rect,
    });
    canvas.interaction.group_drag = None;
    canvas.interaction.pending_group_resize = None;
    canvas.interaction.group_resize = None;
    finish_group_pointer_down(cx);
}

fn finish_group_pointer_down<H: UiHost>(cx: &mut fret_ui::retained_bridge::EventCx<'_, H>) {
    cx.capture_pointer(cx.node);
    super::super::super::paint_invalidation::invalidate_paint(cx);
}
