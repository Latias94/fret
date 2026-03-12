pub const SOURCE: &str = include_str!("field.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Username").into_element(cx),
        shadcn::Input::new(value)
            .a11y_label("Username")
            .placeholder("Enter your username")
            .into_element(cx),
        shadcn::FieldDescription::new("Choose a unique username for your account.")
            .into_element(cx),
    ])
    .refine_layout(max_w_xs)
    .into_element(cx)
    .test_id("ui-gallery-input-field")
}
// endregion: example
