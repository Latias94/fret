use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let round = |cx: &mut ElementContext<'_, App>, size: f32| {
        shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(size))
                    .h_px(Px(size))
                    .flex_shrink_0(),
            )
            .into_element(cx)
    };

    let demo = {
        let avatar_row = {
            let text_lines = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_px(Px(150.0))),
                |cx| {
                    vec![
                        shadcn::Skeleton::new()
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_px(Px(16.0))
                                    .min_w_0(),
                            )
                            .into_element(cx),
                        shadcn::Skeleton::new()
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(100.0)).h_px(Px(16.0)),
                            )
                            .into_element(cx),
                    ]
                },
            );

            stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N4).items_center(),
                |cx| vec![round(cx, 40.0), text_lines],
            )
        };

        let cards = {
            let card = |cx: &mut ElementContext<'_, App>, idx: usize| {
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::Skeleton::new()
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(180.0)).h_px(Px(16.0)),
                            )
                            .into_element(cx),
                        shadcn::Skeleton::new()
                            .refine_layout(
                                LayoutRefinement::default().w_px(Px(136.0)).h_px(Px(16.0)),
                            )
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_full().aspect_ratio(1.0))
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .max_w(Px(320.0))
                        .min_w_0(),
                )
                .into_element(cx)
                .test_id(format!("ui-gallery-skeleton-demo-card-{idx}"))
            };

            doc_layout::wrap_row_snapshot(
                cx,
                &theme,
                Space::N4,
                fret_ui::element::CrossAlign::Start,
                |cx| (1..=3).map(|idx| card(cx, idx)).collect::<Vec<_>>(),
            )
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |_cx| vec![avatar_row, cards],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-demo"),
        )
    };

    let avatar = {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(150.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(100.0)))
                        .into_element(cx),
                ]
            },
        );

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 40.0), text_lines],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-avatar"),
        )
    };

    let card = {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(170.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(144.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-card"),
        )
    };

    let text_section = {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-text"),
        )
    };

    let form = {
        let row = |cx: &mut ElementContext<'_, App>, label_w: Px| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| {
                    vec![
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_px(label_w))
                            .into_element(cx),
                        shadcn::Skeleton::new()
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(32.0)))
                            .into_element(cx),
                    ]
                },
            )
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    row(cx, Px(80.0)),
                    row(cx, Px(96.0)),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-form"),
        )
    };

    let table = {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))),
            |cx| {
                (0..5)
                    .map(|_| {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .gap(Space::N4)
                                .items_center()
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    shadcn::Skeleton::new()
                                        .refine_layout(
                                            LayoutRefinement::default().flex_1().min_w_0(),
                                        )
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(96.0)))
                                        .into_element(cx),
                                    shadcn::Skeleton::new()
                                        .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
                                        .into_element(cx),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-skeleton-table"),
        )
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        let text_lines = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_px(Px(250.0))),
            |cx| {
                vec![
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Skeleton::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(200.0)))
                        .into_element(cx),
                ]
            },
        );

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| vec![round(cx, 48.0), text_lines],
        )
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-skeleton-rtl"),
    );

    let notes = doc_layout::notes(
        cx,
        [
            "Use Skeleton for loading placeholders, not empty states.",
            "Prefer consistent sizes and spacing so content doesn't jump when loaded.",
            "Keep semantics grouped so screen readers can skip placeholder-only regions.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Skeleton demo: avatar row + cards."),
        vec![
            DocSection::new("Demo", demo)
                .description("Avatar row + card list.")
                .code(
                    "rust",
                    r#"let avatar_row = stack::hstack(cx, props, |cx| {
    vec![
        shadcn::Skeleton::new()
            .refine_style(ChromeRefinement::default().rounded(Radius::Full))
            .refine_layout(LayoutRefinement::default().w_px(Px(40.0)).h_px(Px(40.0)))
            .into_element(cx),
        /* text lines */
    ]
});

let cards = (1..=3).map(|_| {
    shadcn::Card::new([
        shadcn::CardHeader::new([shadcn::Skeleton::new().into_element(cx)]).into_element(cx),
        shadcn::CardContent::new([shadcn::Skeleton::new().into_element(cx)]).into_element(cx),
    ])
    .into_element(cx)
});

stack::vstack(cx, props, |_cx| vec![avatar_row, /* cards */]);"#,
                ),
            DocSection::new("Avatar", avatar)
                .description("Smaller avatar placeholder.")
                .code(
                    "rust",
                    r#"shadcn::Skeleton::new()
    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
    .refine_layout(LayoutRefinement::default().w_px(Px(40.0)).h_px(Px(40.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Card", card)
                .description("Skeletons inside a card layout.")
                .code(
                    "rust",
                    r#"shadcn::Card::new(vec![
    shadcn::CardHeader::new(vec![shadcn::Skeleton::new().into_element(cx)]).into_element(cx),
    shadcn::CardContent::new(vec![shadcn::Skeleton::new().into_element(cx)]).into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Text", text_section)
                .description("Multiple lines with varying widths.")
                .code(
                    "rust",
                    r#"stack::vstack(cx, props, |cx| {
    vec![
        shadcn::Skeleton::new().refine_layout(LayoutRefinement::default().w_full()).into_element(cx),
        shadcn::Skeleton::new().refine_layout(LayoutRefinement::default().w_px(Px(240.0))).into_element(cx),
    ]
});"#,
                ),
            DocSection::new("Form", form)
                .description("Form-like blocks.")
                .code(
                    "rust",
                    r#"shadcn::Skeleton::new()
    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(32.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Table", table)
                .description("Row skeletons.")
                .code(
                    "rust",
                    r#"stack::hstack(cx, props, |cx| {
    vec![
        shadcn::Skeleton::new().refine_layout(LayoutRefinement::default().flex_1()).into_element(cx),
        shadcn::Skeleton::new().refine_layout(LayoutRefinement::default().w_px(Px(96.0))).into_element(cx),
    ]
});"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Direction provider should not break layout.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::Skeleton::new().into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-skeleton")]
}
