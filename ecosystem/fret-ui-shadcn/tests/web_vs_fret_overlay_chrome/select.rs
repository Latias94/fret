use super::*;

#[test]
fn web_vs_fret_select_panel_chrome_matches() {
    assert_overlay_chrome_matches(
        "select-scrollable",
        "listbox",
        SemanticsRole::ListBox,
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_select_scrollable_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
            fret_ui_shadcn::Select::new(value, open.clone())
                .a11y_label("Select")
                .item(fret_ui_shadcn::SelectItem::new("one", "One"))
                .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_select_scrollable_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "select-scrollable",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "select-scrollable",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_small_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x450",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_surface_colors_match_web() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_surface_colors_match_web_dark() {
    assert_overlay_surface_colors_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_shadow_matches_web() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_scrollable_tiny_viewport_shadow_matches_web_dark() {
    assert_overlay_shadow_insets_match(
        "select-scrollable.vp1440x240",
        "select-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::ListBox,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_select_scrollable_demo,
    );
}
#[test]
fn web_vs_fret_select_demo_highlighted_option_chrome_matches_web() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-demo.highlight-first",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_demo_page,
    );
}
#[test]
fn web_vs_fret_select_demo_highlighted_option_chrome_matches_web_dark() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-demo.highlight-first",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_demo_page,
    );
}
#[test]
fn web_vs_fret_select_demo_highlighted_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-demo.highlight-first-vp375x240",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_demo_page,
    );
}
#[test]
fn web_vs_fret_select_demo_highlighted_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-demo.highlight-first-vp375x240",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_demo_page,
    );
}
#[test]
fn web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-scrollable.highlight-first",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_scrollable_page,
    );
}
#[test]
fn web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web_dark() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-scrollable.highlight-first",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_scrollable_page,
    );
}
#[test]
fn web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-scrollable.highlight-first-vp375x240",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_scrollable_page,
    );
}
#[test]
fn web_vs_fret_select_scrollable_highlighted_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_highlighted_option_chrome_matches_web(
        "select-scrollable.highlight-first-vp375x240",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_scrollable_page,
    );
}
#[test]
fn web_vs_fret_select_demo_focused_option_chrome_matches_web() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-demo.focus-first",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_demo_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_demo_focused_option_chrome_matches_web_dark() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-demo.focus-first",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_demo_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_demo_focused_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-demo.focus-first-vp375x240",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_demo_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_demo_focused_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-demo.focus-first-vp375x240",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_demo_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_scrollable_focused_option_chrome_matches_web() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-scrollable.focus-first",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_scrollable_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_scrollable_focused_option_chrome_matches_web_dark() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-scrollable.focus-first",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_scrollable_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_scrollable_focused_option_chrome_matches_web_mobile_tiny_viewport() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-scrollable.focus-first-vp375x240",
        "light",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        build_shadcn_select_scrollable_page,
        "Select",
    );
}
#[test]
fn web_vs_fret_select_scrollable_focused_option_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_listbox_focused_option_chrome_matches_web(
        "select-scrollable.focus-first-vp375x240",
        "dark",
        "select-item",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        build_shadcn_select_scrollable_page,
        "Select",
    );
}
