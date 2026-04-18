pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let checked = cx.local_model(|| false);
    let control_id = ControlId::from("ui-gallery-switch-invalid");

    shadcn::Field::new([
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Accept terms and conditions")
                .for_control(control_id.clone())
                .into_element(cx),
            shadcn::FieldDescription::new("You must accept the terms and conditions to continue.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::Switch::new(checked)
            .control_id(control_id)
            .aria_invalid(true)
            .a11y_label("Accept terms and conditions")
            .test_id("ui-gallery-switch-invalid-control")
            .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .invalid(true)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-switch-invalid")
}
// endregion: example
