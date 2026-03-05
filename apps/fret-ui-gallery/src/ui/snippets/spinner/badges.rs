pub const SOURCE: &str = include_str!("badges.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let spinner = |cx: &mut ElementContext<'_, H>| shadcn::Spinner::new().into_element(cx);

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::Badge::new("Syncing")
                    .children([spinner(cx)])
                    .into_element(cx)
                    .test_id("ui-gallery-spinner-badge"),
                shadcn::Badge::new("Updating")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Badge::new("Processing")
                    .variant(shadcn::BadgeVariant::Outline)
                    .children([spinner(cx)])
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-spinner-badges")
}

// endregion: example
