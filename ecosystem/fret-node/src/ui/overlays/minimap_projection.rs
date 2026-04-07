use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;

pub(super) fn minimap_world_bounds<I>(rects: I, viewport: Rect, padding: f32) -> Rect
where
    I: IntoIterator<Item = Rect>,
{
    let mut out: Option<Rect> = None;
    for rect in rects {
        out = Some(match out {
            Some(prev) => rect_union(prev, rect),
            None => rect,
        });
    }

    let mut out = match out {
        Some(prev) => rect_union(prev, viewport),
        None => viewport,
    };
    out.origin.x.0 -= padding;
    out.origin.y.0 -= padding;
    out.size.width.0 += 2.0 * padding;
    out.size.height.0 += 2.0 * padding;
    out
}

pub(super) fn project_world_rect_to_minimap(minimap: Rect, world: Rect, rect: Rect) -> Rect {
    let (scale, ox, oy) = minimap_projection_frame(minimap, world);

    Rect::new(
        Point::new(
            Px(ox + rect.origin.x.0 * scale),
            Px(oy + rect.origin.y.0 * scale),
        ),
        Size::new(
            Px((rect.size.width.0 * scale).max(1.0)),
            Px((rect.size.height.0 * scale).max(1.0)),
        ),
    )
}

pub(super) fn unproject_minimap_point(minimap: Rect, world: Rect, point: Point) -> Option<Point> {
    let (scale, ox, oy) = minimap_projection_frame(minimap, world);
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }

    let x = (point.x.0 - ox) / scale;
    let y = (point.y.0 - oy) / scale;
    Some(Point::new(Px(x), Px(y)))
}

pub(super) fn pan_to_center_canvas_point(
    bounds: Rect,
    zoom: f32,
    canvas_center: Point,
) -> CanvasPoint {
    let zoom = normalized_zoom(zoom);
    let cx = 0.5 * bounds.size.width.0;
    let cy = 0.5 * bounds.size.height.0;
    CanvasPoint {
        x: cx / zoom - canvas_center.x.0,
        y: cy / zoom - canvas_center.y.0,
    }
}

fn minimap_projection_frame(minimap: Rect, world: Rect) -> (f32, f32, f32) {
    let ww = world.size.width.0.max(1.0e-6);
    let wh = world.size.height.0.max(1.0e-6);
    let sx = minimap.size.width.0 / ww;
    let sy = minimap.size.height.0 / wh;
    let scale = sx.min(sy);

    let ox = minimap.origin.x.0 + 0.5 * (minimap.size.width.0 - world.size.width.0 * scale)
        - world.origin.x.0 * scale;
    let oy = minimap.origin.y.0 + 0.5 * (minimap.size.height.0 - world.size.height.0 * scale)
        - world.origin.y.0 * scale;
    (scale, ox, oy)
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    let x0 = a.origin.x.0.min(b.origin.x.0);
    let y0 = a.origin.y.0.min(b.origin.y.0);
    let x1 = (a.origin.x.0 + a.size.width.0).max(b.origin.x.0 + b.size.width.0);
    let y1 = (a.origin.y.0 + a.size.height.0).max(b.origin.y.0 + b.size.height.0);
    Rect::new(
        Point::new(Px(x0), Px(y0)),
        Size::new(Px((x1 - x0).max(1.0)), Px((y1 - y0).max(1.0))),
    )
}

fn normalized_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        minimap_world_bounds, pan_to_center_canvas_point, project_world_rect_to_minimap,
        unproject_minimap_point,
    };
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn minimap_world_bounds_union_keeps_viewport_and_padding() {
        let viewport = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let rects = vec![
            Rect::new(
                Point::new(Px(-5.0), Px(30.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            Rect::new(
                Point::new(Px(50.0), Px(-10.0)),
                Size::new(Px(5.0), Px(10.0)),
            ),
        ];

        let world = minimap_world_bounds(rects, viewport, 4.0);
        assert_eq!(world.origin, Point::new(Px(-9.0), Px(-14.0)));
        assert_eq!(world.size, Size::new(Px(68.0), Px(78.0)));
    }

    #[test]
    fn project_and_unproject_round_trip_center_point() {
        let minimap = Rect::new(
            Point::new(Px(100.0), Px(50.0)),
            Size::new(Px(200.0), Px(120.0)),
        );
        let world = Rect::new(
            Point::new(Px(-50.0), Px(-25.0)),
            Size::new(Px(400.0), Px(200.0)),
        );
        let point = Point::new(Px(75.0), Px(60.0));
        let marker = Rect::new(point, Size::new(Px(20.0), Px(10.0)));

        let projected = project_world_rect_to_minimap(minimap, world, marker);
        let projected_center = Point::new(
            Px(projected.origin.x.0 + 0.5 * projected.size.width.0),
            Px(projected.origin.y.0 + 0.5 * projected.size.height.0),
        );
        let round_trip =
            unproject_minimap_point(minimap, world, projected_center).expect("projected point");

        assert!((round_trip.x.0 - 85.0).abs() <= 1.0e-4);
        assert!((round_trip.y.0 - 65.0).abs() <= 1.0e-4);
    }

    #[test]
    fn pan_to_center_canvas_point_normalizes_invalid_zoom() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let pan = pan_to_center_canvas_point(bounds, f32::NAN, Point::new(Px(120.0), Px(80.0)));
        assert_eq!(pan.x, 280.0);
        assert_eq!(pan.y, 220.0);
    }
}
