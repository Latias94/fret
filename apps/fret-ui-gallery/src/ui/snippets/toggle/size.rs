pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        vec![
            shadcn::toggle_uncontrolled(cx, false, |cx| ui::children![cx; ui::text("Small")])
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Sm)
                .a11y_label("Toggle small")
                .into_element(cx),
            shadcn::toggle_uncontrolled(cx, false, |cx| ui::children![cx; ui::text("Default")])
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Default)
                .a11y_label("Toggle default")
                .into_element(cx),
            shadcn::toggle_uncontrolled(cx, false, |cx| ui::children![cx; ui::text("Large")])
                .variant(shadcn::ToggleVariant::Outline)
                .size(shadcn::ToggleSize::Lg)
                .a11y_label("Toggle large")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-size")
}
// endregion: example
