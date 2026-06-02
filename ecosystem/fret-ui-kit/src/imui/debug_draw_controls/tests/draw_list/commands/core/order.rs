use super::*;

#[test]
fn debug_draw_list_records_commands_in_order() {
    let mut list = ImUiDebugDrawList::default();
    assert!(list.is_empty());

    linear::add_linear_commands(&mut list);
    round_curve::add_round_curve_commands(&mut list);
    text::add_text_command(&mut list);

    assert_eq!(list.command_count(), 19);
    linear::assert_linear_command_order(&list.commands, 0);
    round_curve::assert_round_curve_command_order(&list.commands, linear::LINEAR_COMMAND_COUNT);
    text::assert_text_command_order(
        &list.commands,
        linear::LINEAR_COMMAND_COUNT + round_curve::ROUND_CURVE_COMMAND_COUNT,
    );
}
