pub const SOURCE: &str = include_str!("variants.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

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
        vec![
            shadcn::Button::new("Default")
                .test_id("ui-gallery-button-variant-default")
                .into_element(cx),
            shadcn::Button::new("Secondary")
                .variant(shadcn::ButtonVariant::Secondary)
                .test_id("ui-gallery-button-variant-secondary")
                .into_element(cx),
            shadcn::Button::new("Destructive")
                .variant(shadcn::ButtonVariant::Destructive)
                .test_id("ui-gallery-button-variant-destructive")
                .into_element(cx),
            shadcn::Button::new("Outline")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-button-variant-outline")
                .into_element(cx),
            shadcn::Button::new("Ghost")
                .variant(shadcn::ButtonVariant::Ghost)
                .test_id("ui-gallery-button-variant-ghost")
                .into_element(cx),
            shadcn::Button::new("Link")
                .variant(shadcn::ButtonVariant::Link)
                .test_id("ui-gallery-button-variant-link")
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-button-variants")
}
// endregion: example
