// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::extras::Announcement::new([
        shadcn::extras::AnnouncementTag::new("New").into_element(cx),
        shadcn::extras::AnnouncementTitle::new([cx.text("Shadcn Extras landed in Fret")])
            .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-announcement")
}
// endregion: example
