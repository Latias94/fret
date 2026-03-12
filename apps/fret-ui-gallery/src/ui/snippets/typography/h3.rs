pub const SOURCE: &str = include_str!("h3.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::h3(cx, "Jokester's Revolt")
}
// endregion: example
