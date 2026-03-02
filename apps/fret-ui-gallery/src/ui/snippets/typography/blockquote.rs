pub const SOURCE: &str = include_str!("blockquote.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::typography::blockquote(cx, "Never underestimate the power of a good laugh.")
}
// endregion: example
