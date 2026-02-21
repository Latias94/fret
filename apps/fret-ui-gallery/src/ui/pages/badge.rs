use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let icon = |cx: &mut ElementContext<'_, App>, id: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static(id),
            Some(Px(12.0)),
            Some(fg),
        )
    };

    let variants = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Default").into_element(cx),
                shadcn::Badge::new("Secondary")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new("Destructive")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .into_element(cx),
                shadcn::Badge::new("Outline")
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-variants")
    };

    let with_icon = {
        let secondary_fg = ColorRef::Color(theme.color_token("secondary-foreground"));
        let outline_fg = ColorRef::Color(theme.color_token("foreground"));

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Verified")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .children([icon(cx, "lucide.badge-check", secondary_fg.clone())])
                    .into_element(cx),
                shadcn::Badge::new("Bookmark")
                    .variant(shadcn::BadgeVariant::Outline)
                    .children([icon(cx, "lucide.bookmark", outline_fg.clone())])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-with-icon")
    };

    let with_spinner = {
        let destructive_fg = ColorRef::Color(theme.color_token("destructive-foreground"));
        let secondary_fg = ColorRef::Color(theme.color_token("secondary-foreground"));

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Deleting")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .children([shadcn::Spinner::new()
                        .color(destructive_fg.clone())
                        .into_element(cx)])
                    .into_element(cx),
                shadcn::Badge::new("Generating")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .children([shadcn::Spinner::new()
                        .color(secondary_fg.clone())
                        .into_element(cx)])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-with-spinner")
    };

    let link = {
        let outline_fg = ColorRef::Color(theme.color_token("foreground"));

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Open Link")
                    .variant(shadcn::BadgeVariant::Outline)
                    .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-link")
    };

    let custom_colors = {
        let border_transparent =
            ChromeRefinement::default().border_color(ColorRef::Color(CoreColor::TRANSPARENT));

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Blue")
                    .variant(shadcn::BadgeVariant::Outline)
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(CoreColor {
                                r: 0.90,
                                g: 0.95,
                                b: 1.00,
                                a: 1.0,
                            }))
                            .merge(border_transparent.clone()),
                    )
                    .into_element(cx),
                shadcn::Badge::new("Green")
                    .variant(shadcn::BadgeVariant::Outline)
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(CoreColor {
                                r: 0.91,
                                g: 0.98,
                                b: 0.91,
                                a: 1.0,
                            }))
                            .merge(border_transparent.clone()),
                    )
                    .into_element(cx),
                shadcn::Badge::new("Sky")
                    .variant(shadcn::BadgeVariant::Outline)
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(CoreColor {
                                r: 0.90,
                                g: 0.97,
                                b: 1.00,
                                a: 1.0,
                            }))
                            .merge(border_transparent.clone()),
                    )
                    .into_element(cx),
                shadcn::Badge::new("Purple")
                    .variant(shadcn::BadgeVariant::Outline)
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(CoreColor {
                                r: 0.95,
                                g: 0.92,
                                b: 1.00,
                                a: 1.0,
                            }))
                            .merge(border_transparent.clone()),
                    )
                    .into_element(cx),
                shadcn::Badge::new("Red")
                    .variant(shadcn::BadgeVariant::Outline)
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(CoreColor {
                                r: 1.00,
                                g: 0.92,
                                b: 0.92,
                                a: 1.0,
                            }))
                            .merge(border_transparent.clone()),
                    )
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-custom-colors")
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            let secondary_fg = ColorRef::Color(theme.color_token("secondary-foreground"));
            vec![
                shadcn::Badge::new("شارة").into_element(cx),
                shadcn::Badge::new("ثانوي")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new("متحقق")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .children([icon(cx, "lucide.badge-check", secondary_fg.clone())])
                    .into_element(cx),
            ]
        })
    })
    .test_id("ui-gallery-badge-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "Badge is a small status/label primitive; prefer concise text and keep contrast high.",
            "API reference: `ecosystem/fret-ui-shadcn/src/badge.rs`.",
            "If you customize colors, verify hover/focus states and token-driven variants stay consistent.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Badge docs order: Variants, With Icon, With Spinner, Link, Custom Colors, RTL.",
        ),
        vec![
            DocSection::new("Variants", variants)
                .description("Default shadcn badge variants.")
                .code(
                    "rust",
                    r#"for (label, variant) in [
    ("Default", shadcn::BadgeVariant::Default),
    ("Secondary", shadcn::BadgeVariant::Secondary),
    ("Destructive", shadcn::BadgeVariant::Destructive),
    ("Outline", shadcn::BadgeVariant::Outline),
] {
    shadcn::Badge::new(label).variant(variant).into_element(cx);
}"#,
                ),
            DocSection::new("With Icon", with_icon)
                .description("Compose an icon into the badge content.")
                .code(
                    "rust",
                    r#"shadcn::Badge::new("Verified")
    .variant(shadcn::BadgeVariant::Secondary)
    .children([shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.badge-check"))])
    .into_element(cx);"#,
                ),
            DocSection::new("With Spinner", with_spinner)
                .description("Use a spinner to indicate in-flight actions.")
                .code(
                    "rust",
                    r#"shadcn::Badge::new("Generating")
    .variant(shadcn::BadgeVariant::Secondary)
    .children([shadcn::Spinner::new().into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Link", link)
                .description("Badges can be styled as outlines and composed with link affordances.")
                .code(
                    "rust",
                    r#"shadcn::Badge::new("Open Link")
    .variant(shadcn::BadgeVariant::Outline)
    .children([shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.arrow-up-right"))])
    .into_element(cx);"#,
                ),
            DocSection::new("Custom Colors", custom_colors)
                .description("Refine style for one-off color badges.")
                .code(
                    "rust",
                    r#"shadcn::Badge::new("Blue")
    .variant(shadcn::BadgeVariant::Outline)
    .refine_style(
        ChromeRefinement::default()
            .bg(ColorRef::Color(CoreColor { r: 0.90, g: 0.95, b: 1.00, a: 1.0 }))
            .border_color(ColorRef::Color(CoreColor::TRANSPARENT)),
    )
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Badge layout under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::Badge::new("شارة").into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-badge")]
}
