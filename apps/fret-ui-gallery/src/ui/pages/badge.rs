use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use fret_core::window::ColorScheme;
use fret_ui::Invalidation;
use fret_ui::ThemeNamedColorKey;

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let demo = {
        // Upstream: `registry/new-york-v4/examples/badge-demo.tsx`.
        let row1 = doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Badge")
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
            ]
        });

        let scheme = cx.environment_color_scheme(Invalidation::Paint);
        let blue = match scheme.unwrap_or(ColorScheme::Light) {
            // Tailwind: `bg-blue-600`.
            ColorScheme::Dark => CoreColor {
                r: 0x25 as f32 / 255.0,
                g: 0x63 as f32 / 255.0,
                b: 0xEB as f32 / 255.0,
                a: 1.0,
            },
            // Tailwind: `bg-blue-500`.
            ColorScheme::Light => CoreColor {
                r: 0x3B as f32 / 255.0,
                g: 0x82 as f32 / 255.0,
                b: 0xF6 as f32 / 255.0,
                a: 1.0,
            },
        };

        let row2 = doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Verified")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .leading_icon(fret_icons::IconId::new_static("lucide.badge-check"))
                    .refine_style(
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(blue))
                            .text_color(ColorRef::Named(ThemeNamedColorKey::White)),
                    )
                    .test_id("ui-gallery-badge-demo-verified")
                    .into_element(cx),
                shadcn::Badge::new("8")
                    .label_tabular_nums()
                    .refine_style(
                        ChromeRefinement::default()
                            .rounded(Radius::Full)
                            .px(Space::N1),
                    )
                    .refine_layout(LayoutRefinement::default().min_w(Px(20.0)))
                    .test_id("ui-gallery-badge-demo-count")
                    .into_element(cx),
                shadcn::Badge::new("99")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .label_tabular_nums()
                    .refine_style(
                        ChromeRefinement::default()
                            .rounded(Radius::Full)
                            .px(Space::N1),
                    )
                    .refine_layout(LayoutRefinement::default().min_w(Px(20.0)))
                    .test_id("ui-gallery-badge-demo-count-destructive")
                    .into_element(cx),
                shadcn::Badge::new("20+")
                    .variant(shadcn::BadgeVariant::Outline)
                    .label_tabular_nums()
                    .refine_style(
                        ChromeRefinement::default()
                            .rounded(Radius::Full)
                            .px(Space::N1),
                    )
                    .refine_layout(LayoutRefinement::default().min_w(Px(20.0)))
                    .test_id("ui-gallery-badge-demo-count-outline")
                    .into_element(cx),
            ]
        });

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![row1, row2],
        )
        .test_id("ui-gallery-badge-demo")
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

    let notes = doc_layout::notes(
        cx,
        [
            "Badge is a small status/label primitive; prefer concise text and keep contrast high.",
            "API reference: `ecosystem/fret-ui-shadcn/src/badge.rs`.",
            "Note: the Link render example installs a no-op `on_activate` so diag scripts do not launch a system browser; remove it to enable the default `Effect::OpenUrl` fallback.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Badge docs (demo + asChild-style link render).",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Default shadcn badge variants and common compositions.")
                .code(
                    "rust",
                    r#"shadcn::Badge::new("Badge").into_element(cx);
shadcn::Badge::new("Secondary")
    .variant(shadcn::BadgeVariant::Secondary)
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
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-badge")]
}
