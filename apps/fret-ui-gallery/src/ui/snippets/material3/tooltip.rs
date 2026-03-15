pub const SOURCE: &str = include_str!("tooltip.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content = material3::TooltipProvider::new().with_elements(cx, |cx| {
        let outlined = material3::ButtonVariant::Outlined;

        let top = material3::PlainTooltip::new(
            material3::Button::new("Hover (Top)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-top-trigger")
                .into_element(cx),
            "Plain tooltip (top)",
        )
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let right = material3::PlainTooltip::new(
            material3::Button::new("Hover (Right)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-right-trigger")
                .into_element(cx),
            "Plain tooltip (right)",
        )
        .side(material3::TooltipSide::Right)
        .into_element(cx);

        let bottom = material3::PlainTooltip::new(
            material3::Button::new("Hover (Bottom)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-bottom-trigger")
                .into_element(cx),
            "Plain tooltip (bottom)",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        let left = material3::PlainTooltip::new(
            material3::Button::new("Hover (Left)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-left-trigger")
                .into_element(cx),
            "Plain tooltip (left)",
        )
        .side(material3::TooltipSide::Left)
        .into_element(cx);

        let rich = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-trigger")
                .into_element(cx),
            "Rich tooltip supporting text (body medium).",
        )
        .title("Rich tooltip title")
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let rich_no_title = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich / no title)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-no-title-trigger")
                .into_element(cx),
            "Rich tooltip supporting text only.",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        vec![
            ui::h_flex(|_cx| [top, right, bottom, left])
                    .gap(Space::N4)
                    .layout(LayoutRefinement::default().w_full()).into_element(cx),
            ui::h_flex(|_cx| [rich, rich_no_title])
                    .gap(Space::N4)
                    .layout(LayoutRefinement::default().w_full()).into_element(cx),
            cx.text("Note: Tooltip open delay is controlled via Material3 TooltipProvider (delay-group)."),
        ]
    });

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Tooltip"),
                    shadcn::card_description(
                        "Tooltip MVP: delay group + hover intent + safe-hover corridor + token-driven styling (plain + rich).",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| content),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

// endregion: example
