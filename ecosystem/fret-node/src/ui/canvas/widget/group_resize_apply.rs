mod children_min;
mod pointer_rect;
mod snap;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::CanvasRect;
use crate::ui::canvas::state::GroupResize;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

const MIN_GROUP_WIDTH: f32 = 80.0;
const MIN_GROUP_HEIGHT: f32 = 60.0;

pub(super) fn next_group_resize_rect<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    resize: &GroupResize,
    position: Point,
    modifiers: Modifiers,
) -> CanvasRect {
    let mut new_rect = pointer_rect::group_resize_rect_from_pointer(resize, position);
    pointer_rect::clamp_group_resize_size(&mut new_rect.size.width, MIN_GROUP_WIDTH);
    pointer_rect::clamp_group_resize_size(&mut new_rect.size.height, MIN_GROUP_HEIGHT);

    let (min_w_children, min_h_children) =
        children_min::min_group_resize_children_size(canvas, host, snapshot, resize, &new_rect);
    new_rect.size.width = new_rect.size.width.max(min_w_children);
    new_rect.size.height = new_rect.size.height.max(min_h_children);

    if snap::allow_group_resize_snap(snapshot, modifiers) {
        let (snapped_w, snapped_h) = snap::snapped_group_resize_size::<M>(
            &new_rect,
            snapshot,
            min_w_children,
            min_h_children,
        );
        new_rect.size.width = snapped_w;
        new_rect.size.height = snapped_h;
    }

    new_rect
}
