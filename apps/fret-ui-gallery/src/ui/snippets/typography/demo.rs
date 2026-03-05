pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
            vec![
                shadcn::typography::h1(cx, "Taxing Laughter: The Joke Tax Chronicles"),
                shadcn::typography::lead(
                    cx,
                    "Once upon a time, in a far-off land, there was a very lazy king who spent all day lounging on his throne.",
                ),
                shadcn::typography::h2(cx, "The King's Plan"),
                shadcn::typography::p(
                    cx,
                    "The king thought long and hard, and finally came up with a brilliant plan: he would tax the jokes in the kingdom.",
                ),
                shadcn::typography::blockquote(
                    cx,
                    "After all, everyone enjoys a good joke, so it's only fair that they should pay for the privilege.",
                ),
                shadcn::typography::h3(cx, "The Joke Tax")
                    .test_id("ui-gallery-typography-demo-h3-joke-tax"),
                shadcn::typography::list(
                    cx,
                    [
                        Arc::<str>::from("1st level of puns: 5 gold coins"),
                        Arc::<str>::from("2nd level of jokes: 10 gold coins"),
                        Arc::<str>::from("3rd level of one-liners: 20 gold coins"),
                    ],
                )
                .test_id("ui-gallery-typography-demo-list-joke-tax"),
            ]
        })
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()).into_element(cx)
    .test_id("ui-gallery-typography-demo")
}
// endregion: example
