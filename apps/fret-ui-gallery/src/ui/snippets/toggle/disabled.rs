pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .disabled(true)
                    .a11y_label("Toggle disabled")
                    .label("Disabled")
                    .into_element(cx),
                shadcn::Toggle::uncontrolled(false)
                    .disabled(true)
                    .variant(shadcn::ToggleVariant::Outline)
                    .a11y_label("Toggle disabled outline")
                    .label("Disabled")
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-toggle-disabled")
}
// endregion: example
