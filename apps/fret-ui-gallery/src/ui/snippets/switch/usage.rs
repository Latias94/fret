pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Switch::new(cx.local_model(|| false))
        .a11y_label("Airplane mode")
        .into_element(cx)
        .test_id("ui-gallery-switch-usage")
}
// endregion: example
