pub const SOURCE: &str = include_str!("rounded.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![
            shadcn::Button::new("Rounded")
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .test_id("ui-gallery-button-rounded")
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-button-rounded-row")
}
// endregion: example
