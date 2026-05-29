use fret_core::{Point, Px, Size};
use fret_interaction::dpi;

pub(in crate::imui) fn snap_point_to_device_pixels(scale_factor: f32, p: Point) -> Point {
    dpi::snap_point_to_device_pixels(scale_factor, p)
}

pub(in crate::imui) fn snap_size_to_device_pixels(scale_factor: f32, s: Size) -> Size {
    dpi::snap_size_to_device_pixels(scale_factor, s)
}

pub(in crate::imui) fn point_sub(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 - b.x.0), Px(a.y.0 - b.y.0))
}

pub(in crate::imui) fn point_add(a: Point, b: Point) -> Point {
    Point::new(Px(a.x.0 + b.x.0), Px(a.y.0 + b.y.0))
}
