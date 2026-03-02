pub const SOURCE: &str = include_str!("p.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::typography::p(
        cx,
        "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.",
    )
}
// endregion: example
