use fret_core::{Point, Px, Rect, Size};

mod identity;
mod layout;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    )
}
