// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::typography::muted(cx, "Muted text is suitable for non-primary explanations.")
}
// endregion: example

