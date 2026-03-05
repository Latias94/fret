pub const SOURCE: &str = include_str!("icon.rs");

// region: example
use fret_icons::IconId;
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
    // Upstream: `apps/v4/examples/radix/badge-icon.tsx`.
    row(cx, |cx| {
        vec![
            shadcn::Badge::new("Verified")
                .variant(shadcn::BadgeVariant::Secondary)
                .leading_icon(IconId::new_static("lucide.badge-check"))
                .test_id("ui-gallery-badge-icon-verified")
                .into_element(cx),
            shadcn::Badge::new("Bookmark")
                .variant(shadcn::BadgeVariant::Outline)
                .trailing_icon(IconId::new_static("lucide.bookmark"))
                .test_id("ui-gallery-badge-icon-bookmark")
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-badge-icon")
}
// endregion: example
