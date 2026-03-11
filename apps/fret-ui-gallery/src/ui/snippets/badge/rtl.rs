pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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
    // Upstream: `apps/v4/examples/radix/badge-rtl.tsx`.
    //
    // Note: upstream uses a language selector + translations; in the gallery we keep this snippet
    // copy/paste-friendly and focus on RTL layout direction + glyph shaping.
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        row(cx, |cx| {
            vec![
                shadcn::Badge::new("شارة")
                    .test_id("ui-gallery-badge-rtl-default")
                    .into_element(cx),
                shadcn::Badge::new("ثانوي")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id("ui-gallery-badge-rtl-secondary")
                    .into_element(cx),
                shadcn::Badge::new("مدمر")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .test_id("ui-gallery-badge-rtl-destructive")
                    .into_element(cx),
                shadcn::Badge::new("مخطط")
                    .variant(shadcn::BadgeVariant::Outline)
                    .test_id("ui-gallery-badge-rtl-outline")
                    .into_element(cx),
                shadcn::Badge::new("متحقق")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .leading_icon(IconId::new_static("lucide.badge-check"))
                    .test_id("ui-gallery-badge-rtl-verified")
                    .into_element(cx),
                shadcn::Badge::new("إشارة مرجعية")
                    .variant(shadcn::BadgeVariant::Outline)
                    .trailing_icon(IconId::new_static("lucide.bookmark"))
                    .test_id("ui-gallery-badge-rtl-bookmark")
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-badge-rtl")
    })
}
// endregion: example
