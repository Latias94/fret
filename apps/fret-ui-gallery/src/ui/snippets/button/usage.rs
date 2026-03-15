pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Button")
        .variant(shadcn::ButtonVariant::Outline)
        .test_id("ui-gallery-button-usage")
        .into_element(cx)
}
// endregion: example
