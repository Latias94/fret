use super::*;

#[test]
fn tabs_trigger_pointer_down_selects_on_left_mouse_down() {
    let action = tabs_trigger_pointer_down_action(
        PointerType::Mouse,
        MouseButton::Left,
        Modifiers::default(),
        false,
    );
    assert_eq!(action, TabsTriggerPointerDownAction::Select);
}

#[test]
fn tabs_trigger_pointer_down_prevents_focus_on_ctrl_click() {
    let mut modifiers = Modifiers::default();
    modifiers.ctrl = true;

    let action =
        tabs_trigger_pointer_down_action(PointerType::Mouse, MouseButton::Left, modifiers, false);
    assert_eq!(action, TabsTriggerPointerDownAction::PreventFocus);
}

#[test]
fn tabs_trigger_pointer_down_ignores_touch_to_preserve_click_like_activation() {
    let action = tabs_trigger_pointer_down_action(
        PointerType::Touch,
        MouseButton::Left,
        Modifiers::default(),
        false,
    );
    assert_eq!(action, TabsTriggerPointerDownAction::Ignore);
}
