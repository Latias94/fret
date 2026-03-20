pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Kbd::new("Ctrl")
        .into_element(cx)
        .test_id("ui-gallery-kbd-usage")
}
// endregion: example
