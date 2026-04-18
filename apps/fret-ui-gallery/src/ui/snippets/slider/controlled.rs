pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let controlled_values =
        cx.local_model_keyed("ui-gallery-slider-controlled-values", || vec![0.3, 0.7]);
    let control_id = ControlId::from("ui-gallery-slider-controlled-temperature");
    let values_snapshot: Vec<f32> = cx
        .get_model_cloned(&controlled_values, Invalidation::Paint)
        .unwrap_or_default();
    let values_text = values_snapshot
        .iter()
        .map(|value| format!("{value:.1}"))
        .collect::<Vec<_>>()
        .join(", ");

    let header = ui::h_flex(|cx| {
        vec![
            shadcn::Label::new("Temperature")
                .for_control(control_id.clone())
                .into_element(cx),
            shadcn::raw::typography::muted(values_text).into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .items_center()
    .justify_between()
    .into_element(cx);

    let slider = shadcn::slider(controlled_values)
        .range(0.0, 1.0)
        .step(0.1)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-slider-controlled")
        .into_element(cx);

    ui::v_flex(|_cx| vec![header, slider])
        .gap(Space::N3)
        .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
}
// endregion: example
