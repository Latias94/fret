pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Spinner::new()
        .into_element(cx)
        .test_id("ui-gallery-spinner-usage")
}

// endregion: example
