pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    text_input: Option<Model<String>>,
    text_area: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (text_input, text_area) = cx.with_state(Models::default, |st| {
        (st.text_input.clone(), st.text_area.clone())
    });
    let (text_input, text_area) = match (text_input, text_area) {
        (Some(text_input), Some(text_area)) => (text_input, text_area),
        _ => {
            let text_input = cx.app.models_mut().insert(String::new());
            let text_area = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| {
                st.text_input = Some(text_input.clone());
                st.text_area = Some(text_area.clone());
            });
            (text_input, text_area)
        }
    };

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Profile").into_element(cx),
        shadcn::FieldDescription::new("Group related fields to keep structure explicit.")
            .into_element(cx),
        shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Display name").into_element(cx),
                shadcn::Input::new(text_input.clone())
                    .placeholder("Evil Rabbit")
                    .a11y_label("Display name")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Bio").into_element(cx),
                shadcn::Textarea::new(text_area.clone())
                    .a11y_label("Bio")
                    .refine_layout(LayoutRefinement::default().h_px(Px(88.0)))
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Button::new("Submit").into_element(cx),
                shadcn::Button::new("Cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-form-fieldset")
}
// endregion: example
