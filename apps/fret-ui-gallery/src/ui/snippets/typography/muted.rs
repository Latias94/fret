pub const SOURCE: &str = include_str!("muted.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::muted("Enter your email address.").into_element(cx)
}
// endregion: example
