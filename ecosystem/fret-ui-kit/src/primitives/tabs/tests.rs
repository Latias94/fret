use super::*;

use fret_core::{Point, Px, Rect, Size};

mod controllable_value;
mod semantics;
mod trigger_pointer;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    )
}
