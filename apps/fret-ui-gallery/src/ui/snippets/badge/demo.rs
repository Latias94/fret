pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::window::ColorScheme;
use fret_ui::Invalidation;
use fret_ui::ThemeNamedColorKey;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    // Upstream: `registry/new-york-v4/examples/badge-demo.tsx`.
    let row1 = row(cx, |cx| {
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
        ColorScheme::Dark => fret_ui_kit::colors::linear_from_hex_rgb(0x25_63_EB),
        // Tailwind: `bg-blue-500`.
        ColorScheme::Light => fret_ui_kit::colors::linear_from_hex_rgb(0x3B_82_F6),
    };

    let row2 = row(cx, |cx| {
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
                .label_font_monospace()
                .label_tabular_nums()
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .px(Space::N1),
                )
                .refine_layout(LayoutRefinement::default().h_px(Px(20.0)).min_w(Px(20.0)))
                .test_id("ui-gallery-badge-demo-count")
                .into_element(cx),
            shadcn::Badge::new("99")
                .variant(shadcn::BadgeVariant::Destructive)
                .label_font_monospace()
                .label_tabular_nums()
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .px(Space::N1),
                )
                .refine_layout(LayoutRefinement::default().h_px(Px(20.0)).min_w(Px(20.0)))
                .test_id("ui-gallery-badge-demo-count-destructive")
                .into_element(cx),
            shadcn::Badge::new("20+")
                .variant(shadcn::BadgeVariant::Outline)
                .label_font_monospace()
                .label_tabular_nums()
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .px(Space::N1),
                )
                .refine_layout(LayoutRefinement::default().h_px(Px(20.0)).min_w(Px(20.0)))
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
}
// endregion: example
