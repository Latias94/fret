pub const SOURCE: &str = include_str!("sizes.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let size_small = cx.local_model_keyed("size_small", || false);
    let size_default = cx.local_model_keyed("size_default", || true);

    let small = ui::h_row(|cx| {
        vec![
            shadcn::Switch::new(size_small)
                .a11y_label("Small switch")
                .size(shadcn::SwitchSize::Sm)
                .test_id("ui-gallery-switch-size-small")
                .into_element(cx),
            shadcn::Label::new("Small").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-switch-sizes-sm");

    let default = ui::h_row(|cx| {
        vec![
            shadcn::Switch::new(size_default)
                .a11y_label("Default switch")
                .test_id("ui-gallery-switch-size-default")
                .into_element(cx),
            shadcn::Label::new("Default").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-switch-sizes-default");

    ui::h_flex(|_cx| vec![small, default])
        .gap(Space::N4)
        .items_center()
        .layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(520.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-switch-sizes")
}

// endregion: example
