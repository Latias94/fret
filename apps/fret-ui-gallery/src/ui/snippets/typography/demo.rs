pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::h1("Taxing Laughter: The Joke Tax Chronicles")
                .into_element(cx),
            shadcn::raw::typography::lead(
                "Once upon a time, in a far-off land, there was a very lazy king who spent all day lounging on his throne. One day, his advisors came to him with a problem: the kingdom was running out of money.",
            )
            .into_element(cx),
            shadcn::raw::typography::h2("The King's Plan").into_element(cx),
            shadcn::raw::typography::p(
                "The king thought long and hard, and finally came up with a brilliant plan: he would tax the jokes in the kingdom.",
            )
            .into_element(cx),
            shadcn::raw::typography::blockquote(
                "\"After all,\" he said, \"everyone enjoys a good joke, so it's only fair that they should pay for the privilege.\"",
            )
            .into_element(cx),
            shadcn::raw::typography::h3("The Joke Tax")
                .into_element(cx)
                .test_id("ui-gallery-typography-demo-h3-joke-tax"),
            shadcn::raw::typography::p(
                "The king's subjects were not amused. They grumbled and complained, but the king was firm:",
            )
            .into_element(cx),
            shadcn::raw::typography::list([
                "1st level of puns: 5 gold coins",
                "2nd level of jokes: 10 gold coins",
                "3rd level of one-liners : 20 gold coins",
            ])
            .into_element(cx)
                .test_id("ui-gallery-typography-demo-list-joke-tax"),
            shadcn::raw::typography::p(
                "As a result, people stopped telling jokes, and the kingdom fell into a gloom. But there was one person who refused to let the king's foolishness get him down: a court jester named Jokester.",
            )
            .into_element(cx),
            shadcn::raw::typography::h3("Jokester's Revolt").into_element(cx),
            shadcn::raw::typography::p(
                "Jokester began sneaking into the castle in the middle of the night and leaving jokes all over the place: under the king's pillow, in his soup, even in the royal toilet. The king was furious, but he couldn't seem to stop Jokester.",
            )
            .into_element(cx),
            shadcn::raw::typography::p(
                "And then, one day, the people of the kingdom discovered that the jokes left by Jokester were so funny that they couldn't help but laugh. And once they started laughing, they couldn't stop.",
            )
            .into_element(cx),
            shadcn::raw::typography::h3("The People's Rebellion").into_element(cx),
            shadcn::raw::typography::p(
                "The people of the kingdom, feeling uplifted by the laughter, started to tell jokes and puns again, and soon the entire kingdom was in on the joke.",
            )
            .into_element(cx),
            shadcn::table(|cx| {
                ui::children![
                    cx;
                    shadcn::table_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::table_row(2, |cx| {
                                ui::children![
                                    cx;
                                    shadcn::table_head("King's Treasury"),
                                    shadcn::table_head("People's happiness"),
                                ]
                            })
                            .border_bottom(true),
                        ]
                    }),
                    shadcn::table_body(|cx| {
                        vec![
                            shadcn::table_row(2, |cx| {
                                ui::children![
                                    cx;
                                    shadcn::table_cell(ui::text("Empty")),
                                    shadcn::table_cell(ui::text("Overflowing")),
                                ]
                            })
                            .into_element(cx),
                            shadcn::table_row(2, |cx| {
                                ui::children![
                                    cx;
                                    shadcn::table_cell(ui::text("Modest")),
                                    shadcn::table_cell(ui::text("Satisfied")),
                                ]
                            })
                            .into_element(cx),
                            shadcn::table_row(2, |cx| {
                                ui::children![
                                    cx;
                                    shadcn::table_cell(ui::text("Full")),
                                    shadcn::table_cell(ui::text("Ecstatic")),
                                ]
                            })
                            .into_element(cx),
                        ]
                    }),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::raw::typography::p(
                "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax. Jokester was declared a hero, and the kingdom lived happily ever after.",
            )
            .into_element(cx),
            shadcn::raw::typography::p(
                "The moral of the story is: never underestimate the power of a good laugh and always be careful of bad ideas.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-typography-demo")
}
// endregion: example
