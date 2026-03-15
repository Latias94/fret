pub const SOURCE: &str = include_str!("secondary.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Secondary")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-button-secondary")
        .into_element(cx)
}
// endregion: example
