pub const SOURCE: &str = include_str!("accessibility.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Button 1").into(),
        shadcn::Button::new("Button 2").into(),
    ])
    .a11y_label("Button group")
    .into_element(cx)
    .test_id("ui-gallery-button-group-accessibility")
}
// endregion: example
