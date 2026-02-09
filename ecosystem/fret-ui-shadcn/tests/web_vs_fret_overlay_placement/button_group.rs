use super::*;

#[test]
fn web_vs_fret_button_group_demo_menu_item_height_matches() {
    assert_button_group_demo_constrained_menu_item_height_matches("button-group-demo");
    assert_button_group_demo_constrained_menu_item_height_matches("button-group-demo.vp375x240");
}
#[test]
fn web_vs_fret_button_group_demo_menu_content_insets_match() {
    assert_button_group_demo_constrained_menu_content_insets_match("button-group-demo");
    assert_button_group_demo_constrained_menu_content_insets_match("button-group-demo.vp375x240");
}
#[test]
fn web_vs_fret_button_group_demo_submenu_overlay_placement_matches() {
    assert_button_group_demo_submenu_overlay_placement_matches("button-group-demo.submenu-kbd");
}
#[test]
fn web_vs_fret_button_group_demo_submenu_menu_content_insets_match() {
    assert_button_group_demo_submenu_constrained_menu_content_insets_match(
        "button-group-demo.submenu-kbd",
    );
}
#[test]
fn web_vs_fret_button_group_demo_submenu_menu_item_height_matches() {
    assert_button_group_demo_submenu_menu_item_height_matches("button-group-demo.submenu-kbd");
}
