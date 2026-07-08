pub const SOURCE: &str = include_str!("announcement.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::extras::Announcement::new([
        shadcn::extras::AnnouncementTag::new("New").into_element(cx),
        shadcn::extras::AnnouncementTitle::new([cx.text("Shadcn Extras landed in Fret")])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-announcement")
}
// endregion: example
