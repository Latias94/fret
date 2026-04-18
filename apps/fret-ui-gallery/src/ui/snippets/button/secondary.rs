pub const SOURCE: &str = include_str!("secondary.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Secondary")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-button-secondary")
        .into_element(cx)
}
// endregion: example
