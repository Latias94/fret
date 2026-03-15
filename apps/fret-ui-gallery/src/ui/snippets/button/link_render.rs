pub const SOURCE: &str = include_str!("link_render.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    wrap_row(|cx| {
        vec![
            shadcn::Button::new("Login")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .render(shadcn::ButtonRender::Link {
                    href: Arc::<str>::from("https://example.com/login"),
                    target: None,
                    rel: None,
                })
                // Keep the gallery deterministic: demonstrate link semantics without opening the
                // browser during scripted runs.
                .on_click("ui_gallery.app.open")
                .test_id("ui-gallery-button-render-link")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-button-render-link-row")
}
// endregion: example
