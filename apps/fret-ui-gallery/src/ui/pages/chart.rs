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

    let demo_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "shadcn Chart is Recharts composition. Fret page focuses on chart tooltip/legend contracts.",
                ),
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

    let component_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
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

    let tooltip_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
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

    let legend_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
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

    let accessibility_content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
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

    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full()),
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

    let notes_stack = doc_layout::notes(
        cx,
        [
            "This page validates tooltip/legend composition parity, not full chart drawing parity.",
            "Keep color mapping stable through `chart-*` tokens to avoid dark-theme drift.",
            "Accessibility and full Recharts API integration are intentionally tracked as follow-up work.",
            "When adding chart runtime later, keep this page order unchanged for quick docs side-by-side checks.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Chart docs flow: Demo -> Component -> Tooltip -> Legend -> Accessibility -> RTL.",
        ),
        vec![
            DocSection::new("Demo", demo_content).max_w(Px(760.0)).code(
                "rust",
                r#"let (chart_1, chart_2) =
    cx.with_theme(|theme| (theme.color_token("chart-1"), theme.color_token("chart-2")));

shadcn::ChartTooltipContent::new()
    .label("January")
    .items([
        shadcn::ChartTooltipItem::new("Desktop", "186").color(ColorRef::Color(chart_1)),
        shadcn::ChartTooltipItem::new("Mobile", "80").color(ColorRef::Color(chart_2)),
    ])
    .indicator(shadcn::ChartTooltipIndicator::Dot)
    .into_element(cx);"#,
            ),
            DocSection::new("Component", component_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"// Validate tooltip/legend contracts first, then add renderer bindings later.
shadcn::typography::muted(
    cx,
    "This page focuses on tooltip/legend composition, not full chart rendering.",
);"#,
                ),
            DocSection::new("Tooltip", tooltip_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-tooltip")
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
                ),
            DocSection::new("Legend", legend_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-legend")
                .code(
                    "rust",
                    r#"let legend = shadcn::ChartLegendContent::new()
    .items([
        shadcn::ChartLegendItem::new("Desktop"),
        shadcn::ChartLegendItem::new("Mobile"),
    ])
    .vertical_align(shadcn::ChartLegendVerticalAlign::Bottom)
    .wrap(true)
    .into_element(cx);"#,
                ),
            DocSection::new("Accessibility", accessibility_content)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"// Parity gap marker: upstream Recharts supports `accessibilityLayer`.
shadcn::Alert::new([
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.info")),
    shadcn::AlertTitle::new("Not yet implemented").into_element(cx),
    shadcn::AlertDescription::new("Chart accessibility layer is tracked as follow-up work.")
        .into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl_content)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-chart-rtl")
                .code(
                    "rust",
                    r#"with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::ChartTooltipContent::new().label("?????").into_element(cx)
})"#,
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-chart-component")]
}
