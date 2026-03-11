pub const SOURCE: &str = include_str!("banner.rs");

// region: example
use fret_icons::IconId;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let icon = fret_ui_shadcn::icon::icon(cx, IconId::new_static("lucide.info"));
    fret_ui_shadcn::extras::Banner::new([
        fret_ui_shadcn::extras::BannerIcon::new(icon).into_element(cx),
        fret_ui_shadcn::extras::BannerTitle::new("A new version is available.").into_element(cx),
        fret_ui_shadcn::extras::BannerAction::new("Upgrade").into_element(cx),
        fret_ui_shadcn::extras::BannerClose::new().into_element(cx),
    ])
    .inset(true)
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-banner")
}
// endregion: example
