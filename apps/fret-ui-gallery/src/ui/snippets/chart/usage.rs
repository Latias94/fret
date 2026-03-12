pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::UiCx;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let config: shadcn::ChartConfig = [
        (
            Arc::<str>::from("desktop"),
            shadcn::ChartConfigItem::new()
                .label("Desktop")
                .color(ColorRef::Color(cx.theme().color_token("chart-1"))),
        ),
        (
            Arc::<str>::from("mobile"),
            shadcn::ChartConfigItem::new()
                .label("Mobile")
                .color(ColorRef::Color(cx.theme().color_token("chart-2"))),
        ),
    ]
    .into_iter()
    .collect();

    shadcn::ChartContainer::new(config)
        .id("traffic")
        .test_id("ui-gallery-chart-usage")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(480.0))
                .aspect_ratio(16.0 / 9.0),
        )
        .into_element(cx, |cx| {
            let body = ui::v_flex(|cx| {
                vec![
                    shadcn::ChartTooltip::new(
                        shadcn::ChartTooltipContent::new().label("January").items([
                            shadcn::ChartTooltipItem::new("Desktop", "186")
                                .color(ColorRef::Color(cx.theme().color_token("chart-1"))),
                            shadcn::ChartTooltipItem::new("Mobile", "80")
                                .color(ColorRef::Color(cx.theme().color_token("chart-2"))),
                        ]),
                    )
                    .into_element(cx),
                    shadcn::ChartLegend::new(
                        shadcn::ChartLegendContent::new().items([
                            shadcn::ChartLegendItem::new("Desktop")
                                .color(ColorRef::Color(cx.theme().color_token("chart-1"))),
                            shadcn::ChartLegendItem::new("Mobile")
                                .color(ColorRef::Color(cx.theme().color_token("chart-2"))),
                        ]),
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N4)
            .items_start()
            .justify_center()
            .layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx);

            let theme = cx.theme().snapshot();
            let props = fret_ui_shadcn::decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N4),
                LayoutRefinement::default().w_full().h_full(),
            );
            cx.container(props, move |_cx| vec![body])
        })
}
// endregion: example
