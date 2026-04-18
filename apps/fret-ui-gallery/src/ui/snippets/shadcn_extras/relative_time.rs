pub const SOURCE: &str = include_str!("relative_time.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::raw::extras::RelativeTime::new([
        shadcn::raw::extras::RelativeTimeZone::new("UTC", "February 9, 2026", "15:04:05")
            .into_element(cx),
        shadcn::raw::extras::RelativeTimeZone::new("PST", "February 9, 2026", "07:04:05")
            .into_element(cx),
        shadcn::raw::extras::RelativeTimeZone::new("CET", "February 9, 2026", "16:04:05")
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-relative-time")
}
// endregion: example
