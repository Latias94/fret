pub const SOURCE: &str = include_str!("field_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let checkbox_a = cx.local_model_keyed("checkbox_a", || true);
    let checkbox_b = cx.local_model_keyed("checkbox_b", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldGroup::new([
        shadcn::FieldSet::new([
            shadcn::FieldLabel::new("Responses").into_element(cx),
            shadcn::FieldDescription::new("Get notified for long-running responses.")
                .into_element(cx),
            shadcn::FieldGroup::new([shadcn::Field::new([
                shadcn::Checkbox::new(checkbox_a.clone())
                    .disabled(true)
                    .a11y_label("Push notifications")
                    .into_element(cx),
                shadcn::FieldLabel::new("Push notifications").into_element(cx),
            ])
            .disabled(true)
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx)])
            .checkbox_group()
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::FieldSeparator::new().into_element(cx),
        shadcn::FieldSet::new([
            shadcn::FieldLabel::new("Tasks").into_element(cx),
            shadcn::FieldDescription::new("Get notified when task status changes.")
                .into_element(cx),
            shadcn::FieldGroup::new([shadcn::Field::new([
                shadcn::Checkbox::new(checkbox_b)
                    .a11y_label("Email notifications")
                    .into_element(cx),
                shadcn::FieldLabel::new("Email notifications").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx)])
            .checkbox_group()
            .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-group")
}
// endregion: example
