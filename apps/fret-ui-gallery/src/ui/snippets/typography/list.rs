pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::list([
        Arc::<str>::from("Jokes are free speech."),
        Arc::<str>::from("Laughter improves morale."),
        Arc::<str>::from("Taxes should be fair."),
    ])
    .into_element(cx)
    .test_id("ui-gallery-typography-list")
}
// endregion: example
