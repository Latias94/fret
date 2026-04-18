pub const SOURCE: &str = include_str!("blockquote.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::blockquote(
        "\"After all,\" he said, \"everyone enjoys a good joke, so it's only fair that they should pay for the privilege.\"",
    )
    .into_element(cx)
}
// endregion: example
