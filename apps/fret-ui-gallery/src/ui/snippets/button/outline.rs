pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Outline")
        .variant(shadcn::ButtonVariant::Outline)
        .test_id("ui-gallery-button-outline")
        .into_element(cx)
}
// endregion: example
