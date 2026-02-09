use super::*;

#[test]
fn web_vs_fret_button_group_demo_destructive_menu_item_idle_fg_matches_web() {
    assert_button_group_demo_dropdown_menu_destructive_item_idle_fg_matches_web(
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_button_group_demo_destructive_menu_item_idle_fg_matches_web_dark() {
    assert_button_group_demo_dropdown_menu_destructive_item_idle_fg_matches_web(
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_button_group_demo_destructive_focused_item_chrome_matches_web() {
    assert_button_group_demo_dropdown_menu_destructive_focused_item_chrome_matches_web(
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_button_group_demo_destructive_focused_item_chrome_matches_web_dark() {
    assert_button_group_demo_dropdown_menu_destructive_focused_item_chrome_matches_web(
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
