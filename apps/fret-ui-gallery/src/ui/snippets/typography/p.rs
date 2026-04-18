pub const SOURCE: &str = include_str!("p.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::p(
        "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.",
    ).into_element(cx)
}
// endregion: example
