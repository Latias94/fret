pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::toggle_uncontrolled(
        cx,
        false,
        |cx| ui::children![cx; decl_text::text_button_label(cx, "Toggle")],
    )
    .a11y_label("Toggle formatting")
    .into_element(cx)
    .test_id("ui-gallery-toggle-usage")
}
// endregion: example
