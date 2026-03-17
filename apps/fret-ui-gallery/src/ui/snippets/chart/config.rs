pub const SOURCE: &str = include_str!("config.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let config: shadcn::ChartConfig = [
        (
            Arc::<str>::from("desktop"),
            shadcn::ChartConfigItem::new()
                .label("Desktop")
                .icon(IconId::new_static("lucide.monitor"))
                .color(ColorRef::Color(cx.theme().color_token("chart-1"))),
        ),
        (
            Arc::<str>::from("mobile"),
            shadcn::ChartConfigItem::new()
                .label("Mobile")
                .icon(IconId::new_static("lucide.smartphone"))
                .color(ColorRef::Color(cx.theme().color_token("chart-2"))),
        ),
    ]
    .into_iter()
    .collect();

    shadcn::chart_container(config, |cx| {
        ui::v_flex(|cx| {
            vec![
                shadcn::ChartLegend::new(shadcn::ChartLegendContent::new()).into_element(cx),
                shadcn::raw::typography::muted(
                    "When legend items are omitted, labels, icons, and colors can resolve from ChartConfig.",
                )
                .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .justify_center()
        .layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
    })
    .id("config")
    .test_id("ui-gallery-chart-config")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(520.0))
            .h_px(Px(148.0))
            .aspect_ratio(520.0 / 148.0),
    )
    .into_element(cx)
}
// endregion: example
