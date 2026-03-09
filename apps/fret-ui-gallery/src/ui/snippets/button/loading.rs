pub const SOURCE: &str = include_str!("loading.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
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
    wrap_row(cx, |cx| {
        vec![
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .disabled(true)
                .test_id("ui-gallery-button-loading-submit")
                .children([
                    shadcn::Spinner::new().into_element(cx),
                    ui::text("Generating")
                        .font_medium()
                        .nowrap()
                        .into_element(cx),
                ])
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Secondary)
                .disabled(true)
                .test_id("ui-gallery-button-loading-download")
                .children([
                    ui::text("Downloading")
                        .font_medium()
                        .nowrap()
                        .into_element(cx),
                    shadcn::Spinner::new().into_element(cx),
                ])
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-button-loading")
}
// endregion: example
