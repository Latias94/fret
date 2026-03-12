pub const SOURCE: &str = include_str!("inline_code.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::inline_code("cargo run -p fret-ui-gallery").into_element(cx)
}
// endregion: example
