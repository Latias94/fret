pub const SOURCE: &str = include_str!("accessibility.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Button 1").into(),
        shadcn::Button::new("Button 2").into(),
    ])
    .a11y_label("Button group")
    .into_element(cx)
    .test_id("ui-gallery-button-group-accessibility")
}
// endregion: example
