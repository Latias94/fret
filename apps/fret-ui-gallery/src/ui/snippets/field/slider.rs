pub const SOURCE: &str = include_str!("slider.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let slider_values = cx.local_model(|| vec![200.0_f32, 800.0_f32]);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let current_values = cx
        .watch_model(&slider_values)
        .layout()
        .cloned()
        .unwrap_or_else(|| vec![200.0, 800.0]);
    let start = current_values.first().copied().unwrap_or(200.0_f32).round() as i32;
    let end = current_values.get(1).copied().unwrap_or(800.0_f32).round() as i32;
    let description = format!("Set your budget range (${start}-${end}).");

    shadcn::Field::new([
        shadcn::FieldTitle::new("Price Range").into_element(cx),
        shadcn::FieldDescription::new(description).into_element(cx),
        shadcn::slider(slider_values)
            .range(0.0, 1000.0)
            .step(10.0)
            .a11y_label("Price Range")
            .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-slider")
}
// endregion: example
