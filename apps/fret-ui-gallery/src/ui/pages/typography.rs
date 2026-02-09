use super::super::*;

pub(super) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(760.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

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
    .test_id("ui-gallery-typography-demo");
    let demo = section_card(cx, "Demo", demo_story);

    let h1_sample = shadcn::typography::h1(cx, "The Joke Tax Chronicles");
    let h1 = section_card(cx, "h1", h1_sample);

    let h2_sample = shadcn::typography::h2(cx, "People stopped telling jokes");
    let h2 = section_card(cx, "h2", h2_sample);

    let h3_sample = shadcn::typography::h3(cx, "Jokester's Revolt");
    let h3 = section_card(cx, "h3", h3_sample);

    let h4_sample = shadcn::typography::h4(cx, "The People's Rebellion");
    let h4 = section_card(cx, "h4", h4_sample);

    let p_sample = shadcn::typography::p(
        cx,
        "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.",
    );
    let p = section_card(cx, "p", p_sample);

    let blockquote_sample =
        shadcn::typography::blockquote(cx, "Never underestimate the power of a good laugh.");
    let blockquote = section_card(cx, "blockquote", blockquote_sample);

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
    let table = section_card(cx, "table", table_example);

    let list_example = shadcn::typography::list(
        cx,
        [
            Arc::<str>::from("Jokes are free speech."),
            Arc::<str>::from("Laughter improves morale."),
            Arc::<str>::from("Taxes should be fair."),
        ],
    )
    .test_id("ui-gallery-typography-list");
    let list = section_card(cx, "list", list_example);

    let inline_code_sample = shadcn::typography::inline_code(cx, "cargo run -p fret-ui-gallery");
    let inline_code = section_card(cx, "Inline Code", inline_code_sample);

    let lead_sample = shadcn::typography::lead(cx, "A larger lead paragraph introduces a section.");
    let lead = section_card(cx, "Lead", lead_sample);

    let large_sample = shadcn::typography::large(cx, "A large text block for emphasis.");
    let large = section_card(cx, "Large", large_sample);

    let small_sample = shadcn::typography::small(cx, "Use small for helper text and metadata.");
    let small = section_card(cx, "Small", small_sample);

    let muted_sample =
        shadcn::typography::muted(cx, "Muted text is suitable for non-primary explanations.");
    let muted = section_card(cx, "Muted", muted_sample);

    let rtl_story = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
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
        },
    )
    .test_id("ui-gallery-typography-rtl");
    let rtl = section_card(cx, "RTL", rtl_story);

    let preview_hint = shadcn::typography::muted(
        cx,
        "Typography page follows shadcn docs order and shows one focused sample per section.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                h1,
                h2,
                h3,
                h4,
                p,
                blockquote,
                table,
                list,
                inline_code,
                lead,
                large,
                small,
                muted,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-typography-component");

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Headings & Paragraphs",
                    "typography::h1(cx, \"Title\")\ntypography::h2(cx, \"Section\")\ntypography::p(cx, \"Body text\")",
                ),
                code_block(
                    cx,
                    "List / Table / Inline Code",
                    "typography::list(cx, [Arc::from(\"Item\")])\nTable::new([TableHeader::new(...), TableBody::new(...)])\ntypography::inline_code(cx, \"cargo run ...\")",
                ),
                code_block(
                    cx,
                    "Tone Variants & RTL",
                    "typography::lead(cx, ...); typography::large(cx, ...);\ntypography::small(cx, ...); typography::muted(cx, ...);\nwith_direction_provider(LayoutDirection::Rtl, |cx| ...)",
                ),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "Typography in shadcn is utility-driven; keep heading hierarchy semantic and consistent.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use `lead` for intros, `muted` for hints, and avoid overusing large text in dense panels.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For long-form content, combine typography helpers with table/list blocks for readability.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Validate RTL and narrow viewport wrapping before shipping document-like surfaces.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-typography",
        component_panel,
        code_panel,
        notes_panel,
    )
}
