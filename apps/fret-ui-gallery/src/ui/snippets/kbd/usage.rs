pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Kbd::new("Ctrl")
        .into_element(cx)
        .test_id("ui-gallery-kbd-usage")
}
// endregion: example
