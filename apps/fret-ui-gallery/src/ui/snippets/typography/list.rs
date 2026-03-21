pub const SOURCE: &str = include_str!("list.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::typography::list([
        "1st level of puns: 5 gold coins",
        "2nd level of jokes: 10 gold coins",
        "3rd level of one-liners : 20 gold coins",
    ])
    .into_element(cx)
    .test_id("ui-gallery-typography-list")
}
// endregion: example
