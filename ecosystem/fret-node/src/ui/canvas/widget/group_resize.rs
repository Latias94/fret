mod pointer;
mod tail;

use fret_core::{Modifiers, Point, Rect};
use fret_ui::UiHost;

use crate::core::CanvasRect;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn handle_group_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    let Some(mut resize) = canvas.interaction.group_resize.clone() else {
        return false;
    };

    let auto_pan_delta = pointer::auto_pan_delta::<M>(snapshot, position, cx.bounds);
    let position = pointer::adjusted_position(position, auto_pan_delta);

    let new_rect = super::group_resize_apply::next_group_resize_rect(
        canvas, cx.app, snapshot, &resize, position, modifiers,
    );

    tail::finish_group_resize_move(canvas, cx, &mut resize, new_rect, auto_pan_delta);
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
