pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Button")
        .variant(shadcn::ButtonVariant::Outline)
        .test_id("ui-gallery-button-usage")
        .into_element(cx)
}
// endregion: example
