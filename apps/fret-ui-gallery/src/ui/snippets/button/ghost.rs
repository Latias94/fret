pub const SOURCE: &str = include_str!("ghost.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Ghost")
        .variant(shadcn::ButtonVariant::Ghost)
        .test_id("ui-gallery-button-ghost")
        .into_element(cx)
}
// endregion: example
