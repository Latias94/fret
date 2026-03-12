pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_row(|cx| {
        vec![
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Sm)
                .a11y_label("Toggle small")
                .label("Small")
                .into_element(cx),
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Default)
                .a11y_label("Toggle default")
                .label("Default")
                .into_element(cx),
            shadcn::Toggle::uncontrolled(false)
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Lg)
                .a11y_label("Toggle large")
                .label("Large")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-size")
}
// endregion: example
