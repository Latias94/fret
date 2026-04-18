pub const SOURCE: &str = include_str!("rounded.rs");

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
            shadcn::Button::new("Get Started")
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .test_id("ui-gallery-button-rounded")
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .a11y_label("Scroll to top")
                .icon(IconId::new_static("lucide.arrow-up"))
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .test_id("ui-gallery-button-rounded-icon")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-button-rounded-row")
}
// endregion: example
