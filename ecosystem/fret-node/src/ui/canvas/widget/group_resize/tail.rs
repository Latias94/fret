use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect};
use crate::ui::canvas::state::GroupResize;
use crate::ui::canvas::widget::*;

pub(super) fn finish_group_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    resize: &mut GroupResize,
    new_rect: CanvasRect,
    auto_pan_delta: CanvasPoint,
) {
    update_resize_preview_state(resize, new_rect);
    canvas.interaction.group_resize = Some(resize.clone());

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    super::super::paint_invalidation::invalidate_paint(cx);
}

fn update_resize_preview_state(resize: &mut GroupResize, new_rect: CanvasRect) {
    if resize.current_rect != new_rect {
        resize.current_rect = new_rect;
        resize.preview_rev = resize.preview_rev.wrapping_add(1);
    }
}
