pub const SOURCE: &str = include_str!("variants.rs");

// region: example
use fret::{AppComponentCx, UiChild};
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
        .justify_center()
        .items_center()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    // Upstream: `apps/v4/examples/radix/badge-variants.tsx`.
    row(|cx| {
        vec![
            shadcn::Badge::new("Default")
                .test_id("ui-gallery-badge-variants-default")
                .into_element(cx),
            shadcn::Badge::new("Secondary")
                .variant(shadcn::BadgeVariant::Secondary)
                .test_id("ui-gallery-badge-variants-secondary")
                .into_element(cx),
            shadcn::Badge::new("Destructive")
                .variant(shadcn::BadgeVariant::Destructive)
                .test_id("ui-gallery-badge-variants-destructive")
                .into_element(cx),
            shadcn::Badge::new("Outline")
                .variant(shadcn::BadgeVariant::Outline)
                .test_id("ui-gallery-badge-variants-outline")
                .into_element(cx),
            shadcn::Badge::new("Ghost")
                .variant(shadcn::BadgeVariant::Ghost)
                .test_id("ui-gallery-badge-variants-ghost")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-badge-variants")
}
// endregion: example
