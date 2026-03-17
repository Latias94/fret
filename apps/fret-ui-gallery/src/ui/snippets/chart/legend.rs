pub const SOURCE: &str = include_str!("legend.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let chart_1 = cx.theme().color_token("chart-1");
    let chart_2 = cx.theme().color_token("chart-2");
    let chart_3 = cx.theme().color_token("chart-3");

    let section_intro = |cx: &mut UiCx<'_>, title: &'static str, description: &'static str| {
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

    let colors_config: shadcn::ChartConfig = [
        (
            Arc::<str>::from("desktop"),
            shadcn::ChartConfigItem::new()
                .label("Desktop")
                .color(ColorRef::Color(chart_1)),
        ),
        (
            Arc::<str>::from("mobile"),
            shadcn::ChartConfigItem::new()
                .label("Mobile")
                .color(ColorRef::Color(chart_2)),
        ),
        (
            Arc::<str>::from("tablet"),
            shadcn::ChartConfigItem::new()
                .label("Tablet")
                .color(ColorRef::Color(chart_3)),
        ),
    ]
    .into_iter()
    .collect();

    let colors_legend = shadcn::chart_container(colors_config, |cx| {
        shadcn::ChartLegendContent::new()
            .into_element(cx)
            .test_id("ui-gallery-chart-legend-colors")
    })
    .id("legend-colors")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(360.0))
            .h_px(Px(88.0)),
    )
    .into_element(cx);

    let custom_keys_config: shadcn::ChartConfig = [
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
                shadcn::ChartLegendContent::new()
                    .name_key("browser")
                    .items([
                        shadcn::ChartLegendItem::new("ignored").meta("browser", "chrome"),
                        shadcn::ChartLegendItem::new("ignored").meta("browser", "safari"),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-chart-legend-custom-keys"),
                shadcn::raw::typography::muted(
                    "`name_key` remaps legend labels from item metadata into `ChartConfig` entries.",
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
    .id("legend-custom-keys")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(360.0))
            .h_px(Px(120.0)),
    )
    .into_element(cx);

    ui::v_flex(|cx| {
        vec![
            section_intro(
                cx,
                "Colors",
                "Like shadcn, legend labels, colors, and icons can be derived directly from `ChartConfig` when explicit items are omitted.",
            ),
            colors_legend,
            section_intro(
                cx,
                "Custom",
                "Use `name_key` to map legend names through item metadata into `ChartConfig` labels.",
            ),
            custom_keys,
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}
// endregion: example
