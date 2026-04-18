pub const SOURCE: &str = include_str!("link.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Button::new("Link")
        .variant(shadcn::ButtonVariant::Link)
        .test_id("ui-gallery-button-link")
        .into_element(cx)
}
// endregion: example
