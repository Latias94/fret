pub const SOURCE: &str = include_str!("tooltip.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let chart_1 = cx.theme().color_token("chart-1");
    let chart_2 = cx.theme().color_token("chart-2");
    let chart_3 = cx.theme().color_token("chart-3");

    let section_intro =
        |cx: &mut AppComponentCx<'_>, title: &'static str, description: &'static str| {
            ui::v_flex(|cx| {
                vec![
                    shadcn::raw::typography::small(title).into_element(cx),
                    shadcn::raw::typography::muted(description).into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx)
        };

    let tooltip = |cx: &mut AppComponentCx<'_>,
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

    let custom_label_children = shadcn::ChartTooltipContent::new()
        .label("2024-07-16")
        .items([
            shadcn::ChartTooltipItem::new("Desktop", "186").color(ColorRef::Color(chart_1)),
            shadcn::ChartTooltipItem::new("Mobile", "80").color(ColorRef::Color(chart_2)),
        ])
        .test_id_prefix("ui-gallery-chart-tooltip-custom-label-children")
        .into_element_label_parts(cx, |cx, context| {
            ui::h_row(|cx| {
                vec![
                    ui::text("Date:").text_xs().font_medium().into_element(cx),
                    ui::text(context.label.clone().unwrap_or_default())
                        .text_xs()
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx)
        })
        .test_id("ui-gallery-chart-tooltip-custom-label-children");

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

    let custom_children = shadcn::ChartTooltipContent::new()
        .hide_label(true)
        .items([
            shadcn::ChartTooltipItem::new("Running", "380").meta("unit", "kcal"),
            shadcn::ChartTooltipItem::new("Swimming", "420").meta("unit", "kcal"),
        ])
        .test_id_prefix("ui-gallery-chart-tooltip-custom-children")
        .into_element_parts(cx, |cx, context| {
            let unit = context
                .item
                .metadata
                .get("unit")
                .cloned()
                .unwrap_or_default();

            ui::h_row(|cx| {
                vec![
                    ui::text(format!("{}.", context.index + 1))
                        .text_xs()
                        .font_medium()
                        .into_element(cx),
                    ui::text(context.item.label.clone())
                        .text_xs()
                        .into_element(cx),
                    ui::text(format!("{} {unit}", context.item.value))
                        .text_xs()
                        .font_medium()
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .justify_between()
            .items_center()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx)
        })
        .test_id("ui-gallery-chart-tooltip-custom-children");

    let custom_parts_with_label = shadcn::ChartTooltipContent::new()
        .label("2024-07-16")
        .items([
            shadcn::ChartTooltipItem::new("Running", "380").meta("unit", "kcal"),
            shadcn::ChartTooltipItem::new("Swimming", "420").meta("unit", "kcal"),
        ])
        .test_id_prefix("ui-gallery-chart-tooltip-custom-parts-with-label")
        .into_element_parts_with_label(
            cx,
            |cx, context| {
                ui::v_flex(|cx| {
                    vec![
                        ui::text("Workout Summary")
                            .text_xs()
                            .font_medium()
                            .into_element(cx),
                        ui::text(context.label.clone().unwrap_or_default())
                            .text_xs()
                            .text_color(ColorRef::Color(cx.theme().color_token("muted-foreground")))
                            .into_element(cx),
                    ]
                })
                .gap(Space::N1)
                .items_start()
                .into_element(cx)
            },
            |cx, context| {
                let unit = context
                    .item
                    .metadata
                    .get("unit")
                    .cloned()
                    .unwrap_or_default();

                ui::h_row(|cx| {
                    vec![
                        ui::text(context.item.label.clone())
                            .text_xs()
                            .into_element(cx),
                        ui::text(format!("{} {unit}", context.item.value))
                            .text_xs()
                            .font_medium()
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .justify_between()
                .items_center()
                .layout(LayoutRefinement::default().w_full())
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-chart-tooltip-custom-parts-with-label");

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
                    "Colors and labels are resolved from `ChartConfig`; `label_key` and `name_key` map the engine output into those config entries.",
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
            section_intro(
                cx,
                "Props",
                "Indicator style plus `hide_label` and `hide_indicator` mirror the first shadcn tooltip teaching surface.",
            ),
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
            section_intro(
                cx,
                "Colors",
                "Like shadcn, tooltip chrome should derive color and display labels from `ChartConfig` whenever possible.",
            ),
            custom_keys,
            section_intro(
                cx,
                "Custom",
                "Formatter hooks stay on the default recipe lane; custom header-only, row-only, and combined label-plus-row children stay on the explicit adapter seams.",
            ),
            label_formatter,
            formatter,
            custom_label_children,
            custom_children,
            custom_parts_with_label,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
