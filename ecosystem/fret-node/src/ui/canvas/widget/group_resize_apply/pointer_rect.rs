use fret_core::Point;

use crate::core::CanvasRect;
use crate::ui::canvas::state::GroupResize;

pub(super) fn group_resize_rect_from_pointer(resize: &GroupResize, position: Point) -> CanvasRect {
    let dx = position.x.0 - resize.start_pos.x.0;
    let dy = position.y.0 - resize.start_pos.y.0;

    let mut new_rect = resize.start_rect;
    new_rect.size.width = resize.start_rect.size.width + dx;
    new_rect.size.height = resize.start_rect.size.height + dy;
    new_rect
}

pub(super) fn clamp_group_resize_size(size: &mut f32, min_size: f32) {
    if size.is_finite() {
        *size = size.max(min_size);
    } else {
        *size = min_size;
    }
}
