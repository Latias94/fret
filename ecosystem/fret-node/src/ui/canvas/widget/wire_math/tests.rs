use super::*;
use fret_core::PathCommand;

#[test]
fn path_distance2_on_line_is_zeroish() {
    let commands = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
    ];

    let p = Point::new(Px(5.0), Px(0.0));
    let d2 = wire_distance2_path(p, &commands, 8);
    assert!(d2.is_finite() && d2 <= 1.0e-6);
}

#[test]
fn path_midpoint_and_normal_is_finite() {
    let commands = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
    ];

    let (mid, normal) = path_midpoint_and_normal(&commands, 8).expect("midpoint exists");
    assert!((mid.x.0 - 5.0).abs() <= 1.0e-3);
    assert!(mid.y.0.abs() <= 1.0e-3);
    assert!(normal.x.0.is_finite() && normal.y.0.is_finite());
}

#[test]
fn path_start_end_tangents_follow_control_points() {
    let from = Point::new(Px(0.0), Px(0.0));
    let ctrl1 = Point::new(Px(5.0), Px(1.0));
    let ctrl2 = Point::new(Px(6.0), Px(2.0));
    let to = Point::new(Px(10.0), Px(0.0));

    let commands = [
        PathCommand::MoveTo(from),
        PathCommand::CubicTo { ctrl1, ctrl2, to },
    ];

    let (t0, t1) = path_start_end_tangents(&commands).expect("tangents exist");
    assert_eq!(
        t0,
        Point::new(Px(ctrl1.x.0 - from.x.0), Px(ctrl1.y.0 - from.y.0))
    );
    assert_eq!(
        t1,
        Point::new(Px(to.x.0 - ctrl2.x.0), Px(to.y.0 - ctrl2.y.0))
    );
}
