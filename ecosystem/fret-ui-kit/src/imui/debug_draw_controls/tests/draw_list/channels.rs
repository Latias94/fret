use super::*;

#[test]
fn debug_draw_channels_merge_in_channel_order() {
    let mut list = ImUiDebugDrawList::default();
    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(1.0), Px(1.0)),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Px(1.0),
    );

    list.channels_split(3);
    list.channels_set_current(2);
    list.add_text(
        Point::new(Px(8.0), Px(8.0)),
        "foreground",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.channels_set_current(1);
    list.add_rect_filled(
        Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(4.0), Px(4.0))),
        Color::from_srgb_hex_rgb(0x00_ff_00),
    );
    list.channels_set_current(0);
    list.add_circle_filled(
        Point::new(Px(6.0), Px(6.0)),
        Px(2.0),
        Color::from_srgb_hex_rgb(0x00_00_ff),
    );

    assert_eq!(list.command_count(), 4);
    list.channels_merge();

    assert_eq!(list.command_count(), 4);
    assert!(matches!(
        list.commands[0],
        DebugDrawCommand::Linear(DebugDrawLinearCommand::Line { .. })
    ));
    assert!(matches!(
        list.commands[1],
        DebugDrawCommand::CircleFilled { .. }
    ));
    assert!(matches!(
        list.commands[2],
        DebugDrawCommand::Linear(DebugDrawLinearCommand::RectFilled { .. })
    ));
    assert!(matches!(list.commands[3], DebugDrawCommand::Text { .. }));
}

#[test]
fn debug_draw_channels_ignore_invalid_channel_switches() {
    let mut list = ImUiDebugDrawList::default();
    list.channels_split(2);
    list.channels_set_current(4);
    list.add_text(
        Point::new(Px(0.0), Px(0.0)),
        "still-channel-zero",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.channels_merge();

    assert_eq!(list.command_count(), 1);
    assert!(matches!(list.commands[0], DebugDrawCommand::Text { .. }));
}
