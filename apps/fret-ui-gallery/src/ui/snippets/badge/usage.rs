pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Badge::new("Badge")
        .variant(shadcn::BadgeVariant::Default)
        .into_element(cx)
        .test_id("ui-gallery-badge-usage")
}
// endregion: example
