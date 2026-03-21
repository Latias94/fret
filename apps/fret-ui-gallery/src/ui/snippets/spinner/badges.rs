pub const SOURCE: &str = include_str!("badges.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let spinner = |cx: &mut UiCx<'_>| shadcn::Spinner::new().into_element(cx);

    ui::h_flex(|cx| {
        vec![
            shadcn::Badge::new("Syncing")
                .leading_children([spinner(cx)])
                .into_element(cx)
                .test_id("ui-gallery-spinner-badge"),
            shadcn::Badge::new("Updating")
                .variant(shadcn::BadgeVariant::Secondary)
                .leading_children([spinner(cx)])
                .into_element(cx),
            shadcn::Badge::new("Processing")
                .variant(shadcn::BadgeVariant::Outline)
                .leading_children([spinner(cx)])
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-spinner-badges")
}

// endregion: example
