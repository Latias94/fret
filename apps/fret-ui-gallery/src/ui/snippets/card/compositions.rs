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
                        ui::text(cx, "Content Only").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_only = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header Only").into_element(cx),
                        shadcn::CardDescription::new(
                            "This is a card with a header and a description.",
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
                        shadcn::CardTitle::new("Header and Content").into_element(cx),
                        shadcn::CardDescription::new("This is a card with a header and a content.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text(cx, "Content").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let footer_only = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardFooter::new(vec![
                        ui::text(cx, "Footer Only").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_and_footer = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header + Footer").into_element(cx),
                        shadcn::CardDescription::new("This is a card with a header and a footer.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text(cx, "Footer").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let content_and_footer = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardContent::new(vec![
                        ui::text(cx, "Content").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text(cx, "Footer").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_content_footer = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header + Footer").into_element(cx),
                        shadcn::CardDescription::new("This is a card with a header and a footer.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text(cx, "Content").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text(cx, "Footer").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let header_with_border = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Header with Border").into_element(cx),
                        shadcn::CardDescription::new(
                            "This is a card with a header that has a bottom border.",
                        )
                        .into_element(cx),
                    ])
                    .border_bottom(true)
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text(cx, "Content").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, card)
            };

            let footer_with_border = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardContent::new(vec![
                        ui::text(cx, "Content").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardFooter::new(vec![
                        ui::text(cx, "Footer with Border")
                            .text_sm()
                            .into_element(cx),
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
                header_with_border,
                footer_with_border,
            ]
        },
    )
    .test_id("ui-gallery-card-compositions")
}
// endregion: example
