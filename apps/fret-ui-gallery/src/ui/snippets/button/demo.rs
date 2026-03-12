pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(|cx| {
        vec![
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-button-demo-text")
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .a11y_label("Submit")
                .icon(IconId::new_static("lucide.arrow-up"))
                .test_id("ui-gallery-button-demo-icon")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-button-demo")
}
// endregion: example
