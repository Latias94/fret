use super::*;

#[test]
fn debug_draw_list_records_concave_poly_fill_command() {
    let mut list = ImUiDebugDrawList::default();
    let points = [
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(18.0), Px(0.0)),
        Point::new(Px(10.0), Px(8.0)),
        Point::new(Px(18.0), Px(16.0)),
        Point::new(Px(0.0), Px(16.0)),
    ];

    list.add_concave_poly_filled(points, Color::from_srgb_hex_rgb(0xff_ff_ff));

    let DebugDrawCommand::Linear(DebugDrawLinearCommand::ConcavePolyFilled {
        points: recorded,
        ..
    }) = &list.commands[0]
    else {
        panic!("concave polygon fill should record a dedicated command");
    };
    assert_eq!(&**recorded, &points);
}
