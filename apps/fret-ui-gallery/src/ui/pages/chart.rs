use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let (chart_1, chart_2, chart_3) = cx.with_theme(|theme| {
        (
            theme.color_token("chart-1"),
            theme.color_token("chart-2"),
            theme.color_token("chart-3"),
        )
    });

    let tooltip = |cx: &mut ElementContext<'_, App>,
                   label: &'static str,
                   indicator: shadcn::ChartTooltipIndicator,
                   hide_label: bool,
                   hide_indicator: bool,
                   test_id: &'static str| {
        shadcn::ChartTooltipContent::new()
            .label(label)
            .items([
                shadcn::ChartTooltipItem::new("Desktop", "186").color(ColorRef::Color(chart_1)),
                shadcn::ChartTooltipItem::new("Mobile", "80").color(ColorRef::Color(chart_2)),
                shadcn::ChartTooltipItem::new("Tablet", "42").color(ColorRef::Color(chart_3)),
            ])
            .indicator(indicator)
            .hide_label(hide_label)
            .hide_indicator(hide_indicator)
            .test_id_prefix(test_id)
            .into_element(cx)
            .test_id(test_id)
    };

    let legend = |cx: &mut ElementContext<'_, App>,
                  align: shadcn::ChartLegendVerticalAlign,
                  wrap: bool,
                  hide_icon: bool,
                  test_id: &'static str| {
        shadcn::ChartLegendContent::new()
            .items([
                shadcn::ChartLegendItem::new("Desktop").color(ColorRef::Color(chart_1)),
                shadcn::ChartLegendItem::new("Mobile").color(ColorRef::Color(chart_2)),
                shadcn::ChartLegendItem::new("Tablet").color(ColorRef::Color(chart_3)),
            ])
            .vertical_align(align)
            .wrap(wrap)
            .hide_icon(hide_icon)
            .into_element(cx)
            .test_id(test_id)
    };

    let demo = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Dot,
                    false,
                    false,
                    "ui-gallery-chart-demo-tooltip",
                ),
                legend(
                    cx,
                    shadcn::ChartLegendVerticalAlign::Bottom,
                    true,
                    false,
                    "ui-gallery-chart-demo-legend",
                ),
            ]
        },
    );

    let component = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Map series to `chart-*` tokens so tooltip and legend stay color-consistent across light/dark themes.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Current Fret abstraction intentionally stays lightweight and does not wrap Recharts runtime APIs.",
                ),
            ]
        },
    );

    let tooltip_section = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Dot,
                    false,
                    false,
                    "ui-gallery-chart-tooltip-dot",
                ),
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Line,
                    false,
                    false,
                    "ui-gallery-chart-tooltip-line",
                ),
                tooltip(
                    cx,
                    "January",
                    shadcn::ChartTooltipIndicator::Dashed,
                    true,
                    false,
                    "ui-gallery-chart-tooltip-dashed",
                ),
            ]
        },
    );

    let legend_section = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                legend(
                    cx,
                    shadcn::ChartLegendVerticalAlign::Top,
                    false,
                    false,
                    "ui-gallery-chart-legend-top",
                ),
                legend(
                    cx,
                    shadcn::ChartLegendVerticalAlign::Bottom,
                    true,
                    true,
                    "ui-gallery-chart-legend-wrap-no-icon",
                ),
            ]
        },
    );

    let accessibility = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Upstream Recharts supports `accessibilityLayer` for keyboard and screen-reader navigation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Fret gallery keeps this as a documented parity gap until chart runtime integration is introduced.",
                ),
            ]
        },
    );

    let rtl = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                |cx| {
                    vec![
                        tooltip(
                            cx,
                            "?????",
                            shadcn::ChartTooltipIndicator::Dot,
                            false,
                            false,
                            "ui-gallery-chart-rtl-tooltip",
                        ),
                        legend(
                            cx,
                            shadcn::ChartLegendVerticalAlign::Bottom,
                            true,
                            false,
                            "ui-gallery-chart-rtl-legend",
                        ),
                    ]
                },
            )
        },
    );

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/chart.rs`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "This page validates tooltip/legend composition parity, not full chart drawing parity.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Accessibility and full Recharts API integration are intentionally tracked as follow-up work.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Chart docs flow: Demo -> Component -> Tooltip -> Legend -> Accessibility -> RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("shadcn Chart is Recharts composition. Fret page focuses on tooltip/legend contracts.")
                .max_w(Px(760.0)),
            DocSection::new("Component", component)
                .description("Color mapping + composition notes for chart surfaces.")
                .max_w(Px(760.0)),
            DocSection::new("Tooltip", tooltip_section)
                .description("Tooltip content variants (dot/line/dashed indicators).")
                .code(
                    "rust",
                    r#"let tooltip = shadcn::ChartTooltipContent::new()
    .label("January")
    .items([
        shadcn::ChartTooltipItem::new("Desktop", "186"),
        shadcn::ChartTooltipItem::new("Mobile", "80"),
    ])
    .indicator(shadcn::ChartTooltipIndicator::Line)
    .into_element(cx);"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("Legend", legend_section)
                .description("Legend vertical align + wrapping behavior.")
                .max_w(Px(760.0)),
            DocSection::new("Accessibility", accessibility)
                .description("Documented parity gap until chart runtime integration lands.")
                .max_w(Px(760.0)),
            DocSection::new("RTL", rtl)
                .description("Direction provider sample to validate RTL tooltip/legend layouts.")
                .max_w(Px(760.0)),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-chart-component")]
}
