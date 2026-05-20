mod pointer;
mod tail;

use fret_core::{Modifiers, Point, Rect};
use fret_ui::UiHost;

use crate::core::CanvasRect;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
    group_preview_move_cx::GroupPreviewMoveCx,
};

pub(super) fn handle_group_resize_move<H: UiHost, M, Cx>(
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
    let Some(mut resize) = canvas.interaction.group_resize.clone() else {
        return false;
    };

    let auto_pan_delta = pointer::auto_pan_delta::<M>(snapshot, position, cx.bounds());
    let position = pointer::adjusted_position(position, auto_pan_delta);

    let new_rect = super::group_resize_apply::next_group_resize_rect(
        canvas,
        cx.host(),
        snapshot,
        &resize,
        position,
        modifiers,
    );

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.host(), |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }
    tail::finish_group_resize_move(canvas, cx, &mut resize, new_rect);
    true
}

pub(super) fn group_rect_to_px(rect: CanvasRect) -> Rect {
    super::group_resize_hit::group_rect_to_px(rect)
}

pub(super) fn group_resize_handle_hit(
    handle: Rect,
    position: Point,
    zoom: f32,
    padding_screen: f32,
) -> bool {
    super::group_resize_hit::group_resize_handle_hit(handle, position, zoom, padding_screen)
}
