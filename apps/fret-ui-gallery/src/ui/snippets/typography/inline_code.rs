pub const SOURCE: &str = include_str!("inline_code.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::inline_code("@radix-ui/react-alert-dialog").into_element(cx)
}
// endregion: example
