use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo_story = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
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
        },
    )
    .test_id("ui-gallery-typography-demo");
    let demo = demo_story;

    let h1_sample = shadcn::typography::h1(cx, "The Joke Tax Chronicles");
    let h1 = h1_sample;

    let h2_sample = shadcn::typography::h2(cx, "People stopped telling jokes");
    let h2 = h2_sample;

    let h3_sample = shadcn::typography::h3(cx, "Jokester's Revolt");
    let h3 = h3_sample;

    let h4_sample = shadcn::typography::h4(cx, "The People's Rebellion");
    let h4 = h4_sample;

    let p_sample = shadcn::typography::p(
        cx,
        "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.",
    );
    let p = p_sample;

    let blockquote_sample =
        shadcn::typography::blockquote(cx, "Never underestimate the power of a good laugh.");
    let blockquote = blockquote_sample;

    let table_example = shadcn::Table::new(vec![
        shadcn::TableHeader::new(vec![
            shadcn::TableRow::new(
                2,
                vec![
                    shadcn::TableHead::new("King's Treasury").into_element(cx),
                    shadcn::TableHead::new("People's Happiness").into_element(cx),
                ],
            )
            .border_bottom(true)
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::TableBody::new(vec![
            shadcn::TableRow::new(
                2,
                vec![
                    shadcn::TableCell::new(cx.text("Empty")).into_element(cx),
                    shadcn::TableCell::new(cx.text("Overflowing")).into_element(cx),
                ],
            )
            .into_element(cx),
            shadcn::TableRow::new(
                2,
                vec![
                    shadcn::TableCell::new(cx.text("Modest")).into_element(cx),
                    shadcn::TableCell::new(cx.text("Satisfied")).into_element(cx),
                ],
            )
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-typography-table");
    let table = table_example;

    let list_example = shadcn::typography::list(
        cx,
        [
            Arc::<str>::from("Jokes are free speech."),
            Arc::<str>::from("Laughter improves morale."),
            Arc::<str>::from("Taxes should be fair."),
        ],
    )
    .test_id("ui-gallery-typography-list");
    let list = list_example;

    let inline_code_sample = shadcn::typography::inline_code(cx, "cargo run -p fret-ui-gallery");
    let inline_code = inline_code_sample;

    let lead_sample = shadcn::typography::lead(cx, "A larger lead paragraph introduces a section.");
    let lead = lead_sample;

    let large_sample = shadcn::typography::large(cx, "A large text block for emphasis.");
    let large = large_sample;

    let small_sample = shadcn::typography::small(cx, "Use small for helper text and metadata.");
    let small = small_sample;

    let muted_sample =
        shadcn::typography::muted(cx, "Muted text is suitable for non-primary explanations.");
    let muted = muted_sample;

    let rtl_story = doc_layout::rtl(cx, |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |cx| {
                vec![
                    shadcn::typography::h3(cx, "RTL Sample"),
                    shadcn::typography::p(
                        cx,
                        "This block validates right-to-left direction in typography surfaces.",
                    ),
                    shadcn::typography::muted(
                        cx,
                        "Check paragraph wrapping and heading alignment under RTL.",
                    ),
                ]
            },
        )
    })
    .test_id("ui-gallery-typography-rtl");
    let rtl = rtl_story;

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/typography.rs` and `ecosystem/fret-ui-shadcn/src/table.rs`.",
            "Typography in shadcn is utility-driven; keep heading hierarchy semantic and consistent.",
            "Use `lead` for intros, `muted` for hints, and avoid overusing large text in dense panels.",
            "For long-form content, combine typography helpers with table/list blocks for readability.",
            "Validate RTL and narrow viewport wrapping before shipping document-like surfaces.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Typography page follows shadcn docs order and shows one focused sample per section."),
        vec![
            DocSection::new("Demo", demo)
                .description("A long-form story sample combining headings, paragraphs, and lists.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"stack::vstack(
    cx,
    stack::VStackProps::default().gap(Space::N3).items_start(),
    |cx| {
        vec![
            shadcn::typography::h1(cx, "Taxing Laughter: The Joke Tax Chronicles"),
            shadcn::typography::lead(cx, "Once upon a time..."),
            shadcn::typography::h2(cx, "The King's Plan"),
            shadcn::typography::p(cx, "The king thought long and hard..."),
            shadcn::typography::blockquote(cx, "After all, everyone enjoys a good joke..."),
            shadcn::typography::h3(cx, "The Joke Tax"),
            shadcn::typography::list(
                cx,
                [
                    Arc::<str>::from("1st level of puns: 5 gold coins"),
                    Arc::<str>::from("2nd level of jokes: 10 gold coins"),
                    Arc::<str>::from("3rd level of one-liners: 20 gold coins"),
                ],
            ),
        ]
    },
)
.into_element(cx);"#,
                ),
            DocSection::new("h1", h1)
                .description("Top-level heading.")
                .max_w(Px(760.0))
                .code("rust", r#"shadcn::typography::h1(cx, "The Joke Tax Chronicles");"#),
            DocSection::new("h2", h2)
                .description("Section heading.")
                .max_w(Px(760.0))
                .code("rust", r#"shadcn::typography::h2(cx, "People stopped telling jokes");"#),
            DocSection::new("h3", h3)
                .description("Sub-section heading.")
                .max_w(Px(760.0))
                .code("rust", r#"shadcn::typography::h3(cx, "Jokester's Revolt");"#),
            DocSection::new("h4", h4)
                .description("Low-level heading for grouped content.")
                .max_w(Px(760.0))
                .code("rust", r#"shadcn::typography::h4(cx, "The People's Rebellion");"#),
            DocSection::new("p", p)
                .description("Body paragraph text.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::p(cx, "The king, seeing how much happier...");"#,
                ),
            DocSection::new("blockquote", blockquote)
                .description("Quoted callout text.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::blockquote(cx, "Never underestimate the power of a good laugh.");"#,
                ),
            DocSection::new("table", table)
                .description("Tabular content using shadcn Table parts.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Table::new(vec![
    shadcn::TableHeader::new(vec![
        shadcn::TableRow::new(
            2,
            vec![
                shadcn::TableHead::new("King's Treasury").into_element(cx),
                shadcn::TableHead::new("People's Happiness").into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx),
    ])
    .into_element(cx),
    shadcn::TableBody::new(vec![
        shadcn::TableRow::new(
            2,
            vec![
                shadcn::TableCell::new(cx.text("Empty")).into_element(cx),
                shadcn::TableCell::new(cx.text("Overflowing")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("list", list)
                .description("Bulleted/ordered list content.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::list(
    cx,
    [
        Arc::<str>::from("Jokes are free speech."),
        Arc::<str>::from("Laughter improves morale."),
        Arc::<str>::from("Taxes should be fair."),
    ],
);"#,
                ),
            DocSection::new("Inline Code", inline_code)
                .description("Inline code styling for commands and identifiers.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::inline_code(cx, "cargo run -p fret-ui-gallery");"#,
                ),
            DocSection::new("Lead", lead)
                .description("Intro lead paragraph for sections.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::lead(cx, "A larger lead paragraph introduces a section.");"#,
                ),
            DocSection::new("Large", large)
                .description("Emphasis text for short callouts.")
                .max_w(Px(760.0))
                .code("rust", r#"shadcn::typography::large(cx, "A large text block for emphasis.");"#),
            DocSection::new("Small", small)
                .description("Helper text and metadata.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::small(cx, "Use small for helper text and metadata.");"#,
                ),
            DocSection::new("Muted", muted)
                .description("De-emphasized hint/explanation text.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::typography::muted(cx, "Muted text is suitable for non-primary explanations.");"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Direction provider sample to validate RTL wrapping/alignment.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |cx| {
                vec![
                    shadcn::typography::h3(cx, "RTL Sample"),
                    shadcn::typography::p(cx, "This block validates right-to-left direction..."),
                    shadcn::typography::muted(cx, "Check wrapping and alignment under RTL."),
                ]
            },
        )
    },
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-typography")]
}
