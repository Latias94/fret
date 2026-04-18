pub const SOURCE: &str = include_str!("muted.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::muted("Enter your email address.").into_element(cx)
}
// endregion: example
