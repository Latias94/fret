pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::h1( "Taxing Laughter: The Joke Tax Chronicles").into_element(cx),
                shadcn::raw::typography::lead(
                    "Once upon a time, in a far-off land, there was a very lazy king who spent all day lounging on his throne.",
                ).into_element(cx),
                shadcn::raw::typography::h2( "The King's Plan").into_element(cx),
                shadcn::raw::typography::p(
                    "The king thought long and hard, and finally came up with a brilliant plan: he would tax the jokes in the kingdom.",
                ).into_element(cx),
                shadcn::raw::typography::blockquote(
                    "After all, everyone enjoys a good joke, so it's only fair that they should pay for the privilege.",
                ).into_element(cx),
                shadcn::raw::typography::h3( "The Joke Tax").into_element(cx)
                    .test_id("ui-gallery-typography-demo-h3-joke-tax"),
                shadcn::raw::typography::list(
                    [
                        Arc::<str>::from("1st level of puns: 5 gold coins"),
                        Arc::<str>::from("2nd level of jokes: 10 gold coins"),
                        Arc::<str>::from("3rd level of one-liners: 20 gold coins"),
                    ],
                ).into_element(cx)
                .test_id("ui-gallery-typography-demo-list-joke-tax"),
            ]
        })
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()).into_element(cx)
    .test_id("ui-gallery-typography-demo")
}
// endregion: example
