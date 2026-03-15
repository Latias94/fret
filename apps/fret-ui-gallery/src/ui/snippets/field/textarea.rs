pub const SOURCE: &str = include_str!("textarea.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let feedback = cx.local_model(String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::FieldSet::new([shadcn::FieldGroup::new([shadcn::Field::new([
        shadcn::FieldLabel::new("Feedback").into_element(cx),
        shadcn::Textarea::new(feedback)
            .a11y_label("Feedback")
            .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
            .into_element(cx),
        shadcn::FieldDescription::new("Share your thoughts about our service.").into_element(cx),
    ])
    .into_element(cx)])
    .into_element(cx)])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-textarea")
}
// endregion: example
