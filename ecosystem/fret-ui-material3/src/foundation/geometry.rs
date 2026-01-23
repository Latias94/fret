use fret_core::{Point, Px, Rect};
use fret_ui::action::PointerDownCx;

pub fn down_origin_local(bounds: Rect, down: Option<PointerDownCx>) -> Point {
    let pos = down.map(|d| d.position).unwrap_or_else(|| {
        Point::new(
            Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
            Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
        )
    });
    Point::new(
        Px(pos.x.0 - bounds.origin.x.0),
        Px(pos.y.0 - bounds.origin.y.0),
    )
}

pub fn ripple_max_radius(bounds: Rect, origin_local: Point) -> Px {
    let w = bounds.size.width.0.max(0.0);
    let h = bounds.size.height.0.max(0.0);
    let ox = origin_local.x.0.clamp(0.0, w);
    let oy = origin_local.y.0.clamp(0.0, h);
    let corners = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)];
    let mut max: f32 = 0.0;
    for (cx, cy) in corners {
        let dx = cx - ox;
        let dy = cy - oy;
        max = max.max((dx * dx + dy * dy).sqrt());
    }
    Px(max)
}
