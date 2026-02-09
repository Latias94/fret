use super::*;

#[test]
fn web_vs_fret_calendar_22_mobile_tiny_viewport_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_with_tol(
        "calendar-22.vp375x240",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Panel,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        2.0,
        build_shadcn_calendar_22_page,
    );
}
#[test]
fn web_vs_fret_calendar_22_mobile_tiny_viewport_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_with_tol(
        "calendar-22.vp375x240",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Panel,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        2.0,
        build_shadcn_calendar_22_page,
    );
}
#[test]
fn web_vs_fret_calendar_23_mobile_tiny_viewport_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_with_tol(
        "calendar-23.vp375x240",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Panel,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        2.0,
        build_shadcn_calendar_23_page,
    );
}
#[test]
fn web_vs_fret_calendar_23_mobile_tiny_viewport_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_with_tol(
        "calendar-23.vp375x240",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Panel,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        2.0,
        build_shadcn_calendar_23_page,
    );
}
