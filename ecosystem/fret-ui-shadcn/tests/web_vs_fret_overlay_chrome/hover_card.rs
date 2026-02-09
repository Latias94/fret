use super::*;

#[test]
fn web_vs_fret_hover_card_panel_chrome_matches() {
    assert_overlay_chrome_matches_by_portal_slot(
        "hover-card-demo",
        "hover-card-content",
        |cx, open| {
            let trigger_el = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .into_element(cx);
            let content_el =
                fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")]).into_element(cx);

            fret_ui_shadcn::HoverCard::new(trigger_el, content_el)
                .open(Some(open.clone()))
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_hover_card_demo_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_size_only(
        "hover-card-demo",
        "hover-card-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_hover_card_demo_page,
    );
}
#[test]
fn web_vs_fret_hover_card_demo_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_size_only(
        "hover-card-demo",
        "hover-card-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_hover_card_demo_page,
    );
}
#[test]
fn web_vs_fret_hover_card_demo_tiny_viewport_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_size_only(
        "hover-card-demo.vp1440x240",
        "hover-card-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_hover_card_demo_page,
    );
}
#[test]
fn web_vs_fret_hover_card_demo_tiny_viewport_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme_size_only(
        "hover-card-demo.vp1440x240",
        "hover-card-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_hover_card_demo_page,
    );
}
#[test]
fn web_vs_fret_hover_card_surface_colors_match_web() {
    assert_overlay_chrome_matches_by_portal_slot_theme(
        "hover-card-demo",
        "hover-card-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            let trigger_el = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .into_element(cx);
            let content_el =
                fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")]).into_element(cx);

            fret_ui_shadcn::HoverCard::new(trigger_el, content_el)
                .open(Some(open.clone()))
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_hover_card_surface_colors_match_web_dark() {
    assert_overlay_chrome_matches_by_portal_slot_theme(
        "hover-card-demo",
        "hover-card-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            let trigger_el = fret_ui_shadcn::Button::new("@nextjs")
                .variant(fret_ui_shadcn::ButtonVariant::Link)
                .into_element(cx);
            let content_el =
                fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")]).into_element(cx);

            fret_ui_shadcn::HoverCard::new(trigger_el, content_el)
                .open(Some(open.clone()))
                .into_element(cx)
        },
    );
}
