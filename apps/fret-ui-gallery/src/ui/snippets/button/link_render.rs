// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(cx, children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![shadcn::Button::new("Dashboard")
            .render(shadcn::ButtonRender::Link {
                href: Arc::<str>::from("https://example.com/dashboard"),
                target: None,
                rel: None,
            })
            // Keep the gallery deterministic: demonstrate link semantics without opening the
            // browser during scripted runs.
            .on_click("ui_gallery.app.open")
            .test_id("ui-gallery-button-render-link")
            .into_element(cx)]
    })
    .test_id("ui-gallery-button-render-link-row")
}
// endregion: example

