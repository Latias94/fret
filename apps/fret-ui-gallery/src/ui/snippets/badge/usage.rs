pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Badge::new("Badge")
        .variant(shadcn::BadgeVariant::Default)
        .into_element(cx)
        .test_id("ui-gallery-badge-usage")
}
// endregion: example
