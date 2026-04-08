pub const SOURCE: &str = include_str!("compositions.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn cell<T>(test_id: &'static str, card: T) -> impl IntoUiElement<fret_app::App> + use<T>
where
    T: IntoUiElement<fret_app::App>,
{
    ui::v_flex(move |cx| vec![card.into_element(cx).test_id(test_id)])
        .w_full()
        .min_w_0()
        .items_start()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();

    let gap = MetricRef::space(Space::N4).resolve(&theme);
    let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());

    cx.grid(
        fret_ui::element::GridProps {
            layout,
            cols: 2,
            gap: gap.into(),
            align: fret_ui::element::CrossAlign::Start,
            ..Default::default()
        },
        |cx| {
            let content_only = {
                cell(
                    "ui-gallery-card-compositions-content-only",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_content(
                                |cx| ui::children![cx; ui::text("Content only.")],
                            ),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let header_only = {
                cell(
                    "ui-gallery-card-compositions-header-only",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::card_title("Header only"),
                                    shadcn::card_description(
                                        "This card has a header and a description. No content/footer.",
                                    ),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let header_and_content = {
                cell(
                    "ui-gallery-card-compositions-header-content",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(
                                |cx| ui::children![cx; shadcn::card_title("Header + Content")],
                            ),
                            shadcn::card_content(
                                |cx| ui::children![cx; ui::text("CardContent body.")],
                            ),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let header_with_action = {
                cell(
                    "ui-gallery-card-compositions-header-action",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::card_title("Header + Action")
                                        .test_id("ui-gallery-card-compositions-header-action-title"),
                                    shadcn::card_description(
                                        "Header-only composition with the optional CardAction lane.",
                                    ),
                                    shadcn::card_action(|cx| {
                                        ui::children![
                                            cx;
                                            shadcn::Button::new("Manage")
                                                .variant(shadcn::ButtonVariant::Outline)
                                                .size(shadcn::ButtonSize::Sm)
                                                .test_id("ui-gallery-card-compositions-header-action-trigger"),
                                        ]
                                    }),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let footer_only = {
                cell(
                    "ui-gallery-card-compositions-footer-only",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_footer(
                                |cx| {
                                    ui::children![
                                        cx;
                                        ui::text("Footer only.")
                                            .test_id("ui-gallery-card-compositions-footer-only-text"),
                                    ]
                                },
                            ),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let header_and_footer = {
                cell(
                    "ui-gallery-card-compositions-header-footer",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(
                                |cx| ui::children![cx; shadcn::card_title("Header + Footer")],
                            ),
                            shadcn::card_footer(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("Footer content.")
                                        .test_id("ui-gallery-card-compositions-header-footer-text"),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let content_and_footer = {
                cell(
                    "ui-gallery-card-compositions-content-footer",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_content(
                                |cx| ui::children![cx; ui::text("Body content.")],
                            ),
                            shadcn::card_footer(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("Footer content.")
                                        .test_id("ui-gallery-card-compositions-content-footer-text"),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let header_content_footer = {
                cell(
                    "ui-gallery-card-compositions-header-content-footer",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::card_title("Header + Content + Footer"),
                                    shadcn::card_description(
                                        "Full card structure without the optional action lane.",
                                    ),
                                ]
                            }),
                            shadcn::card_content(
                                |cx| ui::children![cx; ui::text("CardContent body.")],
                            ),
                            shadcn::card_footer(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("Footer content.")
                                        .test_id(
                                            "ui-gallery-card-compositions-header-content-footer-text",
                                        ),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let header_content_footer_action = {
                cell(
                    "ui-gallery-card-compositions-full-action",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::card_title("Full + Action")
                                        .test_id("ui-gallery-card-compositions-full-action-title"),
                                    shadcn::card_description(
                                        "Full card structure with the optional CardAction lane.",
                                    ),
                                    shadcn::card_action(|cx| {
                                        ui::children![
                                            cx;
                                            shadcn::Button::new("Review")
                                                .variant(shadcn::ButtonVariant::Link)
                                                .test_id("ui-gallery-card-compositions-full-action-trigger"),
                                        ]
                                    }),
                                ]
                            }),
                            shadcn::card_content(
                                |cx| ui::children![cx; ui::text("CardContent body.")],
                            ),
                            shadcn::card_footer(|cx| {
                                ui::children![
                                    cx;
                                    ui::text("Footer content.")
                                        .test_id("ui-gallery-card-compositions-full-action-footer-text"),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            let bordered_sections = {
                cell(
                    "ui-gallery-card-compositions-bordered-sections",
                    shadcn::card(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_header(
                                |cx| {
                                    ui::children![
                                        cx;
                                        shadcn::card_title("Header/Footer Borders (Fret)"),
                                        shadcn::card_description(
                                            "Fret follow-up: caller-owned section borders beyond the upstream docs path.",
                                        ),
                                    ]
                                },
                            )
                            .border_bottom(true),
                            shadcn::card_content(|cx| {
                                ui::children![
                                    cx;
                                    ui::text(
                                        "Header/footer borders can be enabled independently.",
                                    )
                                ]
                            }),
                            shadcn::card_footer(
                                |cx| {
                                    ui::children![
                                        cx;
                                        ui::text("Footer with Border")
                                            .test_id("ui-gallery-card-compositions-bordered-footer-text"),
                                    ]
                                },
                            )
                            .border_top(true),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0))),
                )
                .into_element(cx)
            };

            vec![
                content_only,
                header_only,
                header_and_content,
                header_with_action,
                footer_only,
                header_and_footer,
                content_and_footer,
                header_content_footer,
                header_content_footer_action,
                bordered_sections,
            ]
        },
    )
    .test_id("ui-gallery-card-compositions")
}
// endregion: example
