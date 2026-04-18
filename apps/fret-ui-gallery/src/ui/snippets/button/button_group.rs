pub const SOURCE: &str = include_str!("button_group.rs");

// region: example
use fret::{AppComponentCx, UiChild};
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

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    wrap_row(|cx| {
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
    .into_element(cx)
    .test_id("ui-gallery-button-button-group-row")
}
// endregion: example
