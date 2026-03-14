pub const SOURCE: &str = include_str!("small.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::small("Use small for helper text and metadata.").into_element(cx)
}
// endregion: example
