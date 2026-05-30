use super::{
    enum_select_close_auto_focus_policy, enum_select_viewport_test_id,
    rect_visible_within_viewport_y,
};
use fret_core::{Point, Px, Rect, Size};
use fret_ui_kit::primitives::combobox::{
    ComboboxCloseAutoFocusDecision, ComboboxCloseAutoFocusPolicy,
};

#[test]
fn enum_select_close_focus_policy_matches_trigger_owned_combobox() {
    let policy: ComboboxCloseAutoFocusPolicy = enum_select_close_auto_focus_policy();

    assert_eq!(
        policy.on_item_press,
        ComboboxCloseAutoFocusDecision::RestoreTrigger
    );
    assert_eq!(
        policy.on_escape,
        ComboboxCloseAutoFocusDecision::RestoreTrigger
    );
    assert_eq!(
        policy.on_trigger_press,
        ComboboxCloseAutoFocusDecision::RestoreTrigger
    );
    assert_eq!(
        policy.on_outside_press,
        ComboboxCloseAutoFocusDecision::RestoreTrigger
    );
    assert_eq!(
        policy.on_focus_out,
        ComboboxCloseAutoFocusDecision::PreventDefault
    );
}

#[test]
fn enum_select_viewport_test_id_suffixes_list_test_id() {
    assert_eq!(
        enum_select_viewport_test_id("editor.enum.list").as_ref(),
        "editor.enum.list.viewport"
    );
}

#[test]
fn rect_visible_within_viewport_y_matches_nearest_visibility_contract() {
    let viewport = Rect::new(Point::new(Px(0.0), Px(10.0)), Size::new(Px(40.0), Px(40.0)));

    let fully_visible = Rect::new(Point::new(Px(0.0), Px(20.0)), Size::new(Px(40.0), Px(12.0)));
    assert!(rect_visible_within_viewport_y(viewport, fully_visible));

    let clipped_bottom = Rect::new(Point::new(Px(0.0), Px(42.0)), Size::new(Px(40.0), Px(16.0)));
    assert!(!rect_visible_within_viewport_y(viewport, clipped_bottom));

    let tall_child = Rect::new(Point::new(Px(0.0), Px(10.0)), Size::new(Px(40.0), Px(60.0)));
    assert!(rect_visible_within_viewport_y(viewport, tall_child));

    let tall_child_top_hidden =
        Rect::new(Point::new(Px(0.0), Px(4.0)), Size::new(Px(40.0), Px(60.0)));
    assert!(!rect_visible_within_viewport_y(
        viewport,
        tall_child_top_hidden
    ));
}
