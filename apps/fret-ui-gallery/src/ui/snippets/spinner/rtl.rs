// region: example
use fret_core::Px;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Item::new([
            shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
            shadcn::ItemContent::new([shadcn::ItemTitle::new("Processing payment...").into_element(cx)])
                .into_element(cx),
            shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Muted)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-spinner-rtl")
}

// endregion: example

