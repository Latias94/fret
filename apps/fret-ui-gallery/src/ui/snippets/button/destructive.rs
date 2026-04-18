pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Destructive")
        .variant(shadcn::ButtonVariant::Destructive)
        .test_id("ui-gallery-button-destructive")
        .into_element(cx)
}
// endregion: example
