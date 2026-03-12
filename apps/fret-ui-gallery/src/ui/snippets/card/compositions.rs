pub const SOURCE: &str = include_str!("compositions.rs");

// region: example
use fret::UiCx;
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn cell(
    cx: &mut UiCx<'_>,
    test_id: &'static str,
    card: shadcn::Card,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let card = card
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(260.0)))
        .into_element(cx)
        .test_id(test_id);

    ui::v_flex(move |_cx| vec![card])
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
                let card = shadcn::Card::new(vec![
                    shadcn::CardContent::new(vec![
                        ui::text("Content only.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, "ui-gallery-card-compositions-content-only", card).into_element(cx)
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
                cell(cx, "ui-gallery-card-compositions-header-only", card).into_element(cx)
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
                cell(cx, "ui-gallery-card-compositions-header-content", card).into_element(cx)
            };

            let footer_only = {
                let card = shadcn::Card::new(vec![
                    shadcn::CardFooter::new(vec![
                        ui::text("Footer only.").text_sm().into_element(cx),
                    ])
                    .into_element(cx),
                ]);
                cell(cx, "ui-gallery-card-compositions-footer-only", card).into_element(cx)
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
                cell(cx, "ui-gallery-card-compositions-header-footer", card).into_element(cx)
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
                cell(cx, "ui-gallery-card-compositions-content-footer", card).into_element(cx)
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
                cell(
                    cx,
                    "ui-gallery-card-compositions-header-content-footer",
                    card,
                )
                .into_element(cx)
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
                cell(cx, "ui-gallery-card-compositions-bordered-sections", card).into_element(cx)
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
