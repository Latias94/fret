use super::*;

#[test]
fn web_vs_fret_mode_toggle_menu_item_height_matches() {
    assert_mode_toggle_constrained_menu_item_height_matches("mode-toggle");
    assert_mode_toggle_constrained_menu_item_height_matches("mode-toggle.vp375x240");
}
#[test]
fn web_vs_fret_mode_toggle_menu_content_insets_match() {
    assert_mode_toggle_constrained_menu_content_insets_match("mode-toggle");
    assert_mode_toggle_constrained_menu_content_insets_match("mode-toggle.vp375x240");
}
