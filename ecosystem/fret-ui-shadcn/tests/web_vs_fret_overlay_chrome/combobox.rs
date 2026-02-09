use super::*;

#[test]
fn web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "combobox-demo.highlight-first",
        "light",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_combobox_demo_page,
    );
}
#[test]
fn web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web_dark() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "combobox-demo.highlight-first",
        "dark",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_combobox_demo_page,
    );
}
#[test]
fn web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "combobox-demo.highlight-first-vp375x240",
        "light",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_combobox_demo_page,
    );
}
#[test]
fn web_vs_fret_combobox_demo_highlighted_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "combobox-demo.highlight-first-vp375x240",
        "dark",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_combobox_demo_page,
    );
}
#[test]
fn web_vs_fret_combobox_demo_focused_option_chrome_matches_web() {
    assert_listbox_focused_option_chrome_matches_web(
        "combobox-demo.focus-first",
        "light",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_combobox_demo_page,
        "Select a fruit",
    );
}
#[test]
fn web_vs_fret_combobox_demo_focused_option_chrome_matches_web_dark() {
    assert_listbox_focused_option_chrome_matches_web(
        "combobox-demo.focus-first",
        "dark",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_combobox_demo_page,
        "Select a fruit",
    );
}
#[test]
fn web_vs_fret_combobox_demo_focused_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_focused_option_chrome_matches_web(
        "combobox-demo.focus-first-vp375x240",
        "light",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_combobox_demo_page,
        "Select a fruit",
    );
}
#[test]
fn web_vs_fret_combobox_demo_focused_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_focused_option_chrome_matches_web(
        "combobox-demo.focus-first-vp375x240",
        "dark",
        "command-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_combobox_demo_page,
        "Select a fruit",
    );
}
