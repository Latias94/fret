pub const SOURCE: &str = include_str!("lead.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::lead(
        "A modal dialog that interrupts the user with important content and expects a response.",
    )
    .into_element(cx)
}
// endregion: example
