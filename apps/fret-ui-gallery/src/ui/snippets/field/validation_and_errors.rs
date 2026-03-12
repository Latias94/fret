pub const SOURCE: &str = include_str!("validation_and_errors.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let email_invalid = cx.local_model(String::new);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Email").into_element(cx),
        shadcn::Input::new(email_invalid)
            .placeholder("email@example.com")
            .a11y_label("Email")
            .aria_invalid(true)
            .into_element(cx),
        shadcn::FieldError::new("Enter a valid email address.").into_element(cx),
    ])
    .invalid(true)
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-validation-and-errors")
}
// endregion: example
