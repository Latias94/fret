pub const SOURCE: &str = include_str!("announcement.rs");

// region: example
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_shadcn::extras::Announcement::new([
        fret_ui_shadcn::extras::AnnouncementTag::new("New").into_element(cx),
        fret_ui_shadcn::extras::AnnouncementTitle::new([cx.text("Shadcn Extras landed in Fret")])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-announcement")
}
// endregion: example
