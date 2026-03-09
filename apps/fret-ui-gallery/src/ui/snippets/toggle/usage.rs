pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Toggle::uncontrolled(false)
        .a11y_label("Toggle formatting")
        .children([ui::text("Toggle").into_element(cx)])
        .into_element(cx)
        .test_id("ui-gallery-toggle-usage")
}
// endregion: example
