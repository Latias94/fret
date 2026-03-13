pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::h_row(|cx| {
        vec![
            shadcn::toggle_uncontrolled(cx, false, |cx| ui::children![cx; ui::text("Disabled")])
                .disabled(true)
                .a11y_label("Toggle disabled")
                .into_element(cx),
            shadcn::toggle_uncontrolled(cx, false, |cx| ui::children![cx; ui::text("Disabled")])
                .disabled(true)
                .variant(shadcn::ToggleVariant::Outline)
                .a11y_label("Toggle disabled outline")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-toggle-disabled")
}
// endregion: example
