pub const SOURCE: &str = include_str!("contracts.rs");

// region: example
use fret::UiCx;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let chart_1 = cx.theme().color_token("chart-1");
    let chart_2 = cx.theme().color_token("chart-2");
    let chart_3 = cx.theme().color_token("chart-3");

    let tooltip = |cx: &mut UiCx<'_>,
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

    let legend = |cx: &mut UiCx<'_>,
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

    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(
                cx,
                "Chart UI contracts: Tooltip + Legend content recipes.",
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
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
