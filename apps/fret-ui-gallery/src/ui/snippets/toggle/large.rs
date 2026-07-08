pub const SOURCE: &str = include_str!("large.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::toggle_uncontrolled(
        cx,
        false,
        |cx| ui::children![cx; icon::icon(cx, IconId::new_static("lucide.italic"))],
    )
    .size(shadcn::ToggleSize::Lg)
    .a11y_label("Toggle italic")
    .into_element(cx)
    .test_id("ui-gallery-toggle-large")
}
// endregion: example
