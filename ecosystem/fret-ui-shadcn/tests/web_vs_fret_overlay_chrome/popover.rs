use super::*;

#[test]
fn web_vs_fret_popover_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    assert_overlay_surface_colors_match(
        "popover-demo",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open popover")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(320.0))
                                .h_px(Px(245.33334)),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_popover_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    assert_overlay_surface_colors_match(
        "popover-demo",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            Popover::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open popover")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    PopoverContent::new(Vec::new())
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(320.0))
                                .h_px(Px(245.33334)),
                        )
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_popover_panel_chrome_matches() {
    assert_overlay_chrome_matches(
        "popover-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            fret_ui_shadcn::Popover::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Open").into_element(cx),
                |cx| fret_ui_shadcn::PopoverContent::new(Vec::new()).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_popover_demo_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "popover-demo",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_popover_demo_page,
    );
}
#[test]
fn web_vs_fret_popover_demo_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "popover-demo",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_popover_demo_page,
    );
}
#[test]
fn web_vs_fret_popover_demo_tiny_viewport_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "popover-demo.vp1440x240",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_popover_demo_page,
    );
}
#[test]
fn web_vs_fret_popover_demo_tiny_viewport_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "popover-demo.vp1440x240",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_popover_demo_page,
    );
}
#[test]
fn web_vs_fret_popover_demo_mobile_tiny_viewport_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "popover-demo.vp375x240",
        "popover-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_popover_demo_page,
    );
}
#[test]
fn web_vs_fret_popover_demo_mobile_tiny_viewport_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "popover-demo.vp375x240",
        "popover-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_popover_demo_page,
    );
}
