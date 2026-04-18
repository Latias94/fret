pub const SOURCE: &str = include_str!("with_icon.rs");

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
            shadcn::Button::new("New Branch")
                .variant(shadcn::ButtonVariant::Outline)
                .leading_icon(IconId::new_static("lucide.git-branch"))
                .test_id("ui-gallery-button-with-icon")
                .into_element(cx),
            shadcn::Button::new("Fork")
                .variant(shadcn::ButtonVariant::Outline)
                .trailing_icon(IconId::new_static("lucide.git-fork"))
                .test_id("ui-gallery-button-with-trailing-icon")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-button-with-icon-row")
}
// endregion: example
