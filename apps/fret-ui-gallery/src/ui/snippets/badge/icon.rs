pub const SOURCE: &str = include_str!("icon.rs");

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
    // Upstream: `apps/v4/examples/radix/badge-icon.tsx`.
    row(|cx| {
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
    .into_element(cx)
    .test_id("ui-gallery-badge-icon")
}
// endregion: example
