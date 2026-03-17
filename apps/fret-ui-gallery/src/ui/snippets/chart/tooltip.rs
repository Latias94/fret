pub const SOURCE: &str = include_str!("tooltip.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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

    let label_formatter = shadcn::ChartTooltipContent::new()
        .label("2024-07-16")
        .label_formatter(|context| {
            let label = context.label.as_deref().unwrap_or_default();
            format!("Date: {label}")
        })
        .items([
            shadcn::ChartTooltipItem::new("Desktop", "186").color(ColorRef::Color(chart_1)),
            shadcn::ChartTooltipItem::new("Mobile", "80").color(ColorRef::Color(chart_2)),
        ])
        .test_id_prefix("ui-gallery-chart-tooltip-label-formatter")
        .into_element(cx)
        .test_id("ui-gallery-chart-tooltip-label-formatter");

    let formatter = shadcn::ChartTooltipContent::new()
        .hide_label(true)
        .items([
            shadcn::ChartTooltipItem::new("Running", "380")
                .key("running")
                .meta("unit", "kcal"),
            shadcn::ChartTooltipItem::new("Swimming", "420")
                .key("swimming")
                .meta("unit", "kcal"),
        ])
        .formatter(|context| {
            let series = context
                .item
                .key
                .as_deref()
                .unwrap_or(context.item.label.as_ref());
            let unit = context
                .item
                .metadata
                .get("unit")
                .cloned()
                .unwrap_or_default();
            shadcn::ChartTooltipFormattedItem::from_item(&context.item)
                .label(format!("Series {series}"))
                .value_suffix(unit)
                .row_min_width(Px(130.0))
        })
        .test_id_prefix("ui-gallery-chart-tooltip-formatter")
        .into_element(cx)
        .test_id("ui-gallery-chart-tooltip-formatter");

    let custom_keys_config: shadcn::ChartConfig = [
        (
            Arc::<str>::from("visitors"),
            shadcn::ChartConfigItem::new().label("Total Visitors"),
        ),
        (
            Arc::<str>::from("chrome"),
            shadcn::ChartConfigItem::new()
                .label("Chrome")
                .color(ColorRef::Color(chart_1)),
        ),
        (
            Arc::<str>::from("safari"),
            shadcn::ChartConfigItem::new()
                .label("Safari")
                .color(ColorRef::Color(chart_2)),
        ),
    ]
    .into_iter()
    .collect();

    let custom_keys = shadcn::chart_container(custom_keys_config, |cx| {
        ui::v_flex(|cx| {
            vec![
                shadcn::ChartTooltipContent::new()
                    .label("187")
                    .label_key("visitors")
                    .name_key("browser")
                    .items([
                        shadcn::ChartTooltipItem::new("ignored", "187").meta("browser", "chrome"),
                        shadcn::ChartTooltipItem::new("ignored", "200").meta("browser", "safari"),
                    ])
                    .test_id_prefix("ui-gallery-chart-tooltip-custom-keys")
                    .into_element(cx),
                shadcn::raw::typography::muted(
                    "`label_key` resolves the header through `ChartConfig`; `name_key` remaps row labels from item metadata into config entries.",
                )
                .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_start()
        .justify_center()
        .layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
    })
    .id("tooltip-custom-keys")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(360.0))
            .h_px(Px(144.0))
            .aspect_ratio(360.0 / 144.0),
    )
    .into_element(cx);

    ui::v_flex(|cx| {
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
            label_formatter,
            formatter,
            custom_keys,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
