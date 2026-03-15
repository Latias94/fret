pub const SOURCE: &str = include_str!("h2.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::h2("People stopped telling jokes").into_element(cx)
}
// endregion: example
