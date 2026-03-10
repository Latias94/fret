pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Destructive")
        .variant(shadcn::ButtonVariant::Destructive)
        .test_id("ui-gallery-button-destructive")
        .into_element(cx)
}
// endregion: example
