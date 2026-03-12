use super::start_size_from_rect;
use fret_core::{Point, Px, Rect, Size};

#[test]
fn start_size_from_rect_scales_canvas_rect_by_zoom() {
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(10.0)));
    let size = start_size_from_rect(rect, 2.5);
    assert_eq!(size.width, 50.0);
    assert_eq!(size.height, 25.0);
}
