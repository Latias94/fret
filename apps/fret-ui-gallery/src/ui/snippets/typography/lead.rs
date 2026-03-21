pub const SOURCE: &str = include_str!("lead.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::lead(
        "A modal dialog that interrupts the user with important content and expects a response.",
    )
    .into_element(cx)
}
// endregion: example
