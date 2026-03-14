pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::Field::new([
        shadcn::FieldLabel::new("Message").into_element(cx),
        shadcn::Textarea::new(value)
            .a11y_label("Message")
            .placeholder("Type your message here.")
            .aria_invalid(true)
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
    ])
    .invalid(true)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
    .into_element(cx)
    .test_id("ui-gallery-textarea-invalid")
}
// endregion: example
