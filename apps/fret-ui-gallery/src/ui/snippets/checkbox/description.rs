pub const SOURCE: &str = include_str!("description.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let description = cx.local_model(|| true);

    shadcn::Field::new([
        shadcn::Checkbox::new(description)
            .control_id("ui-gallery-checkbox-description")
            .a11y_label("Accept terms and conditions")
            .test_id("ui-gallery-checkbox-description")
            .into_element(cx),
        shadcn::FieldContent::new([
            shadcn::FieldLabel::new("Accept terms and conditions")
                .for_control("ui-gallery-checkbox-description")
                .into_element(cx),
            shadcn::FieldDescription::new(
                "By clicking this checkbox, you agree to the terms and conditions.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .orientation(shadcn::FieldOrientation::Horizontal)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-description-field")
}
// endregion: example
