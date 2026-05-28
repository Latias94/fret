use fret_core::{Point, Px};

pub(in crate::imui::debug_draw_controls) fn quadratic_bezier_point(
    from: Point,
    ctrl: Point,
    to: Point,
    t: f32,
) -> Point {
    let u = 1.0 - t;
    Point::new(
        Px(u * u * from.x.0 + 2.0 * u * t * ctrl.x.0 + t * t * to.x.0),
        Px(u * u * from.y.0 + 2.0 * u * t * ctrl.y.0 + t * t * to.y.0),
    )
}

pub(in crate::imui::debug_draw_controls) fn cubic_bezier_point(
    from: Point,
    ctrl1: Point,
    ctrl2: Point,
    to: Point,
    t: f32,
) -> Point {
    let u = 1.0 - t;
    let uu = u * u;
    let tt = t * t;
    Point::new(
        Px(uu * u * from.x.0
            + 3.0 * uu * t * ctrl1.x.0
            + 3.0 * u * tt * ctrl2.x.0
            + tt * t * to.x.0),
        Px(uu * u * from.y.0
            + 3.0 * uu * t * ctrl1.y.0
            + 3.0 * u * tt * ctrl2.y.0
            + tt * t * to.y.0),
    )
}
