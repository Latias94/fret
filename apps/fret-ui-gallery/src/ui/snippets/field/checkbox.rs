pub const SOURCE: &str = include_str!("checkbox.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let checkbox_a = cx.local_model_keyed("checkbox_a", || true);
    let checkbox_b = cx.local_model_keyed("checkbox_b", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldGroup::new([shadcn::FieldSet::new([
        shadcn::FieldLegend::new("Show these items on the desktop")
            .variant(shadcn::FieldLegendVariant::Label)
            .into_element(cx),
        shadcn::FieldDescription::new("Select the items you want to show.").into_element(cx),
        shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::Checkbox::new(checkbox_a)
                    .a11y_label("Hard disks")
                    .into_element(cx),
                shadcn::FieldLabel::new("Hard disks").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Checkbox::new(checkbox_b)
                    .a11y_label("External disks")
                    .into_element(cx),
                shadcn::FieldLabel::new("External disks").into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .checkbox_group()
        .into_element(cx),
    ])
    .into_element(cx)])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-checkbox")
}
// endregion: example
