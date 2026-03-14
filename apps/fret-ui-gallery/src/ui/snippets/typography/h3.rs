pub const SOURCE: &str = include_str!("h3.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::h3("Jokester's Revolt").into_element(cx)
}
// endregion: example
