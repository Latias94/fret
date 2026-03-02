pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use crate::ui::doc_layout;
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let chart_1 = cx.theme().color_token("chart-1");
    let chart_2 = cx.theme().color_token("chart-2");
    let chart_3 = cx.theme().color_token("chart-3");

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

    doc_layout::rtl(cx, |cx| {
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
                        "يناير",
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
    })
}
// endregion: example
