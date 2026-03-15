pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Destructive")
        .variant(shadcn::ButtonVariant::Destructive)
        .test_id("ui-gallery-button-destructive")
        .into_element(cx)
}
// endregion: example
