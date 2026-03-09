pub const SOURCE: &str = include_str!("compositions.rs");

// region: example
use fret_app::App;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let cell = |cx: &mut ElementContext<'_, App>, card: shadcn::Card| {
        card.refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0)))
            .into_element(cx)
    };

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
                let card = shadcn::Card::new(vec![
                    shadcn::CardContent::new(vec![
                        ui::text("Content only.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_only = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header only").into_element(cx),
                        shadcn::CardDescription::new(
                            "This card has a header and a description. No content/footer.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_and_content = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header + Content").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text("CardContent body.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let footer_only = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardFooter::new(vec![
                        ui::text("Footer only.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_and_footer = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header + Footer").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text("Footer content.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let content_and_footer = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardContent::new(vec![
                        ui::text("Body content.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text("Footer content.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_content_footer = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header + Content + Footer").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text("CardContent body.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text("Footer content.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let bordered_sections = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Bordered Sections").into_element(cx),
                    ])
                    .border_bottom(true)
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text("Header/footer borders can be enabled independently.")
                            .text_sm()
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text("Footer with Border").text_sm().into_element(cx),
                    ])
                    .border_top(true)
                    .into_element(cx),
                ]);
                cell(cx, card)
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
