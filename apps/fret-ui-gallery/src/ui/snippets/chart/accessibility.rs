pub const SOURCE: &str = include_str!("accessibility.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[allow(dead_code)]
pub fn apply_chart_accessibility_defaults(
    props: fret_chart::ChartCanvasPanelProps,
) -> fret_chart::ChartCanvasPanelProps {
    props
        .accessibility_layer(true)
        .input_map(fret_chart::input_map::ChartInputMap::default())
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::typography::muted(
                "Focus the chart canvas, then use arrow keys to move the active point.",
            )
            .into_element(cx),
            shadcn::typography::muted(
                "Fret mirrors the high-level `accessibilityLayer` outcome through the declarative chart panel rather than DOM nodes.",
            )
            .into_element(cx),
            shadcn::typography::muted(
                "A reusable helper keeps the accessibility layer and the default chart input map together.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
    .into_element(cx)
}
// endregion: example
