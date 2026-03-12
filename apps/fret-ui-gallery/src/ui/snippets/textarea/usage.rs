pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model(String::new);

    shadcn::Textarea::new(value)
        .a11y_label("Message")
        .placeholder("Type your message here.")
        .into_element(cx)
        .test_id("ui-gallery-textarea-usage")
}
// endregion: example
