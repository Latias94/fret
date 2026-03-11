use fret_core::Point;

pub(super) fn distance2(start: Point, position: Point) -> f32 {
    let dx = position.x.0 - start.x.0;
    let dy = position.y.0 - start.y.0;
    dx * dx + dy * dy
}
