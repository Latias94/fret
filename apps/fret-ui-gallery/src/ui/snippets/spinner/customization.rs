pub const SOURCE: &str = include_str!("customization.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new()
                    .icon(fret_icons::ids::ui::SETTINGS)
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-spinner-customization")
}
// endregion: example
