pub const SOURCE: &str = include_str!("default.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Button")
        .test_id("ui-gallery-button-default")
        .into_element(cx)
}
// endregion: example
