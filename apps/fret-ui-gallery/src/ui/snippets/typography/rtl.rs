pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        ui::v_stack(|cx| {
            vec![
                shadcn::raw::typography::h3("RTL Sample").into_element(cx),
                shadcn::raw::typography::p(
                    "This block validates right-to-left direction in typography surfaces.",
                )
                .into_element(cx),
                shadcn::raw::typography::muted(
                    "Check paragraph wrapping and heading alignment under RTL.",
                )
                .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_start()
        .into_element(cx)
    })
    .test_id("ui-gallery-typography-rtl")
}
// endregion: example
