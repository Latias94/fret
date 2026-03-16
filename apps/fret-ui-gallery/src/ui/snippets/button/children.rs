pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{UiChild, UiCx};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    wrap_row(|cx| {
        vec![
            shadcn::Button::new("Refresh")
                .variant(shadcn::ButtonVariant::Outline)
                .leading_child(shadcn::Spinner::new().speed(0.0).into_element(cx))
                .trailing_child(shadcn::Kbd::new("Cmd+R").into_element(cx))
                .test_id("ui-gallery-button-children-slotted")
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Secondary)
                .a11y_label("Open command menu")
                .child(cx.text("Command Menu"))
                .child(shadcn::Kbd::new("Cmd+K").into_element(cx))
                .test_id("ui-gallery-button-children-command")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-button-children-row")
}
// endregion: example
