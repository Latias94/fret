pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Button 1").into(),
        shadcn::Button::new("Button 2").into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-usage")
}
// endregion: example
