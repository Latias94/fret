pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Outline")
        .variant(shadcn::ButtonVariant::Outline)
        .test_id("ui-gallery-button-outline")
        .into_element(cx)
}
// endregion: example
