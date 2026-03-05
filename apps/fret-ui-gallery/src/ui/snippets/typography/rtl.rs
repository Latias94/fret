pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        ui::v_stack(|cx| {
            vec![
                shadcn::typography::h3(cx, "RTL Sample"),
                shadcn::typography::p(
                    cx,
                    "This block validates right-to-left direction in typography surfaces.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Check paragraph wrapping and heading alignment under RTL.",
                ),
            ]
        })
        .gap(Space::N2)
        .items_start()
        .into_element(cx)
    })
    .test_id("ui-gallery-typography-rtl")
}
// endregion: example
