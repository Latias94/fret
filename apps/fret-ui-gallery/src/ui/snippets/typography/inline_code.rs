pub const SOURCE: &str = include_str!("inline_code.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::typography::inline_code(cx, "cargo run -p fret-ui-gallery")
}
// endregion: example
