use fret_core::{PathCommand, Point, Px};

pub(in crate::imui::debug_draw_controls) fn circle_path(
    center: Point,
    radius: Px,
) -> [PathCommand; 6] {
    let r = radius.0;
    let k = 0.552_284_8_f32 * r;
    let cx = center.x.0;
    let cy = center.y.0;
    [
        PathCommand::MoveTo(Point::new(Px(cx + r), Px(cy))),
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + r), Px(cy + k)),
            ctrl2: Point::new(Px(cx + k), Px(cy + r)),
            to: Point::new(Px(cx), Px(cy + r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - k), Px(cy + r)),
            ctrl2: Point::new(Px(cx - r), Px(cy + k)),
            to: Point::new(Px(cx - r), Px(cy)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - r), Px(cy - k)),
            ctrl2: Point::new(Px(cx - k), Px(cy - r)),
            to: Point::new(Px(cx), Px(cy - r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + k), Px(cy - r)),
            ctrl2: Point::new(Px(cx + r), Px(cy - k)),
            to: Point::new(Px(cx + r), Px(cy)),
        },
        PathCommand::Close,
    ]
}
