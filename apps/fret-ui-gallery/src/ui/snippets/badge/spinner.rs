pub const SOURCE: &str = include_str!("spinner.rs");

// region: example
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
    // Upstream: `apps/v4/examples/radix/badge-spinner.tsx`.
    row(cx, |cx| {
        vec![
            shadcn::Badge::new("Deleting")
                .variant(shadcn::BadgeVariant::Destructive)
                .children([shadcn::Spinner::new().into_element(cx)])
                .test_id("ui-gallery-badge-spinner-deleting")
                .into_element(cx),
            shadcn::Badge::new("Generating")
                .variant(shadcn::BadgeVariant::Secondary)
                .trailing_children([shadcn::Spinner::new().into_element(cx)])
                .test_id("ui-gallery-badge-spinner-generating")
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-badge-spinner")
}
// endregion: example
