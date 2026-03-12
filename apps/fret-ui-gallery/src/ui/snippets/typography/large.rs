pub const SOURCE: &str = include_str!("large.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::large("A large text block for emphasis.").into_element(cx)
}
// endregion: example
