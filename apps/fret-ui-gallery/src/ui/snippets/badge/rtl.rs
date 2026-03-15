pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_icons::IconId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    // Upstream: `apps/v4/examples/radix/badge-rtl.tsx`.
    //
    // Note: upstream uses a language selector + translations; in the gallery we keep this snippet
    // copy/paste-friendly and focus on RTL layout direction + glyph shaping.
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        row(|cx| {
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
        .into_element(cx)
        .test_id("ui-gallery-badge-rtl")
    })
}
// endregion: example
