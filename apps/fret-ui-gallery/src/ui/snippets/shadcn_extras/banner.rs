pub const SOURCE: &str = include_str!("banner.rs");

// region: example
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let icon = shadcn::raw::icon::icon(cx, IconId::new_static("lucide.info"));
    shadcn::raw::extras::Banner::new([
        shadcn::raw::extras::BannerIcon::new(icon).into_element(cx),
        shadcn::raw::extras::BannerTitle::new("A new version is available.").into_element(cx),
        shadcn::raw::extras::BannerAction::new("Upgrade").into_element(cx),
        shadcn::raw::extras::BannerClose::new().into_element(cx),
    ])
    .inset(true)
    .into_element(cx)
    .test_id("ui-gallery-shadcn-extras-banner")
}
// endregion: example
