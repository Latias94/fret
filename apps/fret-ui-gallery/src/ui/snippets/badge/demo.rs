pub const SOURCE: &str = include_str!("demo.rs");

// region: example
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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    // Upstream: `apps/v4/examples/base/badge-demo.tsx`.
    row(|cx| {
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
    .into_element(cx)
    .test_id("ui-gallery-badge-demo")
}
// endregion: example
