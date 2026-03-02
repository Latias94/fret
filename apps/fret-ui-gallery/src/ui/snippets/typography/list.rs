pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::typography::list(
        cx,
        [
            Arc::<str>::from("Jokes are free speech."),
            Arc::<str>::from("Laughter improves morale."),
            Arc::<str>::from("Taxes should be fair."),
        ],
    )
    .test_id("ui-gallery-typography-list")
}
// endregion: example
