mod rects;
mod resize;

pub(super) use rects::{
    canvas_rect_intersection, canvas_rect_union, clamp_finite_positive, normalize_canvas_rect,
};
pub(super) use resize::apply_resize_handle;

#[cfg(test)]
mod tests;
