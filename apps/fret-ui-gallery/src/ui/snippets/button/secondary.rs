pub const SOURCE: &str = include_str!("secondary.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Button::new("Secondary")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-button-secondary")
        .into_element(cx)
}
// endregion: example
