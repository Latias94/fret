pub const SOURCE: &str = include_str!("h2.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::typography::h2(cx, "People stopped telling jokes")
}
// endregion: example
