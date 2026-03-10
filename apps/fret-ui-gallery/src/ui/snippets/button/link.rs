pub const SOURCE: &str = include_str!("link.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Link")
        .variant(shadcn::ButtonVariant::Link)
        .test_id("ui-gallery-button-link")
        .into_element(cx)
}
// endregion: example
