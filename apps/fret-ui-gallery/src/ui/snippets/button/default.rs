pub const SOURCE: &str = include_str!("default.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Button")
        .test_id("ui-gallery-button-default")
        .into_element(cx)
}
// endregion: example
