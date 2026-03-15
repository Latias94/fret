pub const SOURCE: &str = include_str!("p.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::p(
        "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.",
    ).into_element(cx)
}
// endregion: example
