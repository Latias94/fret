use super::*;

pub(super) const LINEAR_COMMAND_COUNT: usize = 10;

pub(super) fn add_linear_commands(list: &mut ImUiDebugDrawList) {
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
}

pub(super) fn assert_linear_command_order(commands: &[DebugDrawCommand], offset: usize) {
    assert!(matches!(commands[offset], DebugDrawCommand::Line { .. }));
    assert!(matches!(
        commands[offset + 1],
        DebugDrawCommand::Polyline { .. }
    ));
    assert!(matches!(
        commands[offset + 2],
        DebugDrawCommand::ConvexPolyFilled { .. }
    ));
    assert!(matches!(
        commands[offset + 3],
        DebugDrawCommand::Rect { .. }
    ));
    assert!(matches!(
        commands[offset + 4],
        DebugDrawCommand::RectFilled { .. }
    ));
    assert!(matches!(
        commands[offset + 5],
        DebugDrawCommand::RectFilledMultiColor { .. }
    ));
    assert!(matches!(
        commands[offset + 6],
        DebugDrawCommand::Quad { .. }
    ));
    assert!(matches!(
        commands[offset + 7],
        DebugDrawCommand::QuadFilled { .. }
    ));
    assert!(matches!(
        commands[offset + 8],
        DebugDrawCommand::Triangle { .. }
    ));
    assert!(matches!(
        commands[offset + 9],
        DebugDrawCommand::TriangleFilled { .. }
    ));
}

#[test]
fn debug_draw_list_records_linear_commands_in_order() {
    let mut list = ImUiDebugDrawList::default();
    add_linear_commands(&mut list);

    assert_eq!(list.command_count(), LINEAR_COMMAND_COUNT);
    assert_linear_command_order(&list.commands, 0);
}
