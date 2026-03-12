pub const SOURCE: &str = include_str!("small.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::small("Use small for helper text and metadata.").into_element(cx)
}
// endregion: example
