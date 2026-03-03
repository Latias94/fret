pub const SOURCE: &str = include_str!("link.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::ui::h_flex(cx, |cx| {
        vec![
            // Upstream shadcn: `<Badge asChild><Link href="/">Badge</Link></Badge>`.
            shadcn::Badge::new("Badge")
                .render(shadcn::BadgeRender::Link {
                    href: Arc::from("https://example.com"),
                    target: None,
                    rel: None,
                })
                // Avoid launching the system browser during diag runs; the render surface still applies
                // link semantics and Enter-only activation.
                .on_activate(Arc::new(|_host, _acx, _reason| {}))
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
