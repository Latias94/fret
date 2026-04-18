pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model(String::new);

    shadcn::Textarea::new(value)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .into_element(cx)
        .test_id("ui-gallery-textarea-usage")
}
// endregion: example
