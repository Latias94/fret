pub const SOURCE: &str = include_str!("muted.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::raw::typography::muted("Muted text is suitable for non-primary explanations.")
        .into_element(cx)
}
// endregion: example
