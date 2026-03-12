pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text_input = cx.local_model_keyed("text_input", String::new);
    let text_area = cx.local_model_keyed("text_area", String::new);

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
