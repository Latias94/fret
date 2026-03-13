pub const SOURCE: &str = include_str!("ghost.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Ghost")
        .variant(shadcn::ButtonVariant::Ghost)
        .test_id("ui-gallery-button-ghost")
        .into_element(cx)
}
// endregion: example
