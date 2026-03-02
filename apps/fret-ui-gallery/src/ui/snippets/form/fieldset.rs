pub const SOURCE: &str = include_str!("fieldset.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    max_w_md: LayoutRefinement,
) -> AnyElement {
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
