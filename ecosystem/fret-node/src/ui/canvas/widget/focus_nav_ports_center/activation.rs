use fret_core::{Point, Px, Rect};

pub(super) fn resolve_activation_point(
    port_center: Option<Point>,
    last_pos: Option<Point>,
    last_bounds: Option<Rect>,
) -> Point {
    port_center.or(last_pos).unwrap_or_else(|| {
        let bounds = last_bounds.unwrap_or_default();
        Point::new(
            Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
            Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
        )
    })
}

#[cfg(test)]
mod tests;
