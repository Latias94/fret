pub const SOURCE: &str = include_str!("h2.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::h2("The People of the Kingdom").into_element(cx)
}
// endregion: example
