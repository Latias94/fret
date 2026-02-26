use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use fret_ui::ThemeNamedColorKey;

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let variants = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Default")
                    .test_id("ui-gallery-badge-variant-default")
                    .into_element(cx),
                shadcn::Badge::new("Secondary")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id("ui-gallery-badge-variant-secondary")
                    .into_element(cx),
                shadcn::Badge::new("Destructive")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .test_id("ui-gallery-badge-variant-destructive")
                    .into_element(cx),
                shadcn::Badge::new("Outline")
                    .variant(shadcn::BadgeVariant::Outline)
                    .test_id("ui-gallery-badge-variant-outline")
                    .into_element(cx),
                shadcn::Badge::new("Ghost")
                    .variant(shadcn::BadgeVariant::Ghost)
                    .test_id("ui-gallery-badge-variant-ghost")
                    .into_element(cx),
                shadcn::Badge::new("Link")
                    .variant(shadcn::BadgeVariant::Link)
                    .test_id("ui-gallery-badge-variant-link")
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-variants")
    };

    let with_icon = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Verified")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .leading_icon(fret_icons::IconId::new_static("lucide.badge-check"))
                    .into_element(cx),
                shadcn::Badge::new("Bookmark")
                    .variant(shadcn::BadgeVariant::Outline)
                    .leading_icon(fret_icons::IconId::new_static("lucide.bookmark"))
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-with-icon")
    };

    let with_spinner = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Deleting")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .test_id("ui-gallery-badge-spinner-destructive")
                    .children([shadcn::Spinner::new().into_element(cx)])
                    .into_element(cx),
                shadcn::Badge::new("Generating")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id("ui-gallery-badge-spinner-secondary")
                    .children([shadcn::Spinner::new().into_element(cx)])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-with-spinner")
    };

    let link = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Open Link")
                    .variant(shadcn::BadgeVariant::Link)
                    .render(shadcn::BadgeRender::Link {
                        href: Arc::from("https://example.com"),
                        target: None,
                        rel: None,
                    })
                    // Avoid launching the system browser during diag runs; the render surface
                    // still applies link semantics and Enter-only activation.
                    .on_activate(Arc::new(|_host, _acx, _reason| {}))
                    .test_id("ui-gallery-badge-link")
                    .trailing_icon(fret_icons::IconId::new_static("lucide.arrow-right"))
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-link-row")
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
                            .text_color(ColorRef::Named(ThemeNamedColorKey::Black))
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
                            .text_color(ColorRef::Named(ThemeNamedColorKey::Black))
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
                            .text_color(ColorRef::Named(ThemeNamedColorKey::Black))
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
                            .text_color(ColorRef::Named(ThemeNamedColorKey::Black))
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
                            .text_color(ColorRef::Named(ThemeNamedColorKey::Black))
                            .merge(border_transparent.clone()),
                    )
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-custom-colors")
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("شارة").into_element(cx),
                shadcn::Badge::new("ثانوي")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new("متحقق")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .leading_icon(fret_icons::IconId::new_static("lucide.badge-check"))
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
            "Note: the Link render example installs a no-op `on_activate` so diag scripts do not launch a system browser; remove it to enable the default `Effect::OpenUrl` fallback.",
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
    .leading_icon(fret_icons::IconId::new_static("lucide.badge-check"))
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
    .variant(shadcn::BadgeVariant::Link)
    .render(shadcn::BadgeRender::Link { href: Arc::from("https://example.com"), target: None, rel: None })
    .on_activate(Arc::new(|_host, _acx, _reason| {})) // optional; remove to open the URL
    .trailing_icon(fret_icons::IconId::new_static("lucide.arrow-right"))
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
