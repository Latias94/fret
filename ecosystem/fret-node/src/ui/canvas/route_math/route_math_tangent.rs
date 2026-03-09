use super::*;

pub(crate) fn edge_route_start_tangent(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
) -> Point {
    match route {
        EdgeRouteKind::Bezier => {
            let (ctrl1, ctrl2) = wire_ctrl_points(from, to, zoom);
            cubic_bezier_derivative(from, ctrl1, ctrl2, to, 0.0)
        }
        EdgeRouteKind::Straight => Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0)),
        EdgeRouteKind::Step => {
            let mid_x = 0.5 * (from.x.0 + to.x.0);
            let p1 = Point::new(Px(mid_x), from.y);
            let p2 = Point::new(Px(mid_x), to.y);
            let d0 = Point::new(Px(p1.x.0 - from.x.0), Px(p1.y.0 - from.y.0));
            if (d0.x.0 * d0.x.0 + d0.y.0 * d0.y.0) > 1.0e-12 {
                return d0;
            }
            let d1 = Point::new(Px(p2.x.0 - p1.x.0), Px(p2.y.0 - p1.y.0));
            if (d1.x.0 * d1.x.0 + d1.y.0 * d1.y.0) > 1.0e-12 {
                return d1;
            }
            Point::new(Px(to.x.0 - p2.x.0), Px(to.y.0 - p2.y.0))
        }
    }
}

pub(crate) fn edge_route_end_tangent(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
) -> Point {
    match route {
        EdgeRouteKind::Bezier => {
            let (ctrl1, ctrl2) = wire_ctrl_points(from, to, zoom);
            cubic_bezier_derivative(from, ctrl1, ctrl2, to, 1.0)
        }
        EdgeRouteKind::Straight => Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0)),
        EdgeRouteKind::Step => {
            let mid_x = 0.5 * (from.x.0 + to.x.0);
            let p1 = Point::new(Px(mid_x), from.y);
            let p2 = Point::new(Px(mid_x), to.y);
            let d2 = Point::new(Px(to.x.0 - p2.x.0), Px(to.y.0 - p2.y.0));
            if (d2.x.0 * d2.x.0 + d2.y.0 * d2.y.0) > 1.0e-12 {
                return d2;
            }
            let d1 = Point::new(Px(p2.x.0 - p1.x.0), Px(p2.y.0 - p1.y.0));
            if (d1.x.0 * d1.x.0 + d1.y.0 * d1.y.0) > 1.0e-12 {
                return d1;
            }
            Point::new(Px(p1.x.0 - from.x.0), Px(p1.y.0 - from.y.0))
        }
    }
}
