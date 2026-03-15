pub const SOURCE: &str = include_str!("link.rs");

// region: example
use fret::view::AppActivateExt as _;
use fret::{UiChild, UiCx};
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    fret_ui_kit::ui::h_flex(|cx| {
        vec![
            // Upstream shadcn: `<Badge asChild><Link href="/">Badge</Link></Badge>`.
            shadcn::Badge::new("Open Link")
                .render(shadcn::BadgeRender::Link {
                    href: Arc::from("https://example.com"),
                    target: None,
                    rel: None,
                })
                .trailing_icon(IconId::new_static("lucide.arrow-up-right"))
                // Avoid launching the system browser during diag runs; the render surface still applies
                // link semantics and Enter-only activation.
                .listen(cx, |_host, _acx| {})
                .test_id("ui-gallery-badge-link")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .wrap()
    .w_full()
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-badge-link-row")
}
// endregion: example
