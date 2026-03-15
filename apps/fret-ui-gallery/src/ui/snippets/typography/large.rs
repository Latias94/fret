pub const SOURCE: &str = include_str!("large.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::large("A large text block for emphasis.").into_element(cx)
}
// endregion: example
