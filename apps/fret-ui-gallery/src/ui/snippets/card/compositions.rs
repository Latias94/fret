pub const SOURCE: &str = include_str!("compositions.rs");

// region: example
use fret::UiCx;
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

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
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
                                |cx| ui::children![cx; ui::text("Content only.").text_sm()],
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
                                |cx| ui::children![cx; ui::text("CardContent body.").text_sm()],
                            ),
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
                                |cx| ui::children![cx; ui::text("Footer only.").text_sm()],
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
                            shadcn::card_footer(
                                |cx| ui::children![cx; ui::text("Footer content.").text_sm()],
                            ),
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
                                |cx| ui::children![cx; ui::text("Body content.").text_sm()],
                            ),
                            shadcn::card_footer(
                                |cx| ui::children![cx; ui::text("Footer content.").text_sm()],
                            ),
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
                                ]
                            }),
                            shadcn::card_content(
                                |cx| ui::children![cx; ui::text("CardContent body.").text_sm()],
                            ),
                            shadcn::card_footer(
                                |cx| ui::children![cx; ui::text("Footer content.").text_sm()],
                            ),
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
                                |cx| ui::children![cx; shadcn::card_title("Bordered Sections")],
                            )
                            .border_bottom(true),
                            shadcn::card_content(|cx| {
                                ui::children![
                                    cx;
                                    ui::text(
                                        "Header/footer borders can be enabled independently.",
                                    )
                                    .text_sm(),
                                ]
                            }),
                            shadcn::card_footer(
                                |cx| ui::children![cx; ui::text("Footer with Border").text_sm()],
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
                footer_only,
                header_and_footer,
                content_and_footer,
                header_content_footer,
                bordered_sections,
            ]
        },
    )
    .test_id("ui-gallery-card-compositions")
}
// endregion: example
