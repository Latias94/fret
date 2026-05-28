use fret_core::{PathCommand, Point, Px, Rect};

pub(in crate::imui::debug_draw_controls) fn rect_path(rect: Rect) -> [PathCommand; 5] {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = Px(rect.origin.x.0 + rect.size.width.0);
    let y1 = Px(rect.origin.y.0 + rect.size.height.0);
    [
        PathCommand::MoveTo(Point::new(x0, y0)),
        PathCommand::LineTo(Point::new(x1, y0)),
        PathCommand::LineTo(Point::new(x1, y1)),
        PathCommand::LineTo(Point::new(x0, y1)),
        PathCommand::Close,
    ]
}
