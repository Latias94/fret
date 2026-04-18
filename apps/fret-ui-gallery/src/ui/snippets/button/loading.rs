pub const SOURCE: &str = include_str!("loading.rs");

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
            shadcn::Button::new("Generating")
                .variant(shadcn::ButtonVariant::Outline)
                .disabled(true)
                .test_id("ui-gallery-button-loading-submit")
                .leading_child(shadcn::Spinner::new().into_element(cx))
                .into_element(cx),
            shadcn::Button::new("Downloading")
                .variant(shadcn::ButtonVariant::Secondary)
                .disabled(true)
                .test_id("ui-gallery-button-loading-download")
                .trailing_child(shadcn::Spinner::new().into_element(cx))
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-button-loading")
}
// endregion: example
