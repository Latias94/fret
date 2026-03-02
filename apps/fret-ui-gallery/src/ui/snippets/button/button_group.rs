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
            shadcn::ButtonGroup::new(
                [
                    shadcn::Button::new("Left").variant(shadcn::ButtonVariant::Outline),
                    shadcn::Button::new("Middle").variant(shadcn::ButtonVariant::Outline),
                    shadcn::Button::new("Right").variant(shadcn::ButtonVariant::Outline),
                ]
                .into_iter()
                .map(Into::into),
            )
            .a11y_label("Button group")
            .into_element(cx)
            .test_id("ui-gallery-button-button-group"),
        ]
    })
    .test_id("ui-gallery-button-button-group-row")
}
// endregion: example
