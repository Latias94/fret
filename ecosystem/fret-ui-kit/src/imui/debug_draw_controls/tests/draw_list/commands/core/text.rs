use super::*;

pub(super) const TEXT_COMMAND_COUNT: usize = 1;

pub(super) fn add_text_command(list: &mut ImUiDebugDrawList) {
    list.add_text(
        Point::new(Px(4.0), Px(5.0)),
        "debug",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
}

pub(super) fn assert_text_command_order(commands: &[DebugDrawCommand], offset: usize) {
    assert!(matches!(commands[offset], DebugDrawCommand::Text { .. }));
}

#[test]
fn debug_draw_list_records_text_command_in_order() {
    let mut list = ImUiDebugDrawList::default();
    add_text_command(&mut list);

    assert_eq!(list.command_count(), TEXT_COMMAND_COUNT);
    assert_text_command_order(&list.commands, 0);
}
