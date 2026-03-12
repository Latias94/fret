pub const SOURCE: &str = include_str!("airplane_mode.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let model = cx.local_model(|| false);

    ui::h_flex(|cx| {
        vec![
            shadcn::Switch::new(model)
                .a11y_label("Airplane mode")
                .test_id("ui-gallery-switch-airplane-toggle")
                .into_element(cx),
            shadcn::Label::new("Airplane Mode").into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(Px(520.0)),
    )
    .into_element(cx)
    .test_id("ui-gallery-switch-airplane")
}

// endregion: example
