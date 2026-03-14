pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ButtonGroup::new([
        shadcn::Button::new("Button 1").into(),
        shadcn::Button::new("Button 2").into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-usage")
}
// endregion: example
