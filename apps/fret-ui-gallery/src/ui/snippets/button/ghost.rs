pub const SOURCE: &str = include_str!("ghost.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Ghost")
        .variant(shadcn::ButtonVariant::Ghost)
        .test_id("ui-gallery-button-ghost")
        .into_element(cx)
}
// endregion: example
