pub const SOURCE: &str = include_str!("h1.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::h1("The Joke Tax Chronicles").into_element(cx)
}
// endregion: example
