// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    max_w_md: LayoutRefinement,
) -> AnyElement {
    shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Contact").into_element(cx),
        shadcn::FieldDescription::new(
            "Model-bound controls keep values while you stay in the window.",
        )
        .into_element(cx),
        shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Email").into_element(cx),
                shadcn::Input::new(text_input.clone())
                    .a11y_label("Email")
                    .placeholder("name@example.com")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Message").into_element(cx),
                shadcn::Textarea::new(text_area.clone())
                    .a11y_label("Message")
                    .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Checkbox::new(checkbox.clone())
                    .control_id("ui-gallery-form-checkbox-terms")
                    .a11y_label("Accept terms")
                    .into_element(cx),
                shadcn::FieldLabel::new("Accept terms")
                    .for_control("ui-gallery-form-checkbox-terms")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldContent::new([
                    shadcn::FieldLabel::new("Enable feature")
                        .for_control("ui-gallery-form-switch-feature")
                        .into_element(cx),
                    shadcn::FieldDescription::new(
                        "This toggles an optional feature for the current session.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Switch::new(switch.clone())
                    .control_id("ui-gallery-form-switch-feature")
                    .a11y_label("Enable feature")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-form-demo")
}
// endregion: example
