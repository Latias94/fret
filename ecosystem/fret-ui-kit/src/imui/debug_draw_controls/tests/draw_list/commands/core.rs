use super::*;

#[test]
fn debug_draw_list_records_commands_in_order() {
    let mut list = ImUiDebugDrawList::default();
    assert!(list.is_empty());

    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(10.0), Px(10.0)),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Px(1.0),
    );
    list.add_polyline(
        [
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(4.0), Px(8.0)),
            Point::new(Px(8.0), Px(2.0)),
        ],
        Color::from_srgb_hex_rgb(0xff_ff_00),
        Px(1.0),
        false,
    );
    list.add_convex_poly_filled(
        [
            Point::new(Px(12.0), Px(62.0)),
            Point::new(Px(24.0), Px(54.0)),
            Point::new(Px(36.0), Px(62.0)),
            Point::new(Px(30.0), Px(76.0)),
            Point::new(Px(18.0), Px(76.0)),
        ],
        Color::from_srgb_hex_rgb(0x10_b9_81),
    );
    list.add_rect(
        Rect::new(Point::new(Px(2.0), Px(3.0)), Size::new(Px(4.0), Px(5.0))),
        Color::from_srgb_hex_rgb(0x00_ff_00),
        Px(2.0),
    );
    list.add_rect_filled(
        Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(2.0), Px(2.0))),
        Color::from_srgb_hex_rgb(0x00_00_ff),
    );
    list.add_rect_filled_multi_color(
        Rect::new(Point::new(Px(4.0), Px(1.0)), Size::new(Px(6.0), Px(5.0))),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Color::from_srgb_hex_rgb(0x00_ff_00),
        Color::from_srgb_hex_rgb(0x00_00_ff),
        Color::from_srgb_hex_rgb(0xff_ff_00),
    );
    list.add_quad(
        Point::new(Px(8.0), Px(8.0)),
        Point::new(Px(18.0), Px(6.0)),
        Point::new(Px(22.0), Px(18.0)),
        Point::new(Px(10.0), Px(20.0)),
        Color::from_srgb_hex_rgb(0xfb_71_85),
        Px(1.0),
    );
    list.add_quad_filled(
        Point::new(Px(24.0), Px(8.0)),
        Point::new(Px(34.0), Px(6.0)),
        Point::new(Px(38.0), Px(18.0)),
        Point::new(Px(26.0), Px(20.0)),
        Color::from_srgb_hex_rgb(0x2d_d4_bf),
    );
    list.add_triangle(
        Point::new(Px(1.0), Px(1.0)),
        Point::new(Px(5.0), Px(1.0)),
        Point::new(Px(3.0), Px(4.0)),
        Color::from_srgb_hex_rgb(0xff_00_ff),
        Px(1.0),
    );
    list.add_triangle_filled(
        Point::new(Px(2.0), Px(2.0)),
        Point::new(Px(6.0), Px(2.0)),
        Point::new(Px(4.0), Px(5.0)),
        Color::from_srgb_hex_rgb(0x00_ff_ff),
    );
    list.add_circle(
        Point::new(Px(20.0), Px(20.0)),
        Px(8.0),
        Color::from_srgb_hex_rgb(0xff_aa_00),
        Px(2.0),
    );
    list.add_circle_filled(
        Point::new(Px(40.0), Px(20.0)),
        Px(6.0),
        Color::from_srgb_hex_rgb(0xaa_00_ff),
    );
    list.add_ngon(
        Point::new(Px(56.0), Px(20.0)),
        Px(8.0),
        5,
        Color::from_srgb_hex_rgb(0x65_a3_ff),
        Px(1.0),
    );
    list.add_ngon_filled(
        Point::new(Px(76.0), Px(20.0)),
        Px(6.0),
        6,
        Color::from_srgb_hex_rgb(0xc0_84_fc),
    );
    list.add_ellipse(
        Point::new(Px(96.0), Px(20.0)),
        Size::new(Px(12.0), Px(6.0)),
        0.25,
        16,
        Color::from_srgb_hex_rgb(0x38_bd_f8),
        Px(1.0),
    );
    list.add_ellipse_filled(
        Point::new(Px(122.0), Px(20.0)),
        Size::new(Px(10.0), Px(5.0)),
        0.5,
        0,
        Color::from_srgb_hex_rgb(0xf0_ab_fc),
    );
    list.add_bezier_quadratic(
        Point::new(Px(2.0), Px(60.0)),
        Point::new(Px(20.0), Px(42.0)),
        Point::new(Px(38.0), Px(60.0)),
        Color::from_srgb_hex_rgb(0x22_d3_ee),
        Px(1.0),
    );
    list.add_bezier_cubic(
        Point::new(Px(42.0), Px(60.0)),
        Point::new(Px(54.0), Px(42.0)),
        Point::new(Px(70.0), Px(78.0)),
        Point::new(Px(82.0), Px(60.0)),
        Color::from_srgb_hex_rgb(0xf4_72_b6),
        Px(1.0),
    );
    list.add_text(
        Point::new(Px(4.0), Px(5.0)),
        "debug",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );

    assert_eq!(list.command_count(), 19);
    assert!(matches!(list.commands[0], DebugDrawCommand::Line { .. }));
    assert!(matches!(
        list.commands[1],
        DebugDrawCommand::Polyline { .. }
    ));
    assert!(matches!(
        list.commands[2],
        DebugDrawCommand::ConvexPolyFilled { .. }
    ));
    assert!(matches!(list.commands[3], DebugDrawCommand::Rect { .. }));
    assert!(matches!(
        list.commands[4],
        DebugDrawCommand::RectFilled { .. }
    ));
    assert!(matches!(
        list.commands[5],
        DebugDrawCommand::RectFilledMultiColor { .. }
    ));
    assert!(matches!(list.commands[6], DebugDrawCommand::Quad { .. }));
    assert!(matches!(
        list.commands[7],
        DebugDrawCommand::QuadFilled { .. }
    ));
    assert!(matches!(
        list.commands[8],
        DebugDrawCommand::Triangle { .. }
    ));
    assert!(matches!(
        list.commands[9],
        DebugDrawCommand::TriangleFilled { .. }
    ));
    assert!(matches!(list.commands[10], DebugDrawCommand::Circle { .. }));
    assert!(matches!(
        list.commands[11],
        DebugDrawCommand::CircleFilled { .. }
    ));
    assert!(matches!(list.commands[12], DebugDrawCommand::Ngon { .. }));
    assert!(matches!(
        list.commands[13],
        DebugDrawCommand::NgonFilled { .. }
    ));
    assert!(matches!(
        list.commands[14],
        DebugDrawCommand::Ellipse { .. }
    ));
    assert!(matches!(
        list.commands[15],
        DebugDrawCommand::EllipseFilled { .. }
    ));
    assert!(matches!(
        list.commands[16],
        DebugDrawCommand::BezierQuadratic { .. }
    ));
    assert!(matches!(
        list.commands[17],
        DebugDrawCommand::BezierCubic { .. }
    ));
    assert!(matches!(list.commands[18], DebugDrawCommand::Text { .. }));
}
