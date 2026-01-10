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

pub fn intersect_rect(a: Rect, b: Rect) -> Rect {
    let a_left = a.origin.x.0;
    let a_top = a.origin.y.0;
    let a_right = a_left + a.size.width.0.max(0.0);
    let a_bottom = a_top + a.size.height.0.max(0.0);

    let b_left = b.origin.x.0;
    let b_top = b.origin.y.0;
    let b_right = b_left + b.size.width.0.max(0.0);
    let b_bottom = b_top + b.size.height.0.max(0.0);

    let left = a_left.max(b_left);
    let top = a_top.max(b_top);
    let right = a_right.min(b_right);
    let bottom = a_bottom.min(b_bottom);

    Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px((right - left).max(0.0)), Px((bottom - top).max(0.0))),
    )
}
