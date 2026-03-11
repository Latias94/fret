pub const SOURCE: &str = include_str!("blockquote.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::blockquote(cx, "Never underestimate the power of a good laugh.")
}
// endregion: example
