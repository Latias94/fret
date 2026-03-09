pub const SOURCE: &str = include_str!("demo.rs");

// region: example
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
    // Upstream: `apps/v4/examples/base/badge-demo.tsx`.
    row(cx, |cx| {
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
    })
    .test_id("ui-gallery-badge-demo")
}
// endregion: example
