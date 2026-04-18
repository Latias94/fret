pub const SOURCE: &str = include_str!("tags.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::extras::Tags::new(["Alpha", "Beta", "Gamma", "A much longer tag label", "Zeta"])
        .into_element(cx)
        .test_id("ui-gallery-shadcn-extras-tags")
}
// endregion: example
