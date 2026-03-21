pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("ui-gallery-slider-label-value", || vec![50.0]);

    let control_id = ControlId::from("ui-gallery-slider-label");
    let slider = shadcn::slider(value)
        .range(0.0, 100.0)
        .control_id(control_id.clone())
        .test_id_prefix("ui-gallery-slider-label")
        .into_element(cx);

    shadcn::field_group(|cx| {
        ui::children![
            cx;
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Volume")
                        .for_control(control_id.clone())
                        .test_id("ui-gallery-slider-label-label")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Click the label to focus the slider thumb.")
                        .for_control(control_id.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
                slider,
            ]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-slider-label-field")
}
// endregion: example
