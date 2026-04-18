pub const SOURCE: &str = include_str!("anatomy.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", String::new);

    shadcn::Field::new([
        shadcn::FieldLabel::new("Label")
            .for_control("ui-gallery-field-anatomy-input")
            .into_element(cx),
        shadcn::Input::new(value)
            .control_id("ui-gallery-field-anatomy-input")
            .placeholder("Input / Select / Switch / ...")
            .a11y_label("Label")
            .aria_invalid(true)
            .into_element(cx),
        shadcn::FieldDescription::new("Optional helper text.")
            .for_control("ui-gallery-field-anatomy-input")
            .into_element(cx),
        shadcn::FieldError::new("Validation message.")
            .for_control("ui-gallery-field-anatomy-input")
            .into_element(cx),
    ])
    .invalid(true)
    .into_element(cx)
    .test_id("ui-gallery-field-anatomy")
}
// endregion: example
