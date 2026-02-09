use super::*;

#[test]
fn web_vs_fret_tooltip_panel_chrome_matches() {
    assert_hover_overlay_chrome_matches(
        "tooltip-demo",
        "tooltip-content",
        SemanticsRole::Tooltip,
        "Hover",
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_demo_panel_size_matches_web() {
    assert_hover_overlay_panel_size_matches_by_portal_slot_theme(
        "tooltip-demo",
        "tooltip-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_demo_panel_size_matches_web_dark() {
    assert_hover_overlay_panel_size_matches_by_portal_slot_theme(
        "tooltip-demo",
        "tooltip-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_demo_tiny_viewport_panel_size_matches_web() {
    assert_hover_overlay_panel_size_matches_by_portal_slot_theme(
        "tooltip-demo.vp1440x240",
        "tooltip-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_demo_tiny_viewport_panel_size_matches_web_dark() {
    assert_hover_overlay_panel_size_matches_by_portal_slot_theme(
        "tooltip-demo.vp1440x240",
        "tooltip-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_demo_mobile_tiny_viewport_panel_height_matches_web() {
    assert_hover_overlay_panel_height_matches_by_portal_slot_theme(
        "tooltip-demo.vp375x240",
        "tooltip-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_demo_mobile_tiny_viewport_panel_height_matches_web_dark() {
    assert_hover_overlay_panel_height_matches_by_portal_slot_theme(
        "tooltip-demo.vp375x240",
        "tooltip-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_surface_colors_match_web() {
    assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
        "tooltip-demo",
        "tooltip-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_tooltip_surface_colors_match_web_dark() {
    assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
        "tooltip-demo",
        "tooltip-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Tooltip,
        "Hover",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, trigger| {
            let trigger_el = fret_ui_shadcn::Button::new("Hover")
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .into_element(cx);
            trigger.set(Some(trigger_el.id));

            let content_el =
                fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx);

            fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
                .open_delay_frames(0)
                .close_delay_frames(0)
                .into_element(cx)
        },
    );
}
