// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let small = shadcn::Spinner::new()
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes-small");
    let large = shadcn::Spinner::new()
        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-sizes-large");

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N6)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0().max_w(Px(520.0))),
        |_cx| vec![small, large],
    )
    .test_id("ui-gallery-spinner-sizes")
}

// endregion: example

