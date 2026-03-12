pub const SOURCE: &str = include_str!("legend.rs");

// region: example
use fret::UiCx;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let chart_1 = cx.theme().color_token("chart-1");
    let chart_2 = cx.theme().color_token("chart-2");
    let chart_3 = cx.theme().color_token("chart-3");

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
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
