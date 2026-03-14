pub const SOURCE: &str = include_str!("inline_code.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::inline_code("cargo run -p fret-ui-gallery").into_element(cx)
}
// endregion: example
