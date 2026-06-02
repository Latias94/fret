use super::*;

#[test]
fn imui_control_chrome_layout_props_keep_dense_defaults() {
    let row = fill_row_props(MainAlign::SpaceBetween);
    assert_eq!(row.direction, Axis::Horizontal);
    assert_eq!(row.layout.size.width, Length::Fill);
    assert_eq!(row.gap, SpacingLength::Px(super::super::ROW_GAP));
    assert_eq!(row.justify, MainAlign::SpaceBetween);
    assert_eq!(row.align, CrossAlign::Center);

    let centered = centered_row_props();
    assert_eq!(centered.direction, Axis::Horizontal);
    assert_eq!(centered.gap, SpacingLength::Px(super::super::ROW_GAP));
    assert_eq!(centered.justify, MainAlign::Center);
    assert_eq!(centered.align, CrossAlign::Center);

    let stack = fill_stack_props();
    assert_eq!(stack.direction, Axis::Vertical);
    assert_eq!(stack.layout.size.width, Length::Fill);
    assert_eq!(stack.gap, SpacingLength::Px(super::super::STACK_GAP));
    assert_eq!(stack.align, CrossAlign::Stretch);
}
