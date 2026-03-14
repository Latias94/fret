pub const SOURCE: &str = include_str!("blockquote.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::blockquote("Never underestimate the power of a good laugh.")
        .into_element(cx)
}
// endregion: example
