pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    text_input: Option<Model<String>>,
    text_area: Option<Model<String>>,
    checkbox: Option<Model<bool>>,
    switch: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (text_input, text_area, checkbox, switch) = cx.with_state(Models::default, |st| {
        (
            st.text_input.clone(),
            st.text_area.clone(),
            st.checkbox.clone(),
            st.switch.clone(),
        )
    });

    let (text_input, text_area, checkbox, switch) = match (text_input, text_area, checkbox, switch)
    {
        (Some(text_input), Some(text_area), Some(checkbox), Some(switch)) => {
            (text_input, text_area, checkbox, switch)
        }
        _ => {
            let text_input = cx.app.models_mut().insert(String::new());
            let text_area = cx.app.models_mut().insert(String::new());
            let checkbox = cx.app.models_mut().insert(false);
            let switch = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.text_input = Some(text_input.clone());
                st.text_area = Some(text_area.clone());
                st.checkbox = Some(checkbox.clone());
                st.switch = Some(switch.clone());
            });
            (text_input, text_area, checkbox, switch)
        }
    };

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

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
