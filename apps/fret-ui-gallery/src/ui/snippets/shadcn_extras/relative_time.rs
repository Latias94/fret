// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::extras::RelativeTime::new([
        shadcn::extras::RelativeTimeZone::new("UTC", "February 9, 2026", "15:04:05")
            .into_element(cx),
        shadcn::extras::RelativeTimeZone::new("PST", "February 9, 2026", "07:04:05")
            .into_element(cx),
        shadcn::extras::RelativeTimeZone::new("CET", "February 9, 2026", "16:04:05")
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-relative-time")
}
// endregion: example
