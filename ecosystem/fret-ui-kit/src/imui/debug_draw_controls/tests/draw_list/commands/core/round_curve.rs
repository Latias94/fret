use super::*;

pub(super) const ROUND_CURVE_COMMAND_COUNT: usize = 8;

pub(super) fn add_round_curve_commands(list: &mut ImUiDebugDrawList) {
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
}

pub(super) fn assert_round_curve_command_order(commands: &[DebugDrawCommand], offset: usize) {
    assert!(matches!(commands[offset], DebugDrawCommand::Circle { .. }));
    assert!(matches!(
        commands[offset + 1],
        DebugDrawCommand::CircleFilled { .. }
    ));
    assert!(matches!(
        commands[offset + 2],
        DebugDrawCommand::Ngon { .. }
    ));
    assert!(matches!(
        commands[offset + 3],
        DebugDrawCommand::NgonFilled { .. }
    ));
    assert!(matches!(
        commands[offset + 4],
        DebugDrawCommand::Ellipse { .. }
    ));
    assert!(matches!(
        commands[offset + 5],
        DebugDrawCommand::EllipseFilled { .. }
    ));
    assert!(matches!(
        commands[offset + 6],
        DebugDrawCommand::BezierQuadratic { .. }
    ));
    assert!(matches!(
        commands[offset + 7],
        DebugDrawCommand::BezierCubic { .. }
    ));
}

#[test]
fn debug_draw_list_records_round_and_curve_commands_in_order() {
    let mut list = ImUiDebugDrawList::default();
    add_round_curve_commands(&mut list);

    assert_eq!(list.command_count(), ROUND_CURVE_COMMAND_COUNT);
    assert_round_curve_command_order(&list.commands, 0);
}
