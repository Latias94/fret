use fret_core::{Modifiers, Point, Px, Rect};
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

    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();
    let position = Point::new(
        Px(position.x.0 - auto_pan_delta.x),
        Px(position.y.0 - auto_pan_delta.y),
    );

    let new_rect = super::group_resize_apply::next_group_resize_rect(
        canvas, cx.app, snapshot, &resize, position, modifiers,
    );

    if resize.current_rect != new_rect {
        resize.current_rect = new_rect;
        resize.preview_rev = resize.preview_rev.wrapping_add(1);
    }
    canvas.interaction.group_resize = Some(resize);

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
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
