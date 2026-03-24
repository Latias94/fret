pub const SOURCE: &str = include_str!("tags.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::extras::Tags::new(["Alpha", "Beta", "Gamma", "A much longer tag label", "Zeta"])
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-tags")
}
// endregion: example
