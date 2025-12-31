use fret_core::{Edges, Point, Px, Rect, Size};

pub fn inset_rect(rect: Rect, margin: Edges) -> Rect {
    let w = rect.size.width.0.max(0.0);
    let h = rect.size.height.0.max(0.0);

    let l = margin.left.0.max(0.0);
    let t = margin.top.0.max(0.0);
    let r = margin.right.0.max(0.0);
    let b = margin.bottom.0.max(0.0);

    Rect::new(
        Point::new(Px(rect.origin.x.0 + l), Px(rect.origin.y.0 + t)),
        Size::new(Px((w - l - r).max(0.0)), Px((h - t - b).max(0.0))),
    )
}
