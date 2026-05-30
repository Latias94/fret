use super::*;

#[test]
fn bezier_paths_use_native_quad_and_cubic_commands() {
    let from = Point::new(Px(0.0), Px(0.0));
    let ctrl = Point::new(Px(10.0), Px(20.0));
    let ctrl1 = Point::new(Px(8.0), Px(16.0));
    let ctrl2 = Point::new(Px(18.0), Px(-6.0));
    let to = Point::new(Px(24.0), Px(0.0));

    assert_eq!(
        bezier_quadratic_path(from, ctrl, to),
        [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
    );
    assert_eq!(
        bezier_cubic_path(from, ctrl1, ctrl2, to),
        [
            PathCommand::MoveTo(from),
            PathCommand::CubicTo { ctrl1, ctrl2, to },
        ]
    );
}
