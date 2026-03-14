pub const SOURCE: &str = include_str!("muted.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::muted("Muted text is suitable for non-primary explanations.")
        .into_element(cx)
}
// endregion: example
